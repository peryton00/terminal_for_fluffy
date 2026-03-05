use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::app::{ClientInfo, SharedState};
use fluffy_ui::theme;
use shared::protocol::{ClientHandshake, ClientResponse};

/// Start the TCP server on the configured port and accept client connections.
pub async fn start_server(state: SharedState) {
    let port = {
        let st = state.lock().await;
        st.admin_port
    };

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                &format!("Fluffy admin server listening on port {}", port),
                theme::BRAND_COLOR,
            );
            l
        }
        Err(e) => {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                &format!("Failed to bind port {}: {}", port, e),
                theme::ERROR_COLOR,
            );
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let addr_str = addr.to_string();
                let state_clone = Arc::clone(&state);
                tokio::spawn(handle_client(stream, addr_str, state_clone));
            }
            Err(e) => {
                let mut st = state.lock().await;
                st.add_output(
                    "system",
                    &format!("Accept error: {}", e),
                    theme::ERROR_COLOR,
                );
            }
        }
    }
}

/// Handle a single client connection.
async fn handle_client(
    stream: tokio::net::TcpStream,
    addr: String,
    state: SharedState,
) {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    // Read handshake
    let mut handshake_line = String::new();
    match reader.read_line(&mut handshake_line).await {
        Ok(0) => {
            return; // Client disconnected immediately
        }
        Ok(_) => {}
        Err(e) => {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                &format!("Handshake read error from {}: {}", addr, e),
                theme::ERROR_COLOR,
            );
            return;
        }
    }

    let handshake: ClientHandshake = match serde_json::from_str(handshake_line.trim()) {
        Ok(h) => h,
        Err(e) => {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                &format!("Invalid handshake from {}: {}", addr, e),
                theme::ERROR_COLOR,
            );
            return;
        }
    };

    // Create channel for sending commands to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Assign tag and register client
    let tag = {
        let mut st = state.lock().await;
        let tag = st.next_client_tag();
        let client_info = ClientInfo {
            tag: tag.clone(),
            hostname: handshake.hostname.clone(),
            os: handshake.os.clone(),
            os_version: handshake.os_version.clone(),
            ip: handshake.ip.clone(),
            arch: handshake.arch.clone(),
            sender: tx,
            connected_at: chrono::Local::now(),
        };

        st.clients.insert(addr.clone(), client_info);
        st.add_output(
            &tag,
            &format!(
                "Client [{}] connected: {} ({} {}) at {}",
                tag, handshake.hostname, handshake.os, handshake.os_version, handshake.ip
            ),
            theme::SUCCESS_COLOR,
        );
        tag
    };

    // Spawn writer task: forwards commands from the channel to the TCP stream
    let tag_for_writer = tag.clone();
    let addr_for_writer = addr.clone();
    let state_for_writer = Arc::clone(&state);
    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer
                .write_all(format!("{}\n", msg).as_bytes())
                .await
                .is_err()
            {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
        // Writer ended
        let mut st = state_for_writer.lock().await;
        let _ = st.remove_client(&addr_for_writer);
        st.add_output(
            &tag_for_writer,
            &format!("Client [{}] writer disconnected.", tag_for_writer),
            theme::WARNING_COLOR,
        );
    });

    // Reader loop: read responses from the client
    let tag_for_reader = tag.clone();
    let state_for_reader = Arc::clone(&state);
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                // Client disconnected
                break;
            }
            Ok(_) => {
                let line = line_buf.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ClientResponse>(&line) {
                    Ok(response) => {
                        let mut st = state_for_reader.lock().await;
                        let color = if response.success {
                            theme::TEXT_COLOR
                        } else {
                            theme::ERROR_COLOR
                        };
                        st.add_output(&tag_for_reader, &response.output, color);
                    }
                    Err(e) => {
                        let mut st = state_for_reader.lock().await;
                        st.add_output(
                            &tag_for_reader,
                            &format!("Parse error from [{}]: {}", tag_for_reader, e),
                            theme::ERROR_COLOR,
                        );
                    }
                }
            }
            Err(e) => {
                let mut st = state_for_reader.lock().await;
                st.add_output(
                    &tag_for_reader,
                    &format!("Read error from [{}]: {}", tag_for_reader, e),
                    theme::ERROR_COLOR,
                );
                break;
            }
        }
    }

    // Cleanup
    {
        let mut st = state.lock().await;
        if let Some(removed_tag) = st.remove_client(&addr) {
            st.add_output(
                &removed_tag,
                &format!("Client [{}] disconnected.", removed_tag),
                theme::WARNING_COLOR,
            );
            if st.alter_target.as_ref() == Some(&removed_tag) {
                st.alter_target = None;
                st.add_output(
                    "system",
                    &format!(
                        "Alter target [{}] disconnected. Returning to local mode.",
                        removed_tag
                    ),
                    theme::WARNING_COLOR,
                );
            }
        }
    }

    writer_handle.abort();
}
