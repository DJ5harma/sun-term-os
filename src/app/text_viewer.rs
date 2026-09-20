use std::path::PathBuf;

use crate::app::Loadable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextViewerDialog {
    None,
    OpenPath { input: String },
    ConfirmDiscardClose,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewerState {
    pub path: PathBuf,
    pub load: Loadable<()>,
    pub buffer: String,
    pub saved_buffer: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub visible_rows: usize,
    pub dialog: TextViewerDialog,
}

impl TextViewerState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            load: Loadable::Loading,
            buffer: String::new(),
            saved_buffer: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            visible_rows: 1,
            dialog: TextViewerDialog::None,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            path: PathBuf::new(),
            load: Loadable::Ready(()),
            buffer: String::new(),
            saved_buffer: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            visible_rows: 1,
            dialog: TextViewerDialog::None,
        }
    }

    pub fn has_path(&self) -> bool {
        !self.path.as_os_str().is_empty()
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer != self.saved_buffer
    }

    pub fn is_editable(&self) -> bool {
        matches!(self.load, Loadable::Ready(()))
    }

    pub fn line_count(&self) -> usize {
        line_ranges(&self.buffer).len().max(1)
    }

    pub fn clamp_scroll(&mut self) {
        let max = self.line_count().saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max);
    }

    pub fn ensure_cursor_visible(&mut self) {
        self.clamp_scroll();
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        }
        let last_visible = self
            .scroll_offset
            .saturating_add(self.visible_rows.saturating_sub(1));
        if self.cursor_line > last_visible {
            self.scroll_offset = self
                .cursor_line
                .saturating_sub(self.visible_rows.saturating_sub(1));
        }
        self.clamp_scroll();
    }

    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self
                .cursor_col
                .min(line_char_len(&self.buffer, self.cursor_line));
        }
        self.ensure_cursor_visible();
    }

    pub fn move_down(&mut self) {
        let max_line = self.line_count().saturating_sub(1);
        if self.cursor_line < max_line {
            self.cursor_line += 1;
            self.cursor_col = self
                .cursor_col
                .min(line_char_len(&self.buffer, self.cursor_line));
        }
        self.ensure_cursor_visible();
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            return;
        }
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = line_char_len(&self.buffer, self.cursor_line);
        }
        self.ensure_cursor_visible();
    }

    pub fn move_right(&mut self) {
        let line_len = line_char_len(&self.buffer, self.cursor_line);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
            return;
        }
        if self.cursor_line + 1 < self.line_count() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
        self.ensure_cursor_visible();
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
        self.ensure_cursor_visible();
    }

    pub fn move_end(&mut self) {
        self.cursor_col = line_char_len(&self.buffer, self.cursor_line);
        self.ensure_cursor_visible();
    }

    pub fn page_cursor(&mut self, pages: i32) {
        let step = pages.unsigned_abs() as usize * self.visible_rows.max(1);
        if pages < 0 {
            self.cursor_line = self.cursor_line.saturating_sub(step);
        } else {
            let max_line = self.line_count().saturating_sub(1);
            self.cursor_line = (self.cursor_line + step).min(max_line);
        }
        self.cursor_col = self
            .cursor_col
            .min(line_char_len(&self.buffer, self.cursor_line));
        self.ensure_cursor_visible();
    }

    pub fn scroll_view(&mut self, lines: i32) {
        let next = self.scroll_offset as i32 + lines;
        self.scroll_offset = next.max(0) as usize;
        self.clamp_scroll();
    }

    pub fn place_caret(&mut self, line: usize, col: usize) {
        let max_line = self.line_count().saturating_sub(1);
        self.cursor_line = line.min(max_line);
        self.cursor_col = col.min(line_char_len(&self.buffer, self.cursor_line));
        self.ensure_cursor_visible();
    }

    pub fn insert_char(&mut self, ch: char) {
        if !self.is_editable() {
            return;
        }
        let offset = cursor_byte_offset(&self.buffer, self.cursor_line, self.cursor_col);
        if ch == '\n' {
            self.buffer.insert(offset, '\n');
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.buffer.insert(offset, ch);
            self.cursor_col += 1;
        }
        self.ensure_cursor_visible();
    }

    pub fn backspace(&mut self) {
        if !self.is_editable() {
            return;
        }
        if self.cursor_col > 0 {
            let offset = cursor_byte_offset(&self.buffer, self.cursor_line, self.cursor_col);
            let prev = self.buffer[..offset]
                .char_indices()
                .next_back()
                .map(|(index, _)| index)
                .unwrap_or(0);
            self.buffer.replace_range(prev..offset, "");
            self.cursor_col -= 1;
            self.ensure_cursor_visible();
            return;
        }
        if self.cursor_line == 0 {
            return;
        }
        let prev_line = self.cursor_line - 1;
        let prev_len = line_char_len(&self.buffer, prev_line);
        let line_start = cursor_byte_offset(&self.buffer, self.cursor_line, 0);
        if line_start > 0 && self.buffer.as_bytes()[line_start - 1] == b'\n' {
            self.buffer.remove(line_start - 1);
        }
        self.cursor_line = prev_line;
        self.cursor_col = prev_len;
        self.ensure_cursor_visible();
    }

    pub fn delete_forward(&mut self) {
        if !self.is_editable() {
            return;
        }
        let line_len = line_char_len(&self.buffer, self.cursor_line);
        if self.cursor_col < line_len {
            let start = cursor_byte_offset(&self.buffer, self.cursor_line, self.cursor_col);
            let next = self.buffer[start..]
                .char_indices()
                .nth(1)
                .map(|(index, _)| start + index)
                .unwrap_or(self.buffer.len());
            self.buffer.replace_range(start..next, "");
            self.ensure_cursor_visible();
            return;
        }
        if self.cursor_line + 1 >= self.line_count() {
            return;
        }
        let line_end = line_byte_end(&self.buffer, self.cursor_line);
        if line_end < self.buffer.len() && self.buffer.as_bytes()[line_end] == b'\n' {
            self.buffer.remove(line_end);
        }
        self.ensure_cursor_visible();
    }

    pub fn apply_loaded(&mut self, text: String) {
        self.buffer = text;
        self.saved_buffer = self.buffer.clone();
        self.load = Loadable::Ready(());
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn mark_saved(&mut self) {
        self.saved_buffer = self.buffer.clone();
    }
}

fn line_ranges(text: &str) -> Vec<(usize, usize)> {
    if text.is_empty() {
        return vec![(0, 0)];
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            ranges.push((start, index));
            start = index + 1;
        }
    }
    ranges.push((start, text.len()));
    ranges
}

fn line_char_len(text: &str, line: usize) -> usize {
    let ranges = line_ranges(text);
    let Some(&(start, end)) = ranges.get(line) else {
        return 0;
    };
    text[start..end].chars().count()
}

fn line_byte_end(text: &str, line: usize) -> usize {
    line_ranges(text)
        .get(line)
        .map(|&(_, end)| end)
        .unwrap_or(text.len())
}

fn cursor_byte_offset(text: &str, line: usize, col: usize) -> usize {
    let ranges = line_ranges(text);
    let Some(&(start, end)) = ranges.get(line) else {
        return text.len();
    };
    let slice = &text[start..end];
    let byte_in_line = slice
        .char_indices()
        .nth(col)
        .map(|(index, _)| index)
        .unwrap_or(slice.len());
    start + byte_in_line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_backspace_across_lines() {
        let mut state = TextViewerState::new_empty();
        state.apply_loaded("ab\ncd".to_owned());
        state.place_caret(1, 0);
        state.backspace();
        assert_eq!(state.buffer, "abcd");
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 2);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut state = TextViewerState::new_empty();
        state.apply_loaded("hi".to_owned());
        state.place_caret(0, 1);
        state.insert_char('\n');
        assert_eq!(state.buffer, "h\ni");
        assert_eq!(state.cursor_line, 1);
        assert_eq!(state.cursor_col, 0);
    }
}
