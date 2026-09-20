use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use ratatui::layout::{Constraint, Layout};

use crate::{
    actions::Action,
    app::{AppState, ApplicationKind},
};

use super::{
    interaction::{InteractionLayer, InteractionMap},
    theme,
};

fn launcher_block() -> Block<'static> {
    Block::default()
        .title(" APPLICATION LAUNCHER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::AMBER))
        .style(Style::default().bg(theme::SURFACE))
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, interactions: &mut InteractionMap) {
    frame.render_widget(Clear, area);
    let inner = launcher_block().inner(area);
    let items = ApplicationKind::ALL
        .into_iter()
        .enumerate()
        .map(|(index, application)| {
            let selected = index == state.launcher_selection;
            let style = if selected {
                theme::active()
            } else {
                Style::default().fg(theme::TEXT)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}  ", application.title()), style),
                Span::styled(
                    application.launcher_description(),
                    if selected { style } else { theme::muted() },
                ),
            ]))
        });
    frame.render_widget(List::new(items).block(launcher_block()), area);
    let rows = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(std::iter::repeat_n(
            Constraint::Length(1),
            ApplicationKind::ALL.len(),
        ))
        .split(inner);
    for (application, row) in ApplicationKind::ALL.iter().zip(rows.iter()) {
        interactions.register(
            InteractionLayer::Modal,
            *row,
            Action::OpenApplication(*application),
        );
    }
}
