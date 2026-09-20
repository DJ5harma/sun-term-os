//! Empty-desktop launcher: catalog ordering and per-workspace selection state.

use crate::apps::{self, BuiltInApp};
use crate::config::Config;
use crate::domain::ApplicationKind;
use crate::domain::WindowState;

use super::AppState;

/// One tile on the home screen (source of truth: built-in app registry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeEntry {
    pub kind: ApplicationKind,
    pub title: String,
    pub description: String,
    pub shortcut: Option<char>,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HomeScreenState {
    pub selected: usize,
    pub scroll_row: usize,
    pub columns: usize,
    pub tile_rows_visible: usize,
}

impl HomeScreenState {
    pub fn clamp_selection(&mut self, entry_count: usize, total_tile_rows: usize) {
        if entry_count == 0 {
            self.selected = 0;
            self.scroll_row = 0;
            return;
        }
        self.selected = self.selected.min(entry_count - 1);
        if total_tile_rows == 0 {
            self.scroll_row = 0;
            return;
        }
        let selected_row = self.selected / self.columns.max(1);
        if selected_row < self.scroll_row {
            self.scroll_row = selected_row;
        }
        if self.tile_rows_visible > 0 && selected_row >= self.scroll_row + self.tile_rows_visible {
            self.scroll_row = selected_row + 1 - self.tile_rows_visible;
        }
        self.scroll_row = self
            .scroll_row
            .min(total_tile_rows.saturating_sub(self.tile_rows_visible.max(1)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeScreenMode {
    EmptyWorkspace,
    WindowsInBackground { total: usize, minimized: usize },
}

pub fn entries(config: &Config) -> Vec<HomeEntry> {
    let pinned = &config.home.pinned;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for kind in pinned {
        if let Some(app) = apps::by_kind(*kind) {
            push_entry(&mut out, &mut seen, app, true);
        }
    }

    let mut rest: Vec<&BuiltInApp> = apps::all()
        .iter()
        .filter(|app| !seen.contains(&app.kind))
        .collect();
    rest.sort_by_key(|app| app.title.to_ascii_lowercase());
    for app in rest {
        push_entry(&mut out, &mut seen, app, false);
    }
    out
}

fn push_entry(
    out: &mut Vec<HomeEntry>,
    seen: &mut std::collections::HashSet<ApplicationKind>,
    app: &BuiltInApp,
    pinned: bool,
) {
    if !seen.insert(app.kind) {
        return;
    }
    out.push(HomeEntry {
        kind: app.kind,
        title: app.title.to_owned(),
        description: app.description.to_owned(),
        shortcut: app.quick_launch_key,
        pinned,
    });
}

impl AppState {
    /// Desktop shows the home grid when no non-minimized window is focused.
    pub fn shows_home_screen(&self) -> bool {
        self.visible_focus_window().is_none()
    }

    pub fn home_screen_mode(&self) -> Option<HomeScreenMode> {
        if !self.shows_home_screen() {
            return None;
        }
        let workspace = self.current_workspace();
        if workspace.windows.is_empty() {
            return Some(HomeScreenMode::EmptyWorkspace);
        }
        let minimized = workspace
            .windows
            .iter()
            .filter(|window| window.state == WindowState::Minimized)
            .count();
        Some(HomeScreenMode::WindowsInBackground {
            total: workspace.windows.len(),
            minimized,
        })
    }

    pub fn visible_focus_window(&self) -> Option<&crate::domain::Window> {
        self.focused_window()
            .filter(|window| window.state != WindowState::Minimized)
    }

    pub fn home_screen(&self) -> &HomeScreenState {
        &self.home_screens[self.active_workspace]
    }

    pub(crate) fn home_screen_mut(&mut self) -> &mut HomeScreenState {
        &mut self.home_screens[self.active_workspace]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::ApplicationKind;

    #[test]
    fn pinned_entries_come_first() {
        let config = Config::default();
        let list = entries(&config);
        assert!(!list.is_empty());
        assert_eq!(list[0].kind, ApplicationKind::Terminal);
        assert!(list[0].pinned);
        let settings = list.iter().find(|entry| entry.kind == ApplicationKind::Settings);
        assert!(settings.is_some());
        assert!(!settings.unwrap().pinned);
    }

    #[test]
    fn clamp_scroll_follows_selection() {
        let mut home = HomeScreenState {
            selected: 8,
            scroll_row: 0,
            columns: 3,
            tile_rows_visible: 2,
        };
        home.clamp_selection(9, 3);
        assert_eq!(home.scroll_row, 1);
    }
}
