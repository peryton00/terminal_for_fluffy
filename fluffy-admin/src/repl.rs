use crate::app::SharedState;
use crate::client_manager;
use crate::local_executor;
use fluffy_ui::theme;
use shared::parser::parse_command;

/// Process a line of input from the user.
/// Handles admin commands, client targeting, alter mode, and local execution.
pub async fn process_input(state: &SharedState, raw_input: &str) {
    let input = raw_input.trim();
    if input.is_empty() {
        return;
    }

    // Add to history
    {
        let mut st = state.lock().await;
        st.command_history.push(input.to_string());
    }

    // ─── Admin-side built-in commands ───────────────────────

    // Exit/quit
    if input == "exit" || input == "quit" {
        let mut st = state.lock().await;
        st.should_quit = true;
        return;
    }

    // !! — re-run last command
    if input == "!!" {
        let last_cmd = {
            let st = state.lock().await;
            if st.command_history.len() < 2 {
                None
            } else {
                Some(st.command_history[st.command_history.len() - 2].clone())
            }
        };

        match last_cmd {
            Some(cmd) => {
                let mut st = state.lock().await;
                st.add_output("system", &format!("Re-running: {}", cmd), theme::DIM_COLOR);
                drop(st);
                // Use Box::pin for recursive async
                Box::pin(process_input(state, &cmd)).await;
            }
            None => {
                let mut st = state.lock().await;
                st.add_output("system", "No commands in history.", theme::WARNING_COLOR);
            }
        }
        return;
    }

    // Clean — clear output
    if input == "clean" || input == "clear" {
        let mut st = state.lock().await;
        st.output_lines.clear();
        st.scroll_offset = 0;
        return;
    }

    // History
    if input == "history" {
        let st = state.lock().await;
        if st.command_history.is_empty() {
            drop(st);
            let mut st = state.lock().await;
            st.add_output("system", "No commands in history.", theme::DIM_COLOR);
        } else {
            let lines: Vec<String> = st
                .command_history
                .iter()
                .enumerate()
                .map(|(i, cmd)| format!("  {}. {}", i + 1, cmd))
                .collect();
            let output = lines.join("\n");
            drop(st);
            let mut st = state.lock().await;
            st.add_output("system", &output, theme::TEXT_COLOR);
        }
        return;
    }

    // Rolecall
    if input == "rolecall" {
        let output = client_manager::rolecall(state).await;
        let mut st = state.lock().await;
        st.add_output("system", &output, theme::TEXT_COLOR);
        return;
    }

    // fluffy --help
    if input == "fluffy --help" || input == "help" || input == "--help" {
        let mut st = state.lock().await;
        st.show_help = true;
        st.help_scroll = 0;
        return;
    }

    // f alter
    if input.starts_with("f alter ") {
        let target = input.strip_prefix("f alter ").unwrap().trim();
        let mut st = state.lock().await;

        if target == "local" || target == "off" {
            st.alter_target = None;
            st.add_output(
                "system",
                "Alter mode cleared. Commands will run locally.",
                theme::SUCCESS_COLOR,
            );
        } else {
            // Verify the client exists
            let client_exists = st.clients.values().any(|c| c.tag == target);
            if client_exists {
                st.alter_target = Some(target.to_string());
                st.add_output(
                    "system",
                    &format!("Alter mode set to [{}]. All commands will target this client.", target),
                    theme::SUCCESS_COLOR,
                );
            } else {
                st.add_output(
                    "system",
                    &format!(
                        "Client [{}] not found. Use 'rolecall' to see connected clients.",
                        target
                    ),
                    theme::ERROR_COLOR,
                );
            }
        }
        return;
    }

    // Broadcast
    if input.starts_with("broadcast ") {
        let msg = input
            .strip_prefix("broadcast ")
            .unwrap()
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if msg.is_empty() {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                "Usage: broadcast \"message\"",
                theme::WARNING_COLOR,
            );
        } else {
            client_manager::broadcast_notify(state, msg).await;
        }
        return;
    }

    // ─── Check for client tag prefix (f1, f2, ...) ──────────
    let parts: Vec<&str> = input.splitn(2, char::is_whitespace).collect();
    let first_word = parts[0];

    // Check if first word is a client tag (f followed by digits)
    if first_word.starts_with('f') && first_word[1..].chars().all(|c| c.is_ascii_digit()) && parts.len() > 1 {
        let tag = first_word;
        let cmd_str = parts[1].trim();

        // Verify client exists
        let client_exists = {
            let st = state.lock().await;
            st.clients.values().any(|c| c.tag == tag)
        };

        if !client_exists {
            let mut st = state.lock().await;
            st.add_output(
                "system",
                &format!("Client [{}] not found.", tag),
                theme::ERROR_COLOR,
            );
            return;
        }

        // Show the command being sent
        {
            let mut st = state.lock().await;
            st.add_output(tag, &format!("[{}] {}", tag, cmd_str), theme::CLIENT_TAG_COLOR);
        }

        // Parse and send
        if let Some(command) = parse_command(cmd_str) {
            let _ = client_manager::send_command_to_client(state, tag, command).await;
        } else {
            let mut st = state.lock().await;
            st.add_output(
                tag,
                &format!(
                    "Unknown command: '{}'. Type 'fluffy --help' to see all commands.",
                    cmd_str
                ),
                theme::ERROR_COLOR,
            );
        }
        return;
    }

    // ─── Alter mode: send to the active target ──────────────
    let alter_target = {
        let st = state.lock().await;
        st.alter_target.clone()
    };

    if let Some(target) = alter_target {
        // Show the command being sent
        {
            let mut st = state.lock().await;
            st.add_output(
                &target,
                &format!("[{}] {}", target, input),
                theme::CLIENT_TAG_COLOR,
            );
        }

        if let Some(command) = parse_command(input) {
            let _ = client_manager::send_command_to_client(state, &target, command).await;
        } else {
            let mut st = state.lock().await;
            st.add_output(
                &target,
                &format!(
                    "Unknown command: '{}'. Type 'fluffy --help' to see all commands.",
                    input
                ),
                theme::ERROR_COLOR,
            );
        }
        return;
    }

    // ─── Local execution (no target, no alter mode) ─────────
    {
        let mut st = state.lock().await;
        st.add_output("local", &format!("[local] {}", input), theme::LOCAL_TAG_COLOR);
    }

    let output = local_executor::execute_local(input);
    let mut st = state.lock().await;
    st.add_output("local", &output, theme::TEXT_COLOR);
}
