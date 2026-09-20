use crate::{
    actions::{Action, PaletteAction},
    app::{
        AppState, ApplicationKind,
        palette::{clamp_palette_selection, filtered_entries, shell_command_from_query},
    },
};

use super::{Effect, shell};

pub(super) fn reduce(state: &mut AppState, action: PaletteAction) -> Vec<Effect> {
    match action {
        PaletteAction::ToggleLauncher => {
            state.launcher_open = !state.launcher_open;
            state.launcher_selection = 0;
            state.launcher_scroll_offset = 0;
            if state.launcher_open {
                state.launcher_query.clear();
            }
        }
        PaletteAction::CloseLauncher => {
            state.launcher_open = false;
            state.launcher_query.clear();
            state.launcher_selection = 0;
            state.launcher_scroll_offset = 0;
        }
        PaletteAction::PaletteQueryPush(character) => {
            if state.launcher_open {
                state.launcher_query.push(character);
                state.launcher_selection = 0;
                state.launcher_scroll_offset = 0;
            }
        }
        PaletteAction::PaletteQueryBackspace => {
            if state.launcher_open {
                state.launcher_query.pop();
                state.launcher_selection = 0;
                state.launcher_scroll_offset = 0;
            }
        }
        PaletteAction::MoveLauncherUp => {
            if state.launcher_open {
                let count = filtered_entries(state).len();
                if count > 0 && state.launcher_selection > 0 {
                    state.launcher_selection -= 1;
                }
                clamp_palette_selection(
                    &mut state.launcher_selection,
                    &mut state.launcher_scroll_offset,
                    state.launcher_visible_rows.max(1),
                    count,
                );
            }
        }
        PaletteAction::MoveLauncherDown => {
            if state.launcher_open {
                let count = filtered_entries(state).len();
                if count > 0 {
                    state.launcher_selection = (state.launcher_selection + 1).min(count - 1);
                }
                clamp_palette_selection(
                    &mut state.launcher_selection,
                    &mut state.launcher_scroll_offset,
                    state.launcher_visible_rows.max(1),
                    count,
                );
            }
        }
        PaletteAction::ExecuteLauncherSelection => {
            if !state.launcher_open {
                return Vec::new();
            }
            let action = filtered_entries(state)
                .get(state.launcher_selection)
                .map(|entry| entry.action.clone());
            if action == Some(Action::Palette(PaletteAction::RunPaletteShell)) {
                return run_palette_shell(state);
            }
            state.launcher_open = false;
            state.launcher_query.clear();
            state.launcher_selection = 0;
            state.launcher_scroll_offset = 0;
            if let Some(action) = action {
                return super::reduce(state, action);
            }
        }
        PaletteAction::RunPaletteShell => return run_palette_shell(state),
    }
    Vec::new()
}

fn run_palette_shell(state: &mut AppState) -> Vec<Effect> {
    let Some(command) = shell_command_from_query(&state.launcher_query) else {
        state.status = "Enter !command in the palette to run a shell line".to_owned();
        return Vec::new();
    };
    state.launcher_open = false;
    state.launcher_query.clear();
    state.launcher_selection = 0;
    state.launcher_scroll_offset = 0;
    let terminal_id = shell::open_application(state, ApplicationKind::Terminal);
    state.status = format!("Running: {command}");
    vec![
        Effect::Terminal(crate::app::effects::TerminalEffect::Start(terminal_id)),
        Effect::Terminal(crate::app::effects::TerminalEffect::Write(
            terminal_id,
            format!("{command}\n").into_bytes(),
        )),
    ]
}
