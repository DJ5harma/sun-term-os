use crate::{actions::TerminalAction, app::AppState};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: TerminalAction) -> Vec<Effect> {
    match action {
        TerminalAction::ScrollOutput(delta) => {
            if let Some(window) = state.focused_window() {
                let window_id = window.id;
                let visible_rows = state.terminal_body_rows.max(1);
                state
                    .terminal_view_mut(window_id)
                    .scroll_by(delta, visible_rows);
            }
        }
        TerminalAction::ScrollToEnd => {
            if let Some(window) = state.focused_window() {
                state.terminal_view_mut(window.id).reset_scroll();
            }
        }
    }
    Vec::new()
}
