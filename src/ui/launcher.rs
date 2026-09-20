use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::{AppState, palette::filtered_entries};

use super::{
    interaction::{InteractionLayer, InteractionMap},
    theme,
};

fn palette_block() -> Block<'static> {
    Block::default()
        .title(" COMMAND PALETTE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent()))
        .style(Style::default().bg(theme::surface()))
}

/// List rows that fit below the query line (matches scroll clamping).
pub fn list_visible_rows(launcher_area: Rect) -> usize {
    let inner = palette_block().inner(launcher_area);
    if inner.height < 2 {
        return 1;
    }
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    chunks[1].height.max(1) as usize
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, interactions: &mut InteractionMap) {
    frame.render_widget(Clear, area);
    let block = palette_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);

    let query_line = if state.launcher_query.is_empty() {
        Line::from(vec![
            Span::styled("> ", theme::active()),
            Span::styled("Filter commands · !run shell line", theme::muted()),
        ])
    } else {
        Line::from(vec![
            Span::styled("> ", theme::active()),
            Span::styled(
                state.launcher_query.as_str(),
                Style::default().fg(theme::text()),
            ),
            Span::styled("▌", theme::active()),
        ])
    };
    frame.render_widget(Paragraph::new(query_line), chunks[0]);

    let entries = filtered_entries(state);
    let selection = state.launcher_selection;
    let scroll_offset = state.launcher_scroll_offset;
    let visible_rows = list_visible_rows(area);

    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No matching commands").style(theme::muted()),
            chunks[1],
        );
        return;
    }

    let visible = entries
        .iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .enumerate()
        .map(|(row, entry)| {
            let index = scroll_offset + row;
            let selected = index == selection;
            let style = if selected {
                theme::active()
            } else {
                Style::default().fg(theme::text())
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}  ", entry.title), style),
                Span::styled(
                    entry.detail.as_str(),
                    if selected { style } else { theme::muted() },
                ),
            ]))
        });

    frame.render_widget(List::new(visible), chunks[1]);

    let rows = Layout::vertical(std::iter::repeat_n(
        Constraint::Length(1),
        visible_rows.min(entries.len().saturating_sub(scroll_offset)),
    ))
    .split(chunks[1]);
    for (entry, row) in entries
        .iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .zip(rows.iter())
    {
        interactions.register(InteractionLayer::Modal, *row, entry.action.clone());
    }
}
