use crate::{
    actions::ShellAction,
    app::{AppState, ApplicationKind, Window, WindowState},
    apps,
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: ShellAction) -> Vec<Effect> {
    match action {
        ShellAction::Quit => {
            state.should_quit = true;
        }
        ShellAction::Refresh => {
            state.mark_capability_refresh_loading();
            state.status = "Refreshing local capabilities…".to_owned();
            return vec![Effect::RefreshCapabilities];
        }
        ShellAction::OpenApplication(application) => {
            state.launcher_open = false;
            clear_show_desktop(state);
            let window_id = open_application(state, application);
            return open_application_effects(state, application, window_id);
        }
        ShellAction::CloseWindow => {
            if let Some((window_id, application)) = close_focused_window(state) {
                return apps::on_close(application, state, window_id);
            }
        }
        ShellAction::FocusWindowSlot(slot) => {
            focus_window_slot(state, slot);
            state.window_pick_mode = false;
        }
        ShellAction::BeginWindowPick => {
            state.window_pick_mode = true;
            state.status = "Press 1–9 to focus a window · Esc to cancel".to_owned();
        }
        ShellAction::CancelWindowPick => {
            state.window_pick_mode = false;
            state.status = "Window focus cancelled".to_owned();
        }
        ShellAction::FocusWindow(id) => focus_window(state, id),
        ShellAction::MinimizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = WindowState::Minimized;
            }
            focus_window_by_offset(state, 1);
        }
        ShellAction::ToggleMaximizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = match window.state {
                    WindowState::Maximized => WindowState::Normal,
                    _ => WindowState::Maximized,
                };
            }
        }
        ShellAction::SwitchWorkspace(index) if index < state.workspaces.len() => {
            state.active_workspace = index;
            state.status = format!("Workspace {}", index + 1);
        }
        ShellAction::SwitchWorkspace(_) => state.status = "Workspace does not exist".to_owned(),
        ShellAction::ToggleInputDebug => {
            state.input_debug = !state.input_debug;
            state.input_debug_line = if state.input_debug {
                "Input debug on (Ctrl+Alt+D off)".to_owned()
            } else {
                String::new()
            };
        }
        ShellAction::ToggleShowDesktop => toggle_show_desktop(state),
    }
    Vec::new()
}

pub(crate) fn clear_show_desktop(state: &mut AppState) {
    let workspace = state.current_workspace_mut();
    if workspace.show_desktop {
        workspace.show_desktop = false;
        workspace.show_desktop_restore_focus = None;
    }
}

fn toggle_show_desktop(state: &mut AppState) {
    let workspace = state.current_workspace_mut();
    if workspace.show_desktop {
        workspace.show_desktop = false;
        let restore = workspace.show_desktop_restore_focus;
        workspace.show_desktop_restore_focus = None;
        if let Some(id) = restore {
            focus_window(state, id);
        }
        state.status = "Restored windows".to_owned();
        return;
    }
    workspace.show_desktop_restore_focus = workspace.focused_window;
    workspace.show_desktop = true;
    state.status = format!(
        "Show desktop · {} or ⌂ to restore",
        crate::input::SHOW_DESKTOP_HINT
    );
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

pub(super) fn focus_window(state: &mut AppState, id: u64) {
    clear_show_desktop(state);
    let workspace = state.current_workspace_mut();
    if let Some(window) = workspace.windows.iter_mut().find(|window| window.id == id) {
        if window.state == WindowState::Minimized {
            window.state = WindowState::Normal;
        }
        workspace.focused_window = Some(id);
    }
}

fn focus_window_slot(state: &mut AppState, slot: u8) {
    if !(1..=9).contains(&slot) {
        return;
    }
    let windows = state
        .current_workspace()
        .windows
        .iter()
        .map(|window| window.id)
        .collect::<Vec<_>>();
    let index = (slot - 1) as usize;
    if let Some(id) = windows.get(index) {
        focus_window(state, *id);
        state.status = format!("Window {slot}");
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

pub(crate) fn open_application(state: &mut AppState, application: ApplicationKind) -> u64 {
    clear_show_desktop(state);
    let id = state.next_window_id;
    state.next_window_id += 1;
    let machine_id = state.active_machine_id.clone();
    state.current_workspace_mut().windows.push(Window {
        id,
        application,
        state: WindowState::Normal,
        machine_id,
    });
    state.current_workspace_mut().focused_window = Some(id);
    apps::on_open_window(state, application, id);
    state.status = format!("Opened {}", crate::apps::title(application));
    id
}

pub(crate) fn open_application_effects(
    state: &mut AppState,
    application: ApplicationKind,
    window_id: u64,
) -> Vec<Effect> {
    apps::open_effects(application, state, window_id)
}
