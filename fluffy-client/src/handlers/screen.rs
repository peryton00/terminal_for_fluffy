use crate::platform;

/// Handle screenshot command.
pub fn handle_screenshot() -> Result<String, String> {
    platform::screenshot()
}
