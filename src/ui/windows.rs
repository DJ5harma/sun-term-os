use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

use crate::{
    app::{AppState, Window},
    apps,
};

use super::{interaction::InteractionMap, theme};

fn window_block(title: String) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(ratatui::style::Style::default().fg(theme::AMBER))
        .style(ratatui::style::Style::default().bg(theme::SURFACE))
}

/// Content area inside the window chrome block (must match [render]).
pub(crate) fn content_inner(area: Rect, window: &Window, state: &AppState) -> Rect {
    let title = format!(
        " {}  ·  {} ",
        apps::title(window.application),
        state.host_label()
    );
    window_block(title).inner(area)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    interactions: &mut InteractionMap,
) {
    let title = format!(
        " {}  ·  {} ",
        apps::title(window.application),
        state.host_label()
    );
    let block = window_block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    crate::apps::render(
        window.application,
        frame,
        inner,
        state,
        window,
        interactions,
    );
}
