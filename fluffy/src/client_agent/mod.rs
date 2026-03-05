pub mod error;
pub mod executor;
pub mod handlers;
pub mod platform;

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use shared::protocol::{AdminCommand, ClientHandshake, ClientResponse};
use crate::app::{SharedState, ClientServiceStatus};
use fluffy_ui::theme;

pub async fn run_client_agent(state: SharedState, admin_ip: String) {
    let admin_addr = if admin_ip.contains(':') {
        admin_ip.clone()
    } else {
        format!("{}:{}", admin_ip, shared::DEFAULT_PORT)
    };

    {
        let mut st = state.lock().await;
        st.client_service_status = ClientServiceStatus::Connecting(admin_ip.clone());
        st.client_output.push(crate::app::OutputLine {
            tag: "client".to_string(),
            text: format!("Connecting to admin at {}...", admin_addr),
            color: theme::DIM_COLOR,
            timestamp: chrono::Local::now(),
        });
    }

    let stream = match TcpStream::connect(&admin_addr).await {
        Ok(s) => {
            let mut st = state.lock().await;
            st.client_service_status = ClientServiceStatus::Running(admin_ip.clone());
            st.client_output.push(crate::app::OutputLine {
                tag: "client".to_string(),
                text: "Connected to admin.".to_string(),
                color: theme::SUCCESS_COLOR,
                timestamp: chrono::Local::now(),
            });
            s
        }
        Err(e) => {
            let mut st = state.lock().await;
            st.client_service_status = ClientServiceStatus::Error(e.to_string());
            st.client_output.push(crate::app::OutputLine {
                tag: "client".to_string(),
                text: format!("Failed to connect to {}: {}", admin_addr, e),
                color: theme::ERROR_COLOR,
                timestamp: chrono::Local::now(),
            });
            return;
        }
    };

    let (reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut reader = BufReader::new(reader);

    // Send handshake
    let handshake = ClientHandshake {
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        os: std::env::consts::OS.to_string(),
        os_version: get_os_version(),
        ip: get_local_ip(),
        arch: std::env::consts::ARCH.to_string(),
    };

    let handshake_json = match serde_json::to_string(&handshake) {
        Ok(j) => j,
        Err(e) => {
            let mut st = state.lock().await;
            st.client_output.push(crate::app::OutputLine {
                tag: "client".to_string(),
                text: format!("Failed to serialize handshake: {}", e),
                color: theme::ERROR_COLOR,
                timestamp: chrono::Local::now(),
            });
            return;
        }
    };

    {
        let mut w = writer.lock().await;
        let _ = w.write_all(format!("{}\n", handshake_json).as_bytes()).await;
        let _ = w.flush().await;
    }

    {
        let mut st = state.lock().await;
        st.client_output.push(crate::app::OutputLine {
            tag: "client".to_string(),
            text: "Handshake sent. Waiting for commands...".to_string(),
            color: theme::DIM_COLOR,
            timestamp: chrono::Local::now(),
        });
    }

    // Create executor state
    let executor_state = Arc::new(Mutex::new(executor::ExecutorState::new()));

    let mut line_buf = String::new();
    loop {
        // Check for shutdown signal
        {
            let st = state.lock().await;
            if let ClientServiceStatus::Stopped = st.client_service_status {
                break;
            }
        }

        line_buf.clear();
        match tokio::time::timeout(std::time::Duration::from_millis(500), reader.read_line(&mut line_buf)).await {
            Ok(Ok(0)) => {
                let mut st = state.lock().await;
                st.client_service_status = ClientServiceStatus::Stopped;
                st.client_output.push(crate::app::OutputLine {
                    tag: "client".to_string(),
                    text: "Admin disconnected.".to_string(),
                    color: theme::DIM_COLOR,
                    timestamp: chrono::Local::now(),
                });
                break;
            }
            Ok(Ok(_)) => {
                let line = line_buf.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                let admin_cmd: AdminCommand = match serde_json::from_str(&line) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        let mut st = state.lock().await;
                        st.client_output.push(crate::app::OutputLine {
                            tag: "client".to_string(),
                            text: format!("Failed to parse command: {}. Raw: {}", e, line),
                            color: theme::ERROR_COLOR,
                            timestamp: chrono::Local::now(),
                        });
                        continue;
                    }
                };

                let cmd_id = admin_cmd.id;
                let state_clone = Arc::clone(&state);
                let executor_state_clone = Arc::clone(&executor_state);
                let writer_clone = Arc::clone(&writer);

                tokio::spawn(async move {
                    let response = executor::execute(
                        cmd_id,
                        admin_cmd.command,
                        "client",
                        "",
                        &executor_state_clone,
                    )
                    .await;

                    let response_json = match serde_json::to_string(&response) {
                        Ok(j) => j,
                        Err(e) => {
                            let mut st = state_clone.lock().await;
                            st.client_output.push(crate::app::OutputLine {
                                tag: "client".to_string(),
                                text: format!("Failed to serialize response: {}", e),
                                color: theme::ERROR_COLOR,
                                timestamp: chrono::Local::now(),
                            });
                            let err_resp = ClientResponse {
                                id: cmd_id,
                                client_id: String::new(),
                                tag: String::new(),
                                output: format!("Internal serialization error: {}", e),
                                success: false,
                            };
                            serde_json::to_string(&err_resp).unwrap_or_default()
                        }
                    };

                    let mut w = writer_clone.lock().await;
                    let _ = w.write_all(format!("{}\n", response_json).as_bytes()).await;
                    let _ = w.flush().await;
                });
            }
            Ok(Err(e)) => {
                let mut st = state.lock().await;
                st.client_service_status = ClientServiceStatus::Error(e.to_string());
                st.client_output.push(crate::app::OutputLine {
                    tag: "client".to_string(),
                    text: format!("Read error: {}", e),
                    color: theme::ERROR_COLOR,
                    timestamp: chrono::Local::now(),
                });
                break;
            }
            Err(_) => {
                // Timeout, just continue and check status
                continue;
            }
        }
    }

    {
        let mut st = state.lock().await;
        st.client_service_status = ClientServiceStatus::Stopped;
        st.client_output.push(crate::app::OutputLine {
            tag: "client".to_string(),
            text: "Client agent stopped.".to_string(),
            color: theme::DIM_COLOR,
            timestamp: chrono::Local::now(),
        });
    }
}

fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_OperatingSystem).Version",
            ])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("VERSION="))
                    .map(|l| l.trim_start_matches("VERSION=").trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("sw_vers")
            .args(["-productVersion"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        "Unknown".to_string()
    }
}

fn get_local_ip() -> String {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0");
    match socket {
        Ok(s) => {
            if s.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = s.local_addr() {
                    return addr.ip().to_string();
                }
            }
            "127.0.0.1".to_string()
        }
        Err(_) => "127.0.0.1".to_string(),
    }
}
