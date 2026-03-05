/// Fluffy client error handling utilities.
/// All errors are represented as String for serialization over the wire.

/// Wrap any operation that might panic into a safe Result.
pub fn safe_execute<F>(operation: &str, f: F) -> Result<String, String>
where
    F: FnOnce() -> Result<String, String> + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Internal error: '{}' panicked unexpectedly. This is a bug.",
            operation
        )),
    }
}

/// Format a command-not-supported error for the current OS.
pub fn unsupported_on_os(command: &str) -> String {
    format!(
        "Command '{}' is not supported on {}.",
        command,
        std::env::consts::OS
    )
}

/// Format a friendly "command failed" error.
pub fn command_failed(command: &str, reason: &str) -> String {
    format!("'{}' failed: {}", command, reason)
}
