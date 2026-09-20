use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders},
};

use crate::{
    app::{AppState, Window, WindowState},
    apps,
};

use super::{interaction::InteractionMap, theme};

fn window_title(window: &Window, state: &AppState) -> String {
    let app = apps::title(window.application);
    let extra = if window.application == crate::domain::ApplicationKind::Terminal {
        state
            .terminal_view(window.id)
            .and_then(|view| view.title.as_deref())
            .map(|title| format!(" — {title}"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let state_tag = match window.state {
        WindowState::Minimized => " [min]",
        WindowState::Maximized => " [max]",
        WindowState::Normal => "",
    };
    let host = state.host_label();
    let raw = format!(" {app}{extra}{state_tag}  ·  {host} ");
    truncate_title(raw, 64)
}

fn truncate_title(title: String, max_len: usize) -> String {
    if title.len() <= max_len {
        return title;
    }
    format!(
        "{}… ",
        title
            .chars()
            .take(max_len.saturating_sub(2))
            .collect::<String>()
    )
}

fn window_block(title: String, focused: bool) -> Block<'static> {
    let border = if focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border)
        .style(
            Style::default()
                .bg(theme::surface())
                .add_modifier(if focused {
                    Modifier::empty()
                } else {
                    Modifier::DIM
                }),
        )
}

/// Content area inside the window chrome block (must match [render]).
pub(crate) fn content_inner(area: Rect, window: &Window, state: &AppState) -> Rect {
    window_block(window_title(window, state), true).inner(area)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    interactions: &mut InteractionMap,
) {
    let title = window_title(window, state);
    let block = window_block(title, true);
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
