# 🐾 Fluffy Terminal

A cross-platform LAN-based remote administration terminal built in Rust. Control multiple machines on your local network from a single, beautiful TUI interface.

Ready to connect with any software with easy call
## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  fluffy-admin (TUI)                                      │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ Status: FLUFFY | Target: f1 | 3 online | 14:32:00  │ │
│  ├────────────┬────────────────────────────────────────┤ │
│  │  Clients   │  Output                                │ │
│  │  ● f1 pc1  │  [f1] sysinfo result...                │ │
│  │  ● f2 srv  │  [local] ls output...                  │ │
│  ├────────────┴────────────────────────────────────────┤ │
│  │  fluffy [f1]> _                                     │ │
│  └─────────────────────────────────────────────────────┘ │
│              │ TCP :9000                                  │
│              ▼                                           │
│  ┌─────────────────┐  ┌─────────────────┐               │
│  │ fluffy-client f1 │  │ fluffy-client f2 │  ...         │
│  │  (agent on PC)   │  │  (agent on srv)  │               │
│  └─────────────────┘  └─────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

## Project Structure

| Crate                    | Description                                               |
| ------------------------ | --------------------------------------------------------- |
| `shared`                 | Protocol types, command definitions, parser, plugin trait |
| `fluffy-ui`              | Ratatui widget library with consistent theming            |
| `fluffy-admin`           | Admin TUI terminal (the control center)                   |
| `fluffy-client`          | Agent binary installed on controlled machines             |
| `plugins/example-plugin` | Example plugin implementing the `FluffyPlugin` trait      |

## Build

```bash
cd terminal_for_fluffy
cargo build --release
```

Binaries produced:

- `target/release/fluffy-admin` — Run on the admin machine
- `target/release/fluffy-client` — Run on each client machine

## Usage

### Start the admin terminal

```bash
./fluffy-admin
```

### Connect a client (on another machine)

```bash
./fluffy-client 192.168.1.100
```

### Commands

#### Admin Commands (local only)

| Command             | Description                |
| ------------------- | -------------------------- |
| `rolecall`          | List all connected clients |
| `fluffy --help`     | Show help overlay          |
| `f alter f1`        | Target f1 for all commands |
| `f alter local/off` | Return to local mode       |
| `clean`             | Clear the output panel     |
| `history`           | Show command history       |
| `!!`                | Re-run last command        |
| `broadcast "msg"`   | Notify all clients         |
| `exit` / `quit`     | Exit                       |

#### Client Commands

| Command           | Description              |
| ----------------- | ------------------------ |
| `ls [path]`       | List files and folders   |
| `pwd`             | Print working directory  |
| `cd <path>`       | Change directory         |
| `cat <file>`      | Print file contents      |
| `whoami`          | Device + user info       |
| `sysinfo`         | Full system information  |
| `processes`       | List processes (by CPU%) |
| `kill <pid>`      | Kill a process           |
| `disk --info`     | RAM + disk usage         |
| `lock`            | Lock screen              |
| `shutdown`        | Shutdown machine         |
| `restart`         | Restart machine          |
| `notify "msg"`    | Desktop notification     |
| `alert`           | Play alert sound         |
| `locate`          | IP-based geolocation     |
| `netinfo`         | Network information      |
| `users`           | List user accounts       |
| `screenshot`      | Capture screen           |
| `clipboard`       | Read clipboard           |
| `battery`         | Battery status           |
| `upload <file>`   | Transfer TO target       |
| `download <file>` | Transfer FROM target     |
| `ping`            | Latency check            |
| `sh <cmd>`        | Raw shell command        |

### Targeting

```
f1 sysinfo          # Run on client f1
f alter f1           # Set f1 as default target
sysinfo              # Now runs on f1 automatically
f alter off          # Back to local mode
sysinfo              # Runs on admin machine
```

## Cross-Platform Support

Fluffy supports **Windows**, **Linux**, and **macOS** with platform-specific implementations for:

- Screen lock, shutdown, restart
- Desktop notifications and alerts
- Screenshots and clipboard access
- GPU info, battery status, user accounts

## Plugin System

Create custom plugins by implementing the `FluffyPlugin` trait:

```rust
use shared::plugin::{FluffyPlugin, CommandDef, PluginResponse};

pub struct MyPlugin;

impl FluffyPlugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn commands(&self) -> Vec<CommandDef> { vec![/* ... */] }
    fn execute(&self, cmd: &str, args: &[&str]) -> PluginResponse {
        // Your logic here
    }
}
```

## License

MIT
