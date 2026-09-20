use std::path::PathBuf;

use crate::app::Loadable;

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewerState {
    pub path: PathBuf,
    pub content: Loadable<String>,
    pub scroll_offset: usize,
    pub visible_rows: usize,
}

impl TextViewerState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            content: Loadable::Loading,
            scroll_offset: 0,
            visible_rows: 1,
        }
    }

    pub fn clamp_scroll(&mut self, line_count: usize) {
        if line_count == 0 {
            self.scroll_offset = 0;
            return;
        }
        let max_offset = line_count.saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }
}
