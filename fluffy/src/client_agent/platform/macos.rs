use std::process::Command;

/// Lock screen on macOS
pub fn lock() -> Result<String, String> {
    let output = Command::new("pmset")
        .arg("displaysleepnow")
        .output()
        .map_err(|e| format!("Failed to lock screen: {}", e))?;
    if output.status.success() {
        Ok("Screen locked successfully.".to_string())
    } else {
        // Fallback
        let _ = Command::new("osascript")
            .args([
                "-e",
                r#"tell application "System Events" to keystroke "q" using {control down, command down}"#,
            ])
            .output();
        Ok("Screen lock initiated.".to_string())
    }
}

/// Shutdown on macOS
pub fn shutdown() -> Result<String, String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to shut down"#,
        ])
        .output()
        .map_err(|e| format!("Failed to shutdown: {}", e))?;
    if output.status.success() {
        Ok("Shutdown initiated.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Shutdown failed: {}", stderr.trim()))
    }
}

/// Restart on macOS
pub fn restart() -> Result<String, String> {
    let output = Command::new("osascript")
        .args(["-e", r#"tell application "System Events" to restart"#])
        .output()
        .map_err(|e| format!("Failed to restart: {}", e))?;
    if output.status.success() {
        Ok("Restart initiated.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Restart failed: {}", stderr.trim()))
    }
}

/// Show desktop notification on macOS using osascript
pub fn notify(msg: &str) -> Result<String, String> {
    let script = format!(
        r#"display notification "{}" with title "Fluffy Terminal""#,
        msg.replace('"', "\\\"")
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to show notification: {}", e))?;
    if output.status.success() {
        Ok(format!("Notification sent: {}", msg))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Notification failed: {}", stderr.trim()))
    }
}

/// Play alert sound on macOS
pub fn alert() -> Result<String, String> {
    let output = Command::new("afplay")
        .arg("/System/Library/Sounds/Sosumi.aiff")
        .output()
        .map_err(|e| format!("Failed to play alert: {}", e))?;
    if output.status.success() {
        Ok("Alert sound played.".to_string())
    } else {
        // Fallback to Glass sound
        let _ = Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .output();
        Ok("Alert sound played.".to_string())
    }
}

/// Take screenshot on macOS
pub fn screenshot() -> Result<String, String> {
    let path = "/tmp/fluffy_screenshot.png";
    let output = Command::new("screencapture")
        .args(["-x", path])
        .output()
        .map_err(|e| format!("Screenshot failed: {}", e))?;
    if output.status.success() {
        Ok(format!("Screenshot saved to {}", path))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Screenshot failed: {}", stderr.trim()))
    }
}

/// Read clipboard on macOS
pub fn clipboard_read() -> Result<String, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| format!("Failed to read clipboard: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err("Failed to read clipboard.".to_string())
    }
}

/// Get GPU info on macOS
pub fn get_gpu_info() -> String {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let mut gpu_info = Vec::new();
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Chipset Model:")
                    || trimmed.starts_with("VRAM")
                    || trimmed.starts_with("Metal")
                {
                    gpu_info.push(trimmed.to_string());
                }
            }
            if gpu_info.is_empty() {
                "GPU info not available".to_string()
            } else {
                gpu_info.join("\n")
            }
        }
        _ => "GPU info not available".to_string(),
    }
}

/// OS details struct
pub struct OsDetails {
    pub name: String,
    pub version: String,
    pub build: String,
    pub edition: String,
}

/// Get OS details on macOS
pub fn get_os_details() -> OsDetails {
    let mut name = String::from("macOS");
    let mut version = String::new();
    let mut build = String::new();
    let edition = String::new();

    if let Ok(output) = Command::new("sw_vers").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(val) = line.strip_prefix("ProductName:") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("ProductVersion:") {
                version = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("BuildVersion:") {
                build = val.trim().to_string();
            }
        }
    }

    OsDetails {
        name,
        version,
        build,
        edition,
    }
}

/// Battery info struct
pub struct BatteryInfo {
    pub percentage: Option<u8>,
    pub status: String,
    pub time_remaining: Option<String>,
}

/// Get battery info on macOS
pub fn get_battery() -> BatteryInfo {
    let output = Command::new("pmset")
        .args(["-g", "batt"])
        .output();

    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout).to_string();

        let mut percentage = None;
        let mut status = "Unknown".to_string();
        let mut time_remaining = None;

        for line in text.lines() {
            if line.contains('%') {
                // Parse percentage
                if let Some(pct_str) = line.split('\t').nth(1) {
                    let parts: Vec<&str> = pct_str.split(';').collect();
                    if let Some(pct) = parts.first() {
                        percentage = pct.trim().trim_end_matches('%').parse().ok();
                    }
                    if let Some(st) = parts.get(1) {
                        status = st.trim().to_string();
                    }
                    if let Some(time) = parts.get(2) {
                        let t = time.trim().to_string();
                        if !t.contains("not charging") {
                            time_remaining = Some(t);
                        }
                    }
                }
            }
        }

        BatteryInfo {
            percentage,
            status,
            time_remaining,
        }
    } else {
        BatteryInfo {
            percentage: None,
            status: "No battery detected.".to_string(),
            time_remaining: None,
        }
    }
}

/// User info struct
pub struct UserInfo {
    pub username: String,
    pub role: String,
    pub last_login: String,
}

/// Get user accounts on macOS
pub fn get_users() -> Vec<UserInfo> {
    let mut users = Vec::new();

    let output = Command::new("dscl")
        .args([".", "list", "/Users"])
        .output();

    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        for line in text.lines() {
            let username = line.trim().to_string();
            // Skip system users (starting with _)
            if username.starts_with('_') || username.is_empty() {
                continue;
            }

            // Check if admin
            let role = Command::new("dscl")
                .args([".", "read", &format!("/Groups/admin"), "GroupMembership"])
                .output()
                .ok()
                .and_then(|o| {
                    let text = String::from_utf8_lossy(&o.stdout).to_string();
                    if text.contains(&username) {
                        Some("admin".to_string())
                    } else {
                        Some("standard".to_string())
                    }
                })
                .unwrap_or_else(|| "standard".to_string());

            let last_login = Command::new("last")
                .args(["-1", &username])
                .output()
                .ok()
                .and_then(|o| {
                    let text = String::from_utf8_lossy(&o.stdout).to_string();
                    text.lines().next().map(|l| l.to_string())
                })
                .unwrap_or_else(|| "Unknown".to_string());

            users.push(UserInfo {
                username,
                role,
                last_login,
            });
        }
    }

    if users.is_empty() {
        if let Ok(output) = Command::new("whoami").output() {
            let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
            users.push(UserInfo {
                username,
                role: "standard".to_string(),
                last_login: "Current session".to_string(),
            });
        }
    }

    users
}

/// Get install date on macOS
pub fn get_install_date() -> String {
    let output = Command::new("system_profiler")
        .arg("SPSoftwareDataType")
        .output();

    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        for line in text.lines() {
            if line.contains("Time since boot:") || line.contains("System Integrity") {
                continue;
            }
            if line.contains("Time") {
                return line.trim().to_string();
            }
        }
    }

    "Unknown".to_string()
}
