#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_helpers {
    use crossterm::event::KeyEvent;

    use crate::{actions::Action, app::AppState};

    pub fn action_for_key(
        key: KeyEvent,
        launcher_open: bool,
        terminal_focused: bool,
        file_manager_focused: bool,
    ) -> Option<Action> {
        use super::super::router::{KeyDispatch, dispatch_key};
        use crate::actions::ShellAction;
        use crate::app::ApplicationKind;
        use crate::app::reducer::reduce;

        let mut state = AppState::default();
        state.launcher_open = launcher_open;
        if terminal_focused {
            reduce(
                &mut state,
                Action::Shell(ShellAction::OpenApplication(ApplicationKind::Terminal)),
            );
        } else if file_manager_focused {
            reduce(
                &mut state,
                Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
            );
        }
        match dispatch_key(key, &state) {
            KeyDispatch::Action(action) => Some(action),
            KeyDispatch::Terminal(_) | KeyDispatch::Consumed => None,
        }
    }

    pub fn terminal_input(key: KeyEvent) -> Option<Vec<u8>> {
        crate::input::terminal_encode::encode_key(key)
    }
}
