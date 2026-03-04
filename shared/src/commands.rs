use serde::{Deserialize, Serialize};

/// All commands that can be executed on a client or locally.
/// Each variant carries its own arguments if needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// List files at path (default: cwd)
    Ls { path: Option<String> },
    /// Print working directory
    Pwd,
    /// Change directory
    Cd { path: String },
    /// Print file contents
    Cat { path: String },
    /// Show device name + logged-in user
    Whoami,
    /// Full system information
    Sysinfo,
    /// List all running processes
    Processes,
    /// Kill a process by PID
    Kill { pid: u32 },
    /// Disk and RAM info
    DiskInfo,
    /// Lock the screen
    Lock,
    /// Shutdown the machine
    Shutdown,
    /// Restart the machine
    Restart,
    /// Show a desktop notification
    Notify { message: String },
    /// Play an alert sound
    Alert,
    /// Network-based geolocation
    Locate,
    /// Network information
    NetInfo,
    /// List user accounts
    Users,
    /// Capture screenshot
    Screenshot,
    /// Read clipboard text
    Clipboard,
    /// Battery status
    Battery,
    /// Upload file from admin to client
    Upload {
        filename: String,
        data: Vec<u8>,
    },
    /// Download file from client to admin
    Download { path: String },
    /// Ping/pong latency check
    Ping,
    /// Execute a raw shell command
    Shell { command: String },
}
