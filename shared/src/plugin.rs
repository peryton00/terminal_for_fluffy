use serde::{Deserialize, Serialize};

/// Describes a command exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    pub name: String,
    pub syntax: String,
    pub description: String,
}

/// Response from a plugin command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub output: String,
    pub success: bool,
}

/// Trait that any Fluffy plugin must implement.
/// Plugins can register commands that appear in the terminal's help
/// and can be executed through the same interface.
pub trait FluffyPlugin: Send + Sync {
    /// Plugin name (e.g. "example-plugin")
    fn name(&self) -> &str;

    /// Plugin version (e.g. "0.1.0")
    fn version(&self) -> &str;

    /// List of commands this plugin provides
    fn commands(&self) -> Vec<CommandDef>;

    /// Execute a command by name with the given arguments
    fn execute(&self, cmd: &str, args: &[&str]) -> PluginResponse;
}
