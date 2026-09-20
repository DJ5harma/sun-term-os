use std::path::PathBuf;

use crate::{
    actions::TextViewerAction,
    app::{
        AppState,
        effects::{Effect, TextViewerEffect},
        text_viewer::TextViewerDialog,
    },
    domain::ApplicationKind,
};

use super::open_application;

pub(crate) fn open_path_effects(state: &mut AppState, path: PathBuf) -> Vec<Effect> {
    let window_id = open_application(state, ApplicationKind::TextViewer);
    state.init_text_viewer(window_id, path.clone());
    vec![Effect::TextViewer(TextViewerEffect::Read(window_id, path))]
}

pub(super) fn reduce(state: &mut AppState, action: TextViewerAction) -> Vec<Effect> {
    let window_id = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::TextViewer)
        .map(|window| window.id);
    match action {
        TextViewerAction::BeginOpenPath => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
            {
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
                let path = PathBuf::from(trimmed);
                view.path = path.clone();
                view.content = crate::app::Loadable::Loading;
                view.scroll_offset = 0;
                view.dialog = TextViewerDialog::None;
                return vec![Effect::TextViewer(TextViewerEffect::Read(id, path))];
            }
        }
        TextViewerAction::Scroll(delta) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                let next = view.scroll_offset as i32 + delta;
                view.scroll_offset = next.max(0) as usize;
            }
        }
        TextViewerAction::PageScroll(pages) => {
            if let Some(id) = window_id
                && let Some(view) = state.text_viewer_mut(id)
                && matches!(view.dialog, TextViewerDialog::None)
            {
                let delta = pages * view.visible_rows.max(1) as i32;
                let next = view.scroll_offset as i32 + delta;
                view.scroll_offset = next.max(0) as usize;
            }
        }
    }
    Vec::new()
}
