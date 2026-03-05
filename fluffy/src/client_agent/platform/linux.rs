use std::process::Command;

/// Lock screen on Linux using loginctl
pub fn lock() -> Result<String, String> {
    let output = Command::new("loginctl")
        .arg("lock-session")
        .output()
        .map_err(|e| format!("Failed to lock screen: {}. Try: loginctl lock-session", e))?;
    if output.status.success() {
        Ok("Screen locked successfully.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to lock screen: {}", stderr.trim()))
    }
}

/// Shutdown on Linux
pub fn shutdown() -> Result<String, String> {
    let output = Command::new("shutdown")
        .args(["-h", "now"])
        .output()
        .map_err(|e| format!("Failed to shutdown: {}", e))?;
    if output.status.success() {
        Ok("Shutdown initiated.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Shutdown failed: {}", stderr.trim()))
    }
}

/// Restart on Linux
pub fn restart() -> Result<String, String> {
    let output = Command::new("shutdown")
        .args(["-r", "now"])
        .output()
        .map_err(|e| format!("Failed to restart: {}", e))?;
    if output.status.success() {
        Ok("Restart initiated.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Restart failed: {}", stderr.trim()))
    }
}

/// Show desktop notification on Linux using notify-send
pub fn notify(msg: &str) -> Result<String, String> {
    let output = Command::new("notify-send")
        .args(["Fluffy Terminal", msg])
        .output()
        .map_err(|e| {
            format!(
                "Failed to show notification: {}. Install with: sudo apt install libnotify-bin",
                e
            )
        })?;
    if output.status.success() {
        Ok(format!("Notification sent: {}", msg))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Notification failed: {}", stderr.trim()))
    }
}

/// Play alert sound on Linux
pub fn alert() -> Result<String, String> {
    // Try paplay first, then aplay, then beep
    let result = Command::new("paplay")
        .arg("/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga")
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            return Ok("Alert sound played.".to_string());
        }
    }

    // Fallback: try aplay with a simple beep
    let result = Command::new("aplay")
        .arg("/usr/share/sounds/alsa/Front_Center.wav")
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            return Ok("Alert sound played.".to_string());
        }
    }

    // Last resort: terminal bell
    print!("\x07");
    Ok("Alert: terminal bell sent (no audio player found).".to_string())
}

/// Take screenshot on Linux
pub fn screenshot() -> Result<String, String> {
    let path = "/tmp/fluffy_screenshot.png";

    // Try scrot first
    let result = Command::new("scrot").arg(path).output();
    if let Ok(output) = result {
        if output.status.success() {
            return Ok(format!("Screenshot saved to {}", path));
        }
    }

    // Try gnome-screenshot
    let result = Command::new("gnome-screenshot")
        .args(["-f", path])
        .output();
    if let Ok(output) = result {
        if output.status.success() {
            return Ok(format!("Screenshot saved to {}", path));
        }
    }

    // Try import (ImageMagick)
    let result = Command::new("import")
        .args(["-window", "root", path])
        .output();
    if let Ok(output) = result {
        if output.status.success() {
            return Ok(format!("Screenshot saved to {}", path));
        }
    }

    Err("Screenshot failed: No screenshot tool found. Install one of: scrot, gnome-screenshot, imagemagick".to_string())
}

/// Read clipboard on Linux using xclip
pub fn clipboard_read() -> Result<String, String> {
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .map_err(|e| {
            format!(
                "Failed to read clipboard: {}. Install with: sudo apt install xclip",
                e
            )
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // Try xsel as fallback
        let output2 = Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
            .map_err(|e| format!("Failed to read clipboard: {}", e))?;
        if output2.status.success() {
            Ok(String::from_utf8_lossy(&output2.stdout).to_string())
        } else {
            Err("Clipboard read failed. Install xclip or xsel.".to_string())
        }
    }
}

/// Get GPU info on Linux
pub fn get_gpu_info() -> String {
    let output = Command::new("lspci").output();
    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        let gpu_lines: Vec<&str> = text
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("vga") || lower.contains("3d") || lower.contains("display")
            })
            .collect();
        if !gpu_lines.is_empty() {
            return gpu_lines.join("\n");
        }
    }
    "GPU info not available".to_string()
}

/// OS details struct
pub struct OsDetails {
    pub name: String,
    pub version: String,
    pub build: String,
    pub edition: String,
}

/// Get OS details on Linux from /etc/os-release
pub fn get_os_details() -> OsDetails {
    let mut name = String::from("Linux");
    let mut version = String::new();
    let mut build = String::new();
    let edition = String::new();

    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                name = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("VERSION=") {
                version = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                build = val.trim_matches('"').to_string();
            }
        }
    }

    if let Ok(output) = Command::new("uname").arg("-r").output() {
        if output.status.success() {
            let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if build.is_empty() {
                build = kernel;
            } else {
                build = format!("{} (kernel: {})", build, kernel);
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

/// Get battery info on Linux
pub fn get_battery() -> BatteryInfo {
    // Try reading from /sys/class/power_supply/BAT0
    let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity"));

    let status_str = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .or_else(|_| std::fs::read_to_string("/sys/class/power_supply/BAT1/status"));

    match (capacity, status_str) {
        (Ok(cap), Ok(stat)) => BatteryInfo {
            percentage: cap.trim().parse().ok(),
            status: stat.trim().to_string(),
            time_remaining: None,
        },
        _ => BatteryInfo {
            percentage: None,
            status: "No battery detected.".to_string(),
            time_remaining: None,
        },
    }
}

/// User info struct
pub struct UserInfo {
    pub username: String,
    pub role: String,
    pub last_login: String,
}

/// Get user accounts on Linux
pub fn get_users() -> Vec<UserInfo> {
    let mut users = Vec::new();

    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 {
                let username = parts[0].to_string();
                let uid: u32 = parts[2].parse().unwrap_or(0);
                let shell = parts[6];

                // Skip system users (uid < 1000) and nologin shells
                if uid >= 1000
                    && !shell.contains("nologin")
                    && !shell.contains("false")
                    && username != "nobody"
                {
                    let role = if uid == 0 {
                        "admin".to_string()
                    } else {
                        "standard".to_string()
                    };

                    // Try to get last login
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
        }
    }

    if users.is_empty() {
        // Fallback: at least get the current user
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

/// Get install date on Linux
pub fn get_install_date() -> String {
    // Try filesystem creation time
    if let Ok(output) = Command::new("stat")
        .args(["-c", "%w", "/"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() && text != "-" {
            return text;
        }
    }

    if let Ok(output) = Command::new("ls")
        .args(["-lt", "/var/log/installer/"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = text.lines().nth(1) {
                return line.to_string();
            }
        }
    }

    "Unknown".to_string()
}
