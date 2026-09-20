//! Root pointer pipeline: geometry zone → shell chrome / interaction map → actions. Never forwards to the PTY.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::{
    actions::{Action, PaletteAction},
    app::AppState,
    domain::ApplicationKind,
    ui::{
        geometry::{UiGeometry, shell_action_at},
        interaction::InteractionMap,
    },
};

use super::mouse_click::DoubleClickState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerZone {
    TopBar,
    BottomBar,
    Desktop,
    Launcher,
    Outside,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerDispatch {
    pub actions: Vec<Action>,
}

pub struct PointerContext {
    pub launcher_open: bool,
    pub focused_app: Option<ApplicationKind>,
    pub home_screen_active: bool,
}

impl PointerContext {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            launcher_open: state.launcher_open,
            focused_app: state
                .visible_focus_window()
                .map(|window| window.application),
            home_screen_active: state.shows_home_screen(),
        }
    }
}

pub fn dispatch_pointer(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
    map: &InteractionMap,
    double_click: &mut DoubleClickState,
) -> PointerDispatch {
    let context = PointerContext::from_state(state);
    let position = Position::new(mouse.column, mouse.row);
    let zone = zone_at(position, &context, geometry);

    if let Some(actions) = scroll_actions(mouse.kind, zone, &context) {
        return PointerDispatch { actions };
    }

    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return PointerDispatch {
            actions: Vec::new(),
        };
    }

    if matches!(zone, PointerZone::TopBar | PointerZone::BottomBar)
        && let Some(primary) = shell_action_at(mouse.column, mouse.row, geometry, state)
    {
        let closes_launcher =
            state.launcher_open && primary != Action::Palette(PaletteAction::ToggleLauncher);
        let mut actions = double_click.actions_after_primary(&mouse, primary);
        if closes_launcher {
            actions.insert(0, Action::Palette(PaletteAction::CloseLauncher));
        }
        return PointerDispatch { actions };
    }

    if context.launcher_open && zone != PointerZone::Launcher {
        return PointerDispatch {
            actions: vec![Action::Palette(PaletteAction::CloseLauncher)],
        };
    }

    if let Some(primary) = map.resolve(mouse.column, mouse.row) {
        let mut actions = double_click.actions_after_primary(&mouse, primary);
        if state.launcher_open {
            actions.insert(0, Action::Palette(PaletteAction::CloseLauncher));
        }
        return PointerDispatch { actions };
    }

    PointerDispatch {
        actions: Vec::new(),
    }
}

fn zone_at(position: Position, context: &PointerContext, geometry: &UiGeometry) -> PointerZone {
    if context.launcher_open && geometry.launcher.contains(position) {
        return PointerZone::Launcher;
    }
    if geometry.top_bar.area.contains(position) {
        return PointerZone::TopBar;
    }
    if geometry.bottom_bar.area.contains(position) {
        return PointerZone::BottomBar;
    }
    if geometry.desktop.contains(position) {
        return PointerZone::Desktop;
    }
    PointerZone::Outside
}

fn scroll_actions(
    kind: MouseEventKind,
    zone: PointerZone,
    context: &PointerContext,
) -> Option<Vec<Action>> {
    let delta = match kind {
        MouseEventKind::ScrollUp => -1,
        MouseEventKind::ScrollDown => 1,
        _ => return None,
    };
    match zone {
        PointerZone::Launcher if context.launcher_open => {
            let action = if delta < 0 {
                Action::Palette(PaletteAction::MoveLauncherUp)
            } else {
                Action::Palette(PaletteAction::MoveLauncherDown)
            };
            Some(vec![action])
        }
        PointerZone::Desktop => {
            if context.home_screen_active {
                return Some(vec![crate::apps::home_screen::desktop_scroll_action(delta)]);
            }
            context
                .focused_app
                .and_then(|kind| crate::apps::desktop_scroll_action(kind, delta))
                .map(|action| vec![action])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{PaletteAction, ShellAction};
    use crossterm::event::KeyModifiers;
    use ratatui::layout::Rect;

    #[test]
    fn drag_never_produces_actions() {
        let state = AppState::default();
        let geometry = UiGeometry::default();
        let map = InteractionMap::default();
        let mut double = DoubleClickState::default();
        let mouse = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };
        let dispatch = dispatch_pointer(mouse, &state, &geometry, &map, &mut double);
        assert!(dispatch.actions.is_empty());
    }

    #[test]
    fn bottom_bar_launcher_uses_shell_geometry() {
        let state = AppState::default();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 24), &state);
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: geometry.bottom_bar.launcher.x + 1,
            row: geometry.bottom_bar.launcher.y,
            modifiers: KeyModifiers::NONE,
        };
        let dispatch = dispatch_pointer(
            mouse,
            &state,
            &geometry,
            &InteractionMap::default(),
            &mut DoubleClickState::default(),
        );
        assert_eq!(
            dispatch.actions,
            vec![Action::Palette(PaletteAction::ToggleLauncher)]
        );
    }

    fn state_with_two_windows() -> AppState {
        use crate::domain::{Window, WindowState};
        use crate::machine::MachineId;
        let mut state = AppState::default();
        let workspace = &mut state.workspaces[0];
        workspace.windows.push(Window {
            id: 1,
            application: ApplicationKind::Terminal,
            state: WindowState::Normal,
            machine_id: MachineId::Local,
        });
        workspace.windows.push(Window {
            id: 2,
            application: ApplicationKind::FileManager,
            state: WindowState::Normal,
            machine_id: MachineId::Local,
        });
        workspace.focused_window = Some(1);
        state
    }

    #[test]
    fn bottom_bar_window_tab_uses_shell_geometry() {
        let state = state_with_two_windows();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 24), &state);
        let windows = &state.current_workspace().windows;
        assert!(!windows.is_empty());
        let cells =
            crate::ui::geometry::window_tab_cells(geometry.bottom_bar.windows, windows.len());
        let cell = cells[0];
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: cell.x + cell.width / 2,
            row: cell.y,
            modifiers: KeyModifiers::NONE,
        };
        let dispatch = dispatch_pointer(
            mouse,
            &state,
            &geometry,
            &InteractionMap::default(),
            &mut DoubleClickState::default(),
        );
        assert_eq!(
            dispatch.actions,
            vec![Action::Shell(ShellAction::FocusWindow(windows[0].id))]
        );
    }

    #[test]
    fn launcher_open_shell_click_closes_then_acts() {
        let mut state = state_with_two_windows();
        state.launcher_open = true;
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 24), &state);
        let windows = &state.current_workspace().windows;
        let cell =
            crate::ui::geometry::window_tab_cells(geometry.bottom_bar.windows, windows.len())[0];
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: cell.x + cell.width / 2,
            row: cell.y,
            modifiers: KeyModifiers::NONE,
        };
        let dispatch = dispatch_pointer(
            mouse,
            &state,
            &geometry,
            &InteractionMap::default(),
            &mut DoubleClickState::default(),
        );
        assert_eq!(
            dispatch.actions,
            vec![
                Action::Palette(PaletteAction::CloseLauncher),
                Action::Shell(ShellAction::FocusWindow(windows[0].id))
            ]
        );
    }
}
