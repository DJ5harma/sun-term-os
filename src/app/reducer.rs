use crate::{
    actions::Action,
    app::{AppState, ApplicationKind, Window, WindowState},
};

use super::effects::Effect;

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Quit => state.should_quit = true,
        Action::Refresh => {
            state.system = super::Loadable::Loading;
            state.processes = super::Loadable::Loading;
            state.status = "Refreshing local capabilities…".to_owned();
            return vec![Effect::RefreshCapabilities];
        }
        Action::ToggleLauncher => {
            state.launcher_open = !state.launcher_open;
            state.launcher_selection = 0;
        }
        Action::CloseLauncher => state.launcher_open = false,
        Action::MoveLauncherUp => {
            state.launcher_selection = state.launcher_selection.saturating_sub(1)
        }
        Action::MoveLauncherDown => {
            state.launcher_selection =
                (state.launcher_selection + 1).min(ApplicationKind::ALL.len() - 1)
        }
        Action::ExecuteLauncherSelection => {
            state.launcher_open = false;
            if let Some(application) = ApplicationKind::ALL.get(state.launcher_selection) {
                let window_id = open_application(state, *application);
                return terminal_effect(*application, window_id);
            }
        }
        Action::OpenApplication(application) => {
            let window_id = open_application(state, application);
            return terminal_effect(application, window_id);
        }
        Action::CloseWindow => {
            if let Some((window_id, application)) = close_focused_window(state)
                && application == ApplicationKind::Terminal
            {
                return vec![Effect::StopTerminal(window_id)];
            }
        }
        Action::FocusNextWindow => focus_window_by_offset(state, 1),
        Action::FocusPreviousWindow => focus_window_by_offset(state, -1),
        Action::FocusWindow(id) => focus_window(state, id),
        Action::MinimizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = WindowState::Minimized;
            }
            focus_window_by_offset(state, 1);
        }
        Action::ToggleMaximizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = match window.state {
                    WindowState::Maximized => WindowState::Normal,
                    _ => WindowState::Maximized,
                };
            }
        }
        Action::SwitchWorkspace(index) if index < state.workspaces.len() => {
            state.active_workspace = index;
            state.status = format!("Workspace {}", index + 1);
        }
        Action::SwitchWorkspace(_) => state.status = "Workspace does not exist".to_owned(),
    }
    Vec::new()
}

fn terminal_effect(application: ApplicationKind, window_id: u64) -> Vec<Effect> {
    if application == ApplicationKind::Terminal {
        vec![Effect::StartTerminal(window_id)]
    } else {
        Vec::new()
    }
}

fn open_application(state: &mut AppState, application: ApplicationKind) -> u64 {
    let id = state.next_window_id;
    state.next_window_id += 1;
    state.current_workspace_mut().windows.push(Window {
        id,
        application,
        state: WindowState::Normal,
    });
    state.current_workspace_mut().focused_window = Some(id);
    state.status = format!("Opened {}", application.title());
    id
}

fn close_focused_window(state: &mut AppState) -> Option<(u64, ApplicationKind)> {
    let focused = state.current_workspace().focused_window?;
    let workspace = state.current_workspace_mut();
    let application = workspace
        .windows
        .iter()
        .find(|window| window.id == focused)?
        .application;
    workspace.windows.retain(|window| window.id != focused);
    workspace.focused_window = workspace
        .windows
        .iter()
        .rev()
        .find(|window| window.state != WindowState::Minimized)
        .map(|window| window.id);
    state.status = "Window closed".to_owned();
    Some((focused, application))
}

fn focus_window(state: &mut AppState, id: u64) {
    if state
        .current_workspace()
        .windows
        .iter()
        .any(|window| window.id == id && window.state != WindowState::Minimized)
    {
        state.current_workspace_mut().focused_window = Some(id);
    }
}

fn focus_window_by_offset(state: &mut AppState, offset: isize) {
    let visible = state.visible_window_ids();
    if visible.is_empty() {
        state.current_workspace_mut().focused_window = None;
        return;
    }
    let current = state
        .current_workspace()
        .focused_window
        .and_then(|id| visible.iter().position(|item| *item == id))
        .unwrap_or(0);
    let next = (current as isize + offset).rem_euclid(visible.len() as isize) as usize;
    state.current_workspace_mut().focused_window = Some(visible[next]);
}

#[cfg(test)]
mod tests {
    use crate::app::Loadable;

    use super::*;

    #[test]
    fn opening_and_closing_a_window_updates_workspace_state() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::Terminal),
        );

        assert_eq!(state.current_workspace().windows.len(), 1);
        assert_eq!(
            state.focused_window().map(|window| window.application),
            Some(ApplicationKind::Terminal)
        );

        reduce(&mut state, Action::CloseWindow);
        assert!(state.current_workspace().windows.is_empty());
        assert_eq!(state.current_workspace().focused_window, None);
    }

    #[test]
    fn workspaces_keep_independent_windows() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::Processes),
        );
        reduce(&mut state, Action::SwitchWorkspace(1));

        assert!(state.current_workspace().windows.is_empty());

        reduce(&mut state, Action::SwitchWorkspace(0));
        assert_eq!(state.current_workspace().windows.len(), 1);
    }

    #[test]
    fn refresh_returns_an_effect_and_enters_loading_state() {
        let mut state = AppState::default();
        let effects = reduce(&mut state, Action::Refresh);

        assert_eq!(state.status, "Refreshing local capabilities…");
        assert_eq!(state.system, Loadable::Loading);
        assert_eq!(effects, vec![Effect::RefreshCapabilities]);
    }
}
