pub mod layout;
pub mod widgets;

use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::SharedState;
use crate::repl;

use self::layout::{centered_rect, content_layout, main_layout};
use self::widgets::{
    client_list::ClientListWidget,
    help_overlay::HelpOverlayWidget,
    output_panel::{OutputPanelWidget, cursor_position},
    status_bar::StatusBarWidget,
};

/// Run the TUI event loop.
pub async fn run_ui(state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Welcome message
    {
        let mut st = state.lock().await;
        st.add_output(
            "system",
            "Welcome to Fluffy Terminal v0.1.0",
            fluffy_ui::theme::BRAND_COLOR,
        );
        st.add_output(
            "system",
            "Type 'fluffy --help' for commands, 'rolecall' to see clients.",
            fluffy_ui::theme::DIM_COLOR,
        );
    }

    loop {
        // Check quit flag
        {
            let st = state.lock().await;
            if st.should_quit {
                break;
            }
        }

        {
            let st = state.lock().await;

            terminal.draw(|f| {
                let size = f.size();
                let (status_area, content_area) = main_layout(size);
                let (client_area, output_area) = content_layout(content_area);

                // Render status bar
                f.render_widget(StatusBarWidget { state: &st }, status_area);

                // Render client list
                f.render_widget(ClientListWidget { state: &st }, client_area);

                // Render output panel
                let (output_title, output_lines) = match st.mode {
                    crate::app::TerminalMode::Admin => (" Admin Output ", &st.output_lines[..]),
                    crate::app::TerminalMode::Client => (" Client Agent Logs ", &st.client_output[..]),
                };
                f.render_widget(OutputPanelWidget { 
                    state: &st, 
                    title: output_title,
                    lines: output_lines,
                }, output_area);

                // Set cursor position (inside the output panel)
                let (cx, cy) = cursor_position(&st, output_area);
                f.set_cursor(cx, cy);

                // Render help overlay if active
                if st.show_help {
                    let help_area = centered_rect(70, 80, size);
                    f.render_widget(
                        HelpOverlayWidget {
                            scroll: st.help_scroll,
                        },
                        help_area,
                    );
                }
            })?;
        }

        // Handle events with a short timeout for responsive UI updates
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            match ev {
                // Handle mouse events (wheel scroll)
                Event::Mouse(mouse) => {
                    let mut st = state.lock().await;
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if st.scroll_offset == u32::MAX {
                                let len = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                };
                                st.scroll_offset = (len as u32).saturating_sub(1);
                            } else {
                                st.scroll_offset = st.scroll_offset.saturating_sub(1);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if st.scroll_offset != u32::MAX {
                                st.scroll_offset = st.scroll_offset.saturating_add(1);
                                let max = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                } as u32;
                                if st.scroll_offset >= max {
                                    st.scroll_offset = u32::MAX;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Handle keyboard events
                Event::Key(key) => {
                    // On Windows, Crossterm sends both Press and Release events.
                    // We only want to process Press events to avoid double-processing characters.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    let mut st = state.lock().await;

                    // Handle help overlay keys
                    if st.show_help {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                st.show_help = false;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                st.help_scroll = st.help_scroll.saturating_add(1);
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                st.help_scroll = st.help_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown => {
                                st.help_scroll = st.help_scroll.saturating_add(10);
                            }
                            KeyCode::PageUp => {
                                st.help_scroll = st.help_scroll.saturating_sub(10);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        // Ctrl+C to exit
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            st.should_quit = true;
                        }

                        // Enter to submit command
                        KeyCode::Enter => {
                            if !st.input_buffer.is_empty() {
                                let input = st.input_buffer.clone();
                                st.input_buffer.clear();
                                st.cursor_pos = 0;
                                st.history_index = None;
                                st.saved_input.clear();
                                drop(st);
                                repl::process_input(&state, &input).await;
                            }
                        }

                        // Backspace
                        KeyCode::Backspace => {
                            if st.cursor_pos > 0 {
                                st.cursor_pos -= 1;
                                let pos = st.cursor_pos;
                                st.input_buffer.remove(pos);
                            }
                        }

                        // Delete
                        KeyCode::Delete => {
                            let pos = st.cursor_pos;
                            if pos < st.input_buffer.len() {
                                st.input_buffer.remove(pos);
                            }
                        }

                        // Left arrow
                        KeyCode::Left => {
                            if st.cursor_pos > 0 {
                                st.cursor_pos -= 1;
                            }
                        }

                        // Right arrow
                        KeyCode::Right => {
                            if st.cursor_pos < st.input_buffer.len() {
                                st.cursor_pos += 1;
                            }
                        }

                        // Home
                        KeyCode::Home => {
                            st.cursor_pos = 0;
                        }

                        // End
                        KeyCode::End => {
                            st.cursor_pos = st.input_buffer.len();
                        }

                        // Up arrow — History navigation
                        KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if !st.command_history.is_empty() {
                                let new_index = match st.history_index {
                                    Some(idx) => if idx > 0 { Some(idx - 1) } else { Some(0) },
                                    None => {
                                        st.saved_input = st.input_buffer.clone();
                                        Some(st.command_history.len() - 1)
                                    }
                                };
                                
                                if let Some(idx) = new_index {
                                    st.history_index = Some(idx);
                                    st.input_buffer = st.command_history[idx].clone();
                                    st.cursor_pos = st.input_buffer.len();
                                }
                            }
                        }

                        // Down arrow — History navigation
                        KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if let Some(idx) = st.history_index {
                                let new_index = idx + 1;
                                if new_index < st.command_history.len() {
                                    st.history_index = Some(new_index);
                                    st.input_buffer = st.command_history[new_index].clone();
                                    st.cursor_pos = st.input_buffer.len();
                                } else {
                                    st.history_index = None;
                                    st.input_buffer = st.saved_input.clone();
                                    st.cursor_pos = st.input_buffer.len();
                                }
                            }
                        }

                        // Shift + Up — scroll output up
                        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if st.scroll_offset == u32::MAX {
                                // Start manual scroll from the bottom
                                let len = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                };
                                st.scroll_offset = (len as u32).saturating_sub(1);
                            } else {
                                st.scroll_offset = st.scroll_offset.saturating_sub(1);
                            }
                        }

                        // Shift + Down — scroll output down
                        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if st.scroll_offset != u32::MAX {
                                st.scroll_offset = st.scroll_offset.saturating_add(1);
                                let max = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                } as u32;
                                if st.scroll_offset >= max {
                                    // Re-enable sticky bottom if we hit the end
                                    st.scroll_offset = u32::MAX;
                                }
                            }
                        }

                        // Page Up
                        KeyCode::PageUp => {
                            if st.scroll_offset == u32::MAX {
                                let len = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                };
                                st.scroll_offset = (len as u32).saturating_sub(10);
                            } else {
                                st.scroll_offset = st.scroll_offset.saturating_sub(10);
                            }
                        }

                        // Page Down
                        KeyCode::PageDown => {
                            if st.scroll_offset != u32::MAX {
                                st.scroll_offset = st.scroll_offset.saturating_add(10);
                                let max = match st.mode {
                                    crate::app::TerminalMode::Admin => st.output_lines.len(),
                                    crate::app::TerminalMode::Client => st.client_output.len(),
                                } as u32;
                                if st.scroll_offset >= max {
                                    st.scroll_offset = u32::MAX;
                                }
                            }
                        }

                        // Escape
                        KeyCode::Esc => {
                            st.input_buffer.clear();
                            st.cursor_pos = 0;
                        }

                        // Regular character input
                        KeyCode::Char(c) => {
                            let pos = st.cursor_pos;
                            st.input_buffer.insert(pos, c);
                            st.cursor_pos += 1;
                        }

                        // Tab — could be used for auto-complete in the future
                        KeyCode::Tab => {
                            // No-op for now
                        }

                        _ => {}
                    }
                }
                _ => {} // Catch-all for other events (e.g., Resize, FocusGained/Lost)
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
