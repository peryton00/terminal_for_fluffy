use crate::commands::Command;

/// Parse a raw input string into a Command enum.
/// Returns None if the input doesn't match any known command.
pub fn parse_command(input: &str) -> Option<Command> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Split into command and arguments
    let parts: Vec<&str> = input.splitn(2, char::is_whitespace).collect();
    let cmd = parts[0].to_lowercase();
    let args_str = if parts.len() > 1 {
        parts[1].trim()
    } else {
        ""
    };

    match cmd.as_str() {
        "ls" => {
            let path = if args_str.is_empty() {
                None
            } else {
                Some(args_str.to_string())
            };
            Some(Command::Ls { path })
        }
        "pwd" => Some(Command::Pwd),
        "cd" => {
            if args_str.is_empty() {
                None // cd requires a path
            } else {
                Some(Command::Cd {
                    path: args_str.to_string(),
                })
            }
        }
        "cat" => {
            if args_str.is_empty() {
                None
            } else {
                Some(Command::Cat {
                    path: args_str.to_string(),
                })
            }
        }
        "whoami" => Some(Command::Whoami),
        "sysinfo" => Some(Command::Sysinfo),
        "processes" => Some(Command::Processes),
        "kill" => {
            if args_str.is_empty() {
                None
            } else {
                match args_str.parse::<u32>() {
                    Ok(pid) => Some(Command::Kill { pid }),
                    Err(_) => None,
                }
            }
        }
        "disk" => {
            // Accept "disk --info" or just "disk"
            Some(Command::DiskInfo)
        }
        "lock" => Some(Command::Lock),
        "shutdown" => Some(Command::Shutdown),
        "restart" => Some(Command::Restart),
        "notify" => {
            if args_str.is_empty() {
                None
            } else {
                // Strip surrounding quotes if present
                let msg = args_str.trim_matches('"').trim_matches('\'').to_string();
                Some(Command::Notify { message: msg })
            }
        }
        "alert" => Some(Command::Alert),
        "locate" => Some(Command::Locate),
        "netinfo" => Some(Command::NetInfo),
        "users" => Some(Command::Users),
        "screenshot" => Some(Command::Screenshot),
        "clipboard" => Some(Command::Clipboard),
        "battery" => Some(Command::Battery),
        "upload" => {
            if args_str.is_empty() {
                None
            } else {
                Some(Command::Upload {
                    filename: args_str.to_string(),
                    data: Vec::new(), // Data is filled in by the admin before sending
                })
            }
        }
        "download" => {
            if args_str.is_empty() {
                None
            } else {
                Some(Command::Download {
                    path: args_str.to_string(),
                })
            }
        }
        "ping" => Some(Command::Ping),
        "sh" => {
            if args_str.is_empty() {
                None
            } else {
                Some(Command::Shell {
                    command: args_str.to_string(),
                })
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ls() {
        let cmd = parse_command("ls").unwrap();
        assert!(matches!(cmd, Command::Ls { path: None }));

        let cmd = parse_command("ls /tmp").unwrap();
        assert!(matches!(cmd, Command::Ls { path: Some(ref p) } if p == "/tmp"));
    }

    #[test]
    fn test_parse_cd() {
        assert!(parse_command("cd").is_none());
        let cmd = parse_command("cd /home").unwrap();
        assert!(matches!(cmd, Command::Cd { path: ref p } if p == "/home"));
    }

    #[test]
    fn test_parse_kill() {
        assert!(parse_command("kill").is_none());
        assert!(parse_command("kill abc").is_none());
        let cmd = parse_command("kill 1234").unwrap();
        assert!(matches!(cmd, Command::Kill { pid: 1234 }));
    }

    #[test]
    fn test_parse_notify() {
        let cmd = parse_command(r#"notify "hello world""#).unwrap();
        assert!(matches!(cmd, Command::Notify { message: ref m } if m == "hello world"));
    }

    #[test]
    fn test_parse_shell() {
        let cmd = parse_command("sh echo hello").unwrap();
        assert!(matches!(cmd, Command::Shell { command: ref c } if c == "echo hello"));
    }

    #[test]
    fn test_parse_unknown() {
        assert!(parse_command("unknown_cmd").is_none());
    }
}
