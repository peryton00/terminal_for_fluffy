use shared::plugin::{CommandDef, FluffyPlugin, PluginResponse};

/// Example plugin demonstrating the FluffyPlugin trait.
pub struct ExamplePlugin;

impl ExamplePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl FluffyPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example-plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn commands(&self) -> Vec<CommandDef> {
        vec![
            CommandDef {
                name: "hello".to_string(),
                syntax: "hello [name]".to_string(),
                description: "Say hello! Optionally provide a name.".to_string(),
            },
            CommandDef {
                name: "echo".to_string(),
                syntax: "echo <message>".to_string(),
                description: "Echo back the given message.".to_string(),
            },
            CommandDef {
                name: "random".to_string(),
                syntax: "random".to_string(),
                description: "Generate a random number between 1 and 100.".to_string(),
            },
        ]
    }

    fn execute(&self, cmd: &str, args: &[&str]) -> PluginResponse {
        match cmd {
            "hello" => {
                let name = if args.is_empty() {
                    "World"
                } else {
                    args[0]
                };
                PluginResponse {
                    output: format!(
                        "🐾 Hello, {}! Welcome to the Fluffy ecosystem!",
                        name
                    ),
                    success: true,
                }
            }
            "echo" => {
                if args.is_empty() {
                    PluginResponse {
                        output: "Usage: echo <message>".to_string(),
                        success: false,
                    }
                } else {
                    PluginResponse {
                        output: args.join(" "),
                        success: true,
                    }
                }
            }
            "random" => {
                // Simple pseudo-random using system time
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                let num = (seed % 100) + 1;
                PluginResponse {
                    output: format!("🎲 Random number: {}", num),
                    success: true,
                }
            }
            _ => PluginResponse {
                output: format!(
                    "Unknown plugin command: '{}'. Available: hello, echo, random",
                    cmd
                ),
                success: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::plugin::FluffyPlugin;

    #[test]
    fn test_hello() {
        let plugin = ExamplePlugin::new();
        let resp = plugin.execute("hello", &[]);
        assert!(resp.success);
        assert!(resp.output.contains("Hello, World"));
    }

    #[test]
    fn test_hello_with_name() {
        let plugin = ExamplePlugin::new();
        let resp = plugin.execute("hello", &["Fluffy"]);
        assert!(resp.success);
        assert!(resp.output.contains("Hello, Fluffy"));
    }

    #[test]
    fn test_echo() {
        let plugin = ExamplePlugin::new();
        let resp = plugin.execute("echo", &["test", "message"]);
        assert!(resp.success);
        assert_eq!(resp.output, "test message");
    }

    #[test]
    fn test_random() {
        let plugin = ExamplePlugin::new();
        let resp = plugin.execute("random", &[]);
        assert!(resp.success);
        assert!(resp.output.contains("Random number"));
    }

    #[test]
    fn test_unknown() {
        let plugin = ExamplePlugin::new();
        let resp = plugin.execute("nonexistent", &[]);
        assert!(!resp.success);
    }

    #[test]
    fn test_commands_list() {
        let plugin = ExamplePlugin::new();
        let cmds = plugin.commands();
        assert_eq!(cmds.len(), 3);
    }
}
