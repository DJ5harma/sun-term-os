use crate::app::Loadable;
use crate::machine::applications::ApplicationEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct LauncherState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_rows: usize,
    pub filter: String,
    pub filter_active: bool,
    pub listing: Loadable<Vec<ApplicationEntry>>,
}

impl LauncherState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            visible_rows: 1,
            filter: String::new(),
            filter_active: false,
            listing: Loadable::Loading,
        }
    }

    pub fn clamp_selection(&mut self, count: usize) {
        if count == 0 {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }
        self.selected_index = self.selected_index.min(count - 1);
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        if self.selected_index >= self.scroll_offset + self.visible_rows.max(1) {
            self.scroll_offset = self
                .selected_index
                .saturating_sub(self.visible_rows.saturating_sub(1));
        }
    }
}
