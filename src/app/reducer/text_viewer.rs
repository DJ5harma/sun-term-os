use std::path::PathBuf;

use crate::{
    actions::TextViewerAction,
    app::{
        AppState, Loadable,
        effects::{Effect, TextViewerEffect},
        text_viewer::TextViewerDialog,
    },
    domain::ApplicationKind,
};

use super::{close_window, open_application};

pub(crate) fn open_path_effects(state: &mut AppState, path: PathBuf) -> Vec<Effect> {
    let window_id = open_application(state, ApplicationKind::TextViewer);
    state.init_text_viewer(window_id, path.clone());
    vec![Effect::TextViewer(TextViewerEffect::Read(window_id, path))]
}

fn focused_text_viewer(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::TextViewer)
        .map(|window| window.id)
}

pub(super) fn reduce(state: &mut AppState, action: TextViewerAction) -> Vec<Effect> {
    let window_id = focused_text_viewer(state);
    match action {
        TextViewerAction::BeginOpenPath => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
            {
                if view.is_dirty() {
                    state.status =
                        "Notepad — save changes (Ctrl+S) before opening another file".to_owned();
                    return Vec::new();
                }
                let input = if view.has_path() {
                    view.path.display().to_string()
                } else {
                    String::new()
                };
                view.dialog = TextViewerDialog::OpenPath { input };
                state.status = "Open file · type path · Enter · Esc cancel".to_owned();
            }
        }
        TextViewerAction::DialogPush(ch) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && let TextViewerDialog::OpenPath { input } = &mut view.dialog
            {
                input.push(ch);
            }
        }
        TextViewerAction::DialogBackspace => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && let TextViewerDialog::OpenPath { input } = &mut view.dialog
            {
                input.pop();
            }
        }
        TextViewerAction::DialogCancel => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
            {
                view.dialog = TextViewerDialog::None;
                state.status = "Cancelled".to_owned();
            }
        }
        TextViewerAction::DiscardAndClose => {
            if let Some(id) = window_id {
                return close_window(state, id);
            }
        }
        TextViewerAction::DialogCommit => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && let TextViewerDialog::OpenPath { input } = view.dialog.clone()
            {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    state.status = "Enter a file path".to_owned();
                    return Vec::new();
                }
                if view.is_dirty() {
                    state.status =
                        "Notepad — save changes (Ctrl+S) before opening another file".to_owned();
                    return Vec::new();
                }
                let path = PathBuf::from(trimmed);
                view.path = path.clone();
                view.load = Loadable::Loading;
                view.buffer.clear();
                view.scroll_offset = 0;
                view.dialog = TextViewerDialog::None;
                return vec![Effect::TextViewer(TextViewerEffect::Read(id, path))];
            }
        }
        TextViewerAction::CursorUp => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_up();
            }
        }
        TextViewerAction::CursorDown => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_down();
            }
        }
        TextViewerAction::CursorLeft => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_left();
            }
        }
        TextViewerAction::CursorRight => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_right();
            }
        }
        TextViewerAction::CursorHome => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_home();
            }
        }
        TextViewerAction::CursorEnd => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.move_end();
            }
        }
        TextViewerAction::PageScroll(pages) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.page_cursor(pages);
            }
        }
        TextViewerAction::ScrollView(lines) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.scroll_view(lines);
            }
        }
        TextViewerAction::InsertChar(ch) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.insert_char(ch);
            }
        }
        TextViewerAction::InsertTab => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                for _ in 0..4 {
                    view.insert_char(' ');
                }
            }
        }
        TextViewerAction::Backspace => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.backspace();
            }
        }
        TextViewerAction::Delete => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.delete_forward();
            }
        }
        TextViewerAction::PlaceCaret { line, col } => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                view.place_caret(line, col);
            }
        }
        TextViewerAction::Save => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer(id)
            {
                if !view.is_editable() {
                    state.status = "Nothing to save yet".to_owned();
                    return Vec::new();
                }
                if !view.has_path() {
                    state.status = "No file path — use : to open or save a path first".to_owned();
                    return Vec::new();
                }
                if !view.is_dirty() {
                    state.status = "No changes to save".to_owned();
                    return Vec::new();
                }
                let path = view.path.clone();
                let contents = view.buffer.clone();
                state.status = format!("Saving {}…", path.display());
                return vec![Effect::TextViewer(TextViewerEffect::Write(
                    id, path, contents,
                ))];
            }
        }
    }
    Vec::new()
}
