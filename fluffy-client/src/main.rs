mod error;
mod executor;
mod handlers;
mod platform;

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use shared::protocol::{AdminCommand, ClientHandshake, ClientResponse};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let admin_addr = if args.len() > 1 {
        format!("{}:9000", args[1])
    } else {
        eprintln!("Usage: fluffy-client <admin-ip>");
        eprintln!("Example: fluffy-client 192.168.1.100");
        std::process::exit(1);
    };

    println!("[fluffy-client] Connecting to admin at {}...", admin_addr);

    let stream = match TcpStream::connect(&admin_addr).await {
        Ok(s) => {
            println!("[fluffy-client] Connected to admin.");
            s
        }
        Err(e) => {
            eprintln!("[fluffy-client] Failed to connect to {}: {}", admin_addr, e);
            std::process::exit(1);
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
            eprintln!("[fluffy-client] Failed to serialize handshake: {}", e);
            return;
        }
    };

    {
        let mut w = writer.lock().await;
        if let Err(e) = w.write_all(format!("{}\n", handshake_json).as_bytes()).await {
            eprintln!("[fluffy-client] Failed to send handshake: {}", e);
            return;
        }
        if let Err(e) = w.flush().await {
            eprintln!("[fluffy-client] Failed to flush handshake: {}", e);
            return;
        }
    }

    println!("[fluffy-client] Handshake sent. Waiting for commands...");

    // Create executor state
    let state = Arc::new(Mutex::new(executor::ExecutorState::new()));

    // Read commands in a loop
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                println!("[fluffy-client] Admin disconnected.");
                break;
            }
            Ok(_) => {
                let line = line_buf.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // Parse the admin command
                let admin_cmd: AdminCommand = match serde_json::from_str(&line) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        eprintln!(
                            "[fluffy-client] Failed to parse command: {}. Raw: {}",
                            e, line
                        );
                        continue;
                    }
                };

                let cmd_id = admin_cmd.id;
                let state_clone = Arc::clone(&state);
                let writer_clone = Arc::clone(&writer);

                // Execute in a spawned task to not block the read loop
                tokio::spawn(async move {
                    let response = executor::execute(
                        cmd_id,
                        admin_cmd.command,
                        "client",
                        "",
                        &state_clone,
                    )
                    .await;

                    let response_json = match serde_json::to_string(&response) {
                        Ok(j) => j,
                        Err(e) => {
                            eprintln!("[fluffy-client] Failed to serialize response: {}", e);
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
                    if let Err(e) = w
                        .write_all(format!("{}\n", response_json).as_bytes())
                        .await
                    {
                        eprintln!("[fluffy-client] Failed to send response: {}", e);
                    }
                    let _ = w.flush().await;
                });
            }
            Err(e) => {
                eprintln!("[fluffy-client] Read error: {}", e);
                break;
            }
        }
    }

    println!("[fluffy-client] Exiting.");
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
