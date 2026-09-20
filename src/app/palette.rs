use super::AppState;
use crate::actions::{Action, PaletteAction, ShellAction};
use crate::apps;

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteEntry {
    pub title: String,
    pub detail: String,
    pub action: Action,
}

pub fn all_entries(state: &AppState) -> Vec<PaletteEntry> {
    let mut entries = Vec::new();

    for app in apps::all() {
        entries.push(apps::palette_entry(app));
    }

    entries.push(PaletteEntry {
        title: "Refresh".to_owned(),
        detail: "Reload system and process data".to_owned(),
        action: Action::Shell(ShellAction::Refresh),
    });

    for index in 0..state.workspaces.len() {
        let slot = index + 1;
        entries.push(PaletteEntry {
            title: format!("Workspace {slot}"),
            detail: format!("Switch to workspace {slot} (F{slot})"),
            action: Action::Shell(ShellAction::SwitchWorkspace(index)),
        });
    }

    let windows = &state.current_workspace().windows;
    for (index, window) in windows.iter().enumerate().take(9) {
        let slot = index + 1;
        entries.push(PaletteEntry {
            title: format!("Focus window {slot}: {}", apps::title(window.application)),
            detail: format!("Bottom bar slot {slot}"),
            action: Action::Shell(ShellAction::FocusWindowSlot(slot as u8)),
        });
    }

    if state.focused_window().is_some() {
        entries.push(PaletteEntry {
            title: "Close window".to_owned(),
            detail: "Close the focused window (Ctrl+W)".to_owned(),
            action: Action::Shell(ShellAction::CloseWindow),
        });
        entries.push(PaletteEntry {
            title: "Minimize window".to_owned(),
            detail: "Minimize the focused window (Ctrl+M)".to_owned(),
            action: Action::Shell(ShellAction::MinimizeWindow),
        });
        entries.push(PaletteEntry {
            title: "Toggle maximize".to_owned(),
            detail: "Maximize or restore the focused window (Ctrl+F)".to_owned(),
            action: Action::Shell(ShellAction::ToggleMaximizeWindow),
        });
    }

    entries.extend(apps::palette_extras(state));

    entries.push(PaletteEntry {
        title: "Quit sun-term-os".to_owned(),
        detail: "Exit the desktop".to_owned(),
        action: Action::Shell(ShellAction::Quit),
    });

    entries
}

/// Match query tokens as substrings in title + detail (case insensitive).
pub fn matches_query(query: &str, title: &str, detail: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let haystack = format!("{title} {detail}").to_lowercase();
    query
        .split_whitespace()
        .all(|token| haystack.contains(token))
}

/// Text after a leading `!` is run in a new terminal when the palette executes the entry.
pub fn shell_command_from_query(query: &str) -> Option<String> {
    let trimmed = query.trim();
    if !trimmed.starts_with('!') {
        return None;
    }
    let command = trimmed[1..].trim();
    if command.is_empty() {
        None
    } else {
        Some(command.to_owned())
    }
}

pub fn filtered_entries(state: &AppState) -> Vec<PaletteEntry> {
    if let Some(command) = shell_command_from_query(&state.launcher_query) {
        return vec![PaletteEntry {
            title: format!("Run: {command}"),
            detail: "New terminal · !command palette prefix".to_owned(),
            action: Action::Palette(PaletteAction::RunPaletteShell),
        }];
    }
    all_entries(state)
        .into_iter()
        .filter(|entry| matches_query(&state.launcher_query, &entry.title, &entry.detail))
        .collect()
}

pub fn clamp_palette_selection(
    selection: &mut usize,
    scroll_offset: &mut usize,
    visible_rows: usize,
    count: usize,
) {
    if count == 0 {
        *selection = 0;
        *scroll_offset = 0;
        return;
    }
    *selection = (*selection).min(count - 1);
    if visible_rows == 0 {
        return;
    }
    if *selection < *scroll_offset {
        *scroll_offset = *selection;
    }
    if *selection >= *scroll_offset + visible_rows {
        *scroll_offset = *selection + 1 - visible_rows;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matches_tokens_in_any_order() {
        assert!(matches_query("term run", "Terminal", "Run commands"));
        assert!(matches_query(
            "proc view",
            "Process Manager",
            "View running processes"
        ));
        assert!(!matches_query("zzz", "Terminal", "Run commands"));
    }

    #[test]
    fn empty_query_lists_everything() {
        let state = AppState::default();
        assert!(filtered_entries(&state).len() >= apps::all().len());
    }

    #[test]
    fn bang_prefix_runs_shell_only_entry() {
        let mut state = AppState::default();
        state.launcher_query = "!echo hello".to_owned();
        let entries = filtered_entries(&state);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].action,
            Action::Palette(PaletteAction::RunPaletteShell)
        );
    }
}
