mod app;
mod client_agent;
mod client_manager;
mod local_executor;
mod net;
mod repl;
mod ui;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let state = app::new_shared_state();

    let mut admin_mode = false;
    let mut client_mode = false;
    let mut connect_ip = None;
    let mut port: u16 = shared::DEFAULT_PORT;

    // Very simple arg parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--admin" => {
                admin_mode = true;
            }
            "--client" => {
                client_mode = true;
            }
            "--connect" => {
                if i + 1 < args.len() {
                    connect_ip = Some(args[i + 1].clone());
                    client_mode = true;
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Default to admin mode if neither admin nor client mode was explicitly requested
    if !admin_mode && !client_mode {
        admin_mode = true;
    }

    // Set initial mode in state
    {
        let mut st = state.lock().await;
        if client_mode && !admin_mode {
            st.mode = app::TerminalMode::Client;
        } else {
            st.mode = app::TerminalMode::Admin;
        }
        st.admin_port = port;
        st.server_active = admin_mode;
    }

    if admin_mode {
        // Start the TCP server in the background
        let server_state = Arc::clone(&state);
        tokio::spawn(async move {
            net::start_server(server_state).await;
        });
    }

    if client_mode {
        if let Some(mut ip) = connect_ip {
            // If no port specified in the connect IP, use the provided --port or DEFAULT_PORT
            if !ip.contains(':') {
                ip = format!("{}:{}", ip, port);
            }
            let client_state = Arc::clone(&state);
            tokio::spawn(async move {
                client_agent::run_client_agent(client_state, ip).await;
            });
        }
    }

    // Run the TUI (this blocks until user exits)
    if let Err(e) = ui::run_ui(state).await {
        eprintln!("UI error: {}", e);
    }

    println!("Fluffy Terminal exited. Goodbye!");
}
