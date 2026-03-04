pub mod layout;
pub mod widgets;

use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
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
    input_bar::{cursor_position, InputBarWidget},
    output_panel::OutputPanelWidget,
    status_bar::StatusBarWidget,
};

/// Run the TUI event loop.
pub async fn run_ui(state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
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
                let (status_area, content_area, input_area) = main_layout(size);
                let (client_area, output_area) = content_layout(content_area);

                // Render status bar
                f.render_widget(StatusBarWidget { state: &st }, status_area);

                // Render client list
                f.render_widget(ClientListWidget { state: &st }, client_area);

                // Render output panel
                f.render_widget(OutputPanelWidget { state: &st }, output_area);

                // Render input bar
                f.render_widget(InputBarWidget { state: &st }, input_area);

                // Set cursor position
                let (cx, cy) = cursor_position(&st, input_area);
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
            if let Event::Key(key) = event::read()? {
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

                    // Up arrow — scroll output up
                    KeyCode::Up => {
                        st.scroll_offset = st.scroll_offset.saturating_sub(1);
                    }

                    // Down arrow — scroll output down
                    KeyCode::Down => {
                        st.scroll_offset = st.scroll_offset.saturating_add(1);
                        let max = st.output_lines.len();
                        if st.scroll_offset > max {
                            st.scroll_offset = max;
                        }
                    }

                    // Page Up
                    KeyCode::PageUp => {
                        st.scroll_offset = st.scroll_offset.saturating_sub(10);
                    }

                    // Page Down
                    KeyCode::PageDown => {
                        st.scroll_offset = st.scroll_offset.saturating_add(10);
                        let max = st.output_lines.len();
                        if st.scroll_offset > max {
                            st.scroll_offset = max;
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
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
