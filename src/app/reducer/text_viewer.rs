use std::path::PathBuf;

use crate::{
    actions::TextViewerAction,
    app::{AppState, effects::TextViewerEffect},
    domain::ApplicationKind,
};

use super::{Effect, open_application};

pub(crate) fn open_path_effects(state: &mut AppState, path: PathBuf) -> Vec<Effect> {
    let window_id = open_application(state, ApplicationKind::TextViewer);
    state.init_text_viewer(window_id, path.clone());
    vec![Effect::TextViewer(TextViewerEffect::Read(window_id, path))]
}

pub(super) fn reduce(state: &mut AppState, action: TextViewerAction) -> Vec<Effect> {
    if let Some(window) = state.focused_window()
        && let Some(view) = state.text_viewer_mut(window.id)
    {
        match action {
            TextViewerAction::Scroll(delta) => {
                let next = view.scroll_offset as i32 + delta;
                view.scroll_offset = next.max(0) as usize;
            }
            TextViewerAction::PageScroll(pages) => {
                let delta = pages * view.visible_rows.max(1) as i32;
                let next = view.scroll_offset as i32 + delta;
                view.scroll_offset = next.max(0) as usize;
            }
        }
    }
    Vec::new()
}
