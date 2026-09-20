//! Layered keyboard dispatch: global bindings first, then modal / app layers.

use crossterm::event::KeyEvent;

use crate::{actions::Action, domain::ApplicationKind};

use super::{
    bindings::{match_global, match_modal, match_window_pick},
    keybindings::{
        desktop_key, file_manager_delete_confirm_key, file_manager_dialog_input_key,
        file_manager_key, process_filter_key, process_manager_key,
    },
    normalize::normalize_key_event,
    terminal_encode::encode_key,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusContext {
    Launcher,
    Window(ApplicationKind),
    Chrome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManagerDialogMode {
    None,
    DeleteConfirm,
    Rename,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInputContext {
    pub focus: FocusContext,
    pub window_pick_mode: bool,
    pub process_filter_active: bool,
    pub file_manager_dialog: FileManagerDialogMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyDispatch {
    Action(Action),
    Terminal(Vec<u8>),
    /// Key ignored without forwarding to a terminal.
    Consumed,
}

pub fn dispatch_key(key: KeyEvent, context: &KeyInputContext) -> KeyDispatch {
    if !key.is_press() && !key.is_repeat() {
        return KeyDispatch::Consumed;
    }

    let normalized = normalize_key_event(key);

    if context.window_pick_mode {
        return match match_window_pick(normalized) {
            Some(action) => KeyDispatch::Action(action),
            None => KeyDispatch::Consumed,
        };
    }

    if let Some(action) = match_global(normalized) {
        return KeyDispatch::Action(action);
    }

    match context.focus {
        FocusContext::Launcher => match match_modal(normalized) {
            Some(action) => KeyDispatch::Action(action),
            None => KeyDispatch::Consumed,
        },
        FocusContext::Window(ApplicationKind::FileManager) => {
            let dispatch = match context.file_manager_dialog {
                FileManagerDialogMode::DeleteConfirm => file_manager_delete_confirm_key(normalized),
                FileManagerDialogMode::Rename | FileManagerDialogMode::Create => {
                    file_manager_dialog_input_key(normalized)
                }
                FileManagerDialogMode::None => file_manager_key(normalized),
            };
            match dispatch {
                Some(action) => KeyDispatch::Action(action),
                None => KeyDispatch::Consumed,
            }
        }
        FocusContext::Window(ApplicationKind::Terminal) => {
            if let Some(bytes) = encode_key(normalized) {
                KeyDispatch::Terminal(bytes)
            } else {
                KeyDispatch::Consumed
            }
        }
        FocusContext::Window(ApplicationKind::Processes) => {
            if context.process_filter_active {
                match process_filter_key(normalized) {
                    Some(action) => KeyDispatch::Action(action),
                    None => KeyDispatch::Consumed,
                }
            } else {
                match process_manager_key(normalized) {
                    Some(action) => KeyDispatch::Action(action),
                    None => KeyDispatch::Consumed,
                }
            }
        }
        FocusContext::Window(ApplicationKind::SystemInfo) | FocusContext::Chrome => {
            match desktop_key(normalized) {
                Some(action) => KeyDispatch::Action(action),
                None => KeyDispatch::Consumed,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn fm_ctx() -> KeyInputContext {
        KeyInputContext {
            focus: FocusContext::Window(ApplicationKind::FileManager),
            window_pick_mode: false,
            process_filter_active: false,
            file_manager_dialog: FileManagerDialogMode::None,
        }
    }

    fn terminal_ctx() -> KeyInputContext {
        KeyInputContext {
            focus: FocusContext::Window(ApplicationKind::Terminal),
            window_pick_mode: false,
            process_filter_active: false,
            file_manager_dialog: FileManagerDialogMode::None,
        }
    }

    fn pick_ctx() -> KeyInputContext {
        KeyInputContext {
            focus: FocusContext::Window(ApplicationKind::Terminal),
            window_pick_mode: true,
            process_filter_active: false,
            file_manager_dialog: FileManagerDialogMode::None,
        }
    }

    #[test]
    fn ctrl_g_starts_pick_from_terminal() {
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(
            dispatch_key(key, &terminal_ctx()),
            KeyDispatch::Action(Action::BeginWindowPick)
        );
    }

    #[test]
    fn digit_in_pick_mode_focuses_from_file_manager() {
        let key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &pick_ctx()),
            KeyDispatch::Action(Action::FocusWindowSlot(2))
        );
    }

    #[test]
    fn bare_digit_does_not_focus_without_pick() {
        let key = KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE);
        assert_eq!(dispatch_key(key, &fm_ctx()), KeyDispatch::Consumed);
    }

    #[test]
    fn file_manager_plain_tab_toggles_pane() {
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &fm_ctx()),
            KeyDispatch::Action(Action::FileManagerTogglePane)
        );
    }

    #[test]
    fn shift_r_begins_rename_in_file_manager() {
        let key = KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT);
        assert_eq!(
            dispatch_key(key, &fm_ctx()),
            KeyDispatch::Action(Action::FileManagerBeginRename)
        );
    }

    #[test]
    fn enter_in_rename_dialog_commits() {
        let ctx = KeyInputContext {
            focus: FocusContext::Window(ApplicationKind::FileManager),
            window_pick_mode: false,
            process_filter_active: false,
            file_manager_dialog: FileManagerDialogMode::Rename,
        };
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &ctx),
            KeyDispatch::Action(Action::FileManagerDialogCommit)
        );
    }
}
