use crate::platform;

/// Handle clipboard command — read current clipboard text.
pub fn handle_clipboard() -> Result<String, String> {
    platform::clipboard_read()
}
