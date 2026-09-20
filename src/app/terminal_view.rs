#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalViewState {
    /// Lines scrolled up from the live end (0 = follow output).
    pub scroll_from_bottom: usize,
    pub title: Option<String>,
    pub line_buffer: Vec<String>,
}

impl TerminalViewState {
    pub fn follow_output(&self) -> bool {
        self.scroll_from_bottom == 0
    }

    pub fn update_screen(&mut self, content: &str) {
        self.line_buffer = content.lines().map(str::to_owned).collect();
        if self.line_buffer.is_empty() {
            self.line_buffer.push(String::new());
        }
    }

    pub fn scroll_by(&mut self, delta: i32, visible_rows: usize) {
        let max_scroll = self.line_buffer.len().saturating_sub(visible_rows.max(1));
        if delta < 0 {
            self.scroll_from_bottom = self
                .scroll_from_bottom
                .saturating_add((-delta) as usize)
                .min(max_scroll);
        } else {
            self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(delta as usize);
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_from_bottom = 0;
    }

    pub fn visible_text(&self, visible_rows: usize) -> String {
        let rows = visible_rows.max(1);
        let len = self.line_buffer.len();
        if len == 0 {
            return String::new();
        }
        let end = len.saturating_sub(self.scroll_from_bottom);
        let start = end.saturating_sub(rows);
        self.line_buffer[start..end].join("\n")
    }
}

pub fn parse_terminal_title(bytes: &[u8]) -> Option<String> {
    let chunk = String::from_utf8_lossy(bytes);
    for marker in ["\x1b]0;", "\x1b]2;"] {
        if let Some(start) = chunk.find(marker) {
            let rest = &chunk[start + marker.len()..];
            let end = rest
                .find('\x07')
                .or_else(|| rest.find("\x1b\\"))
                .unwrap_or(rest.len());
            let title = rest[..end].trim();
            if !title.is_empty() {
                return Some(title.to_owned());
            }
        }
    }
    None
}
