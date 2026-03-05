use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    Admin,
    Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientServiceStatus {
    Stopped,
    Connecting(String),
    Running(String),
    Error(String),
}

/// Represents a connected client.
#[derive(Clone)]
pub struct ClientInfo {
    pub tag: String,
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub ip: String,
    pub arch: String,
    pub sender: mpsc::UnboundedSender<String>,
    pub connected_at: chrono::DateTime<chrono::Local>,
}

/// An output line shown in the output panel.
#[derive(Clone)]
pub struct OutputLine {
    pub tag: String,
    pub text: String,
    pub color: Color,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Full application state.
pub struct AppState {
    /// Connected clients keyed by socket address string
    pub clients: HashMap<String, ClientInfo>,
    /// All output lines to display
    pub output_lines: Vec<OutputLine>,
    /// Command history
    pub command_history: Vec<String>,
    /// Alter mode target (e.g., "f1")
    pub alter_target: Option<String>,
    /// Current input buffer
    pub input_buffer: String,
    /// Cursor position in the input buffer
    pub cursor_pos: usize,
    /// Index in history we are currently navigating (None if typing new command)
    pub history_index: Option<usize>,
    /// Temporary storage for the input buffer before navigating history
    pub saved_input: String,
    /// Whether the help overlay is shown
    pub show_help: bool,
    /// Scroll offset for the output panel (rendered lines)
    pub scroll_offset: u32,
    /// Counter for assigning client tags (never resets)
    pub client_counter: usize,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Help scroll offset
    pub help_scroll: u16,
    /// Last calculated total visual rows (for scroll transitions)
    pub last_visual_row_count: u32,
    /// Pending command ID counter
    pub command_id_counter: u64,
    /// Current display mode (Admin or Client)
    pub mode: TerminalMode,
    /// Status of the background client service
    pub client_service_status: ClientServiceStatus,
    /// Buffer for client-specific logs/output
    pub client_output: Vec<OutputLine>,
    /// The port the admin server is listening on
    pub admin_port: u16,
    /// Whether the TCP server is active in this process
    pub server_active: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            output_lines: Vec::new(),
            command_history: Vec::new(),
            alter_target: None,
            input_buffer: String::new(),
            cursor_pos: 0,
            history_index: None,
            saved_input: String::new(),
            show_help: false,
            scroll_offset: u32::MAX,
            client_counter: 0,
            should_quit: false,
            command_id_counter: 0,
            help_scroll: 0,
            last_visual_row_count: 0,
            mode: TerminalMode::Admin,
            client_service_status: ClientServiceStatus::Stopped,
            client_output: Vec::new(),
            admin_port: shared::DEFAULT_PORT,
            server_active: false,
        }
    }

    /// Get the next command ID.
    pub fn next_command_id(&mut self) -> u64 {
        self.command_id_counter += 1;
        self.command_id_counter
    }

    /// Assign a new tag (f1, f2, ...) for a connecting client.
    pub fn next_client_tag(&mut self) -> String {
        self.client_counter += 1;
        format!("f{}", self.client_counter)
    }

    /// Add an output line. Splits by newlines automatically.
    pub fn add_output(&mut self, tag: &str, text: &str, color: Color) {
        for line_text in text.lines() {
            self.output_lines.push(OutputLine {
                tag: tag.to_string(),
                text: line_text.to_string(),
                color,
                timestamp: chrono::Local::now(),
            });
        }
        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Add a client output line. Splits by newlines automatically.
    pub fn add_client_output(&mut self, tag: &str, text: &str, color: Color) {
        for line_text in text.lines() {
            self.client_output.push(OutputLine {
                tag: tag.to_string(),
                text: line_text.to_string(),
                color,
                timestamp: chrono::Local::now(),
            });
        }
        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Scroll to bottom of output.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = u32::MAX;
    }

    /// Get the prompt string.
    pub fn prompt(&self) -> String {
        match &self.alter_target {
            Some(tag) => format!("fluffy [{}]> ", tag),
            None => "fluffy> ".to_string(),
        }
    }

    /// Find a client by tag (f1, f2, ...).
    pub fn find_client_by_tag(&self, tag: &str) -> Option<(&String, &ClientInfo)> {
        self.clients.iter().find(|(_, c)| c.tag == tag)
    }

    /// Remove a client by socket address and return its tag.
    pub fn remove_client(&mut self, addr: &str) -> Option<String> {
        if let Some(client) = self.clients.remove(addr) {
            // If this was the alter target, clear alter mode
            if self.alter_target.as_ref() == Some(&client.tag) {
                self.alter_target = None;
            }
            Some(client.tag)
        } else {
            None
        }
    }
}

/// Thread-safe app state handle.
pub type SharedState = Arc<Mutex<AppState>>;

/// Create a new shared state.
pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState::new()))
}
