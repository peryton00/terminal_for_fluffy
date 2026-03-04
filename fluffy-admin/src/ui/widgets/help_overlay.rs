use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use fluffy_ui::theme;

/// The help overlay popup.
pub struct HelpOverlayWidget {
    pub scroll: u16,
}

impl Widget for HelpOverlayWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area behind the popup
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::brand_style())
            .title(" Fluffy Terminal — Help (ESC to close) ")
            .title_style(theme::brand_style());

        let help_text = get_help_text();
        let lines: Vec<Line> = help_text
            .lines()
            .map(|l| {
                if l.starts_with("  ─") || l.starts_with("═") {
                    Line::from(Span::styled(l, theme::brand_style()))
                } else if l.starts_with("  ADMIN") || l.starts_with("  CLIENT") || l.starts_with("  PLUGIN") {
                    Line::from(Span::styled(l, theme::brand_style()))
                } else if l.contains("→") {
                    let parts: Vec<&str> = l.splitn(2, '→').collect();
                    Line::from(vec![
                        Span::styled(parts[0], theme::highlight_style()),
                        Span::styled("→", theme::dim_style()),
                        Span::styled(
                            parts.get(1).unwrap_or(&"").to_string(),
                            theme::text_style(),
                        ),
                    ])
                } else {
                    Line::from(Span::styled(l, theme::text_style()))
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        Widget::render(paragraph, area, buf);
    }
}

fn get_help_text() -> &'static str {
    r#"
  ═══════════════════════════════════════
  ADMIN COMMANDS (run locally)
  ─────────────────────────────────────

  rolecall            → List all connected clients
  fluffy --help       → Show this help overlay
  f alter <tag>       → Set target (e.g., f alter f1)
  f alter local       → Clear alter mode (run locally)
  f alter off         → Same as f alter local
  clean               → Clear the output panel
  history             → Show command history
  !!                  → Re-run the last command
  broadcast "msg"     → Send notification to ALL clients
  exit / quit         → Exit fluffy-admin

  ═══════════════════════════════════════
  CLIENT COMMANDS (sent to target)
  ─────────────────────────────────────

  ls [path]           → List files and folders
  pwd                 → Print working directory
  cd <path>           → Change directory
  cat <file>          → Print file contents
  whoami              → Device name + username + role
  sysinfo             → Full system information
  processes           → List all processes (by CPU%)
  kill <pid>          → Kill a process by PID
  disk --info         → RAM + disk usage info
  lock                → Lock the target's screen
  shutdown            → Shutdown the target
  restart             → Restart the target
  notify "msg"        → Desktop notification on target
  alert               → Play alert sound on target
  locate              → Network geolocation (IP-based)
  netinfo             → Network information
  users               → List user accounts
  screenshot          → Capture screen (save as PNG)
  clipboard           → Read clipboard text content
  battery             → Battery status
  upload <file>       → Transfer file TO target
  download <file>     → Transfer file FROM target
  ping                → Latency check (returns PONG)
  sh <command>        → Run raw shell command

  ═══════════════════════════════════════
  TARGETING
  ─────────────────────────────────────

  f1 sysinfo          → Run sysinfo on client f1
  f alter f1          → All commands go to f1
  sysinfo             → (after alter) same as f1 sysinfo
  f alter off         → Back to local mode

  ═══════════════════════════════════════
  SHORTCUTS
  ─────────────────────────────────────

  Ctrl+C              → Exit
  ESC                 → Close help overlay
  ↑/↓                 → Scroll output
  Page Up/Down        → Scroll output (fast)
"#
}
