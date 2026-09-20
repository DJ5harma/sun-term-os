use crate::actions::{Action, ProcessAction};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Paragraph, Row, Table},
};

use crate::app::{
    process_manager::matching_indices,
    state::{AppState, Loadable, ProcessManagerState},
};

use super::{
    interaction::{InteractionLayer, InteractionMap},
    theme,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    _state: &AppState,
    window_id: u64,
    manager: &ProcessManagerState,
    interactions: &mut InteractionMap,
) {
    let header_rows = 2u16;
    let help = if manager.filter_active {
        format!(" filter: {}▌  Esc done · Enter apply ", manager.filter)
    } else {
        " ↑↓ move · / filter · x SIGTERM · r refresh · x kills selection ".to_owned()
    };
    if area.height <= header_rows {
        frame.render_widget(Paragraph::new(help).style(theme::muted()), area);
        return;
    }
    let chunks = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(1),
        ratatui::layout::Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(Paragraph::new(help).style(theme::muted()), chunks[0]);

    match &manager.listing {
        Loadable::Loading => {
            frame.render_widget(
                Paragraph::new("Reading process table…").style(theme::muted()),
                chunks[1],
            );
        }
        Loadable::Failed(error) => {
            frame.render_widget(
                Paragraph::new(error.as_str()).style(Style::default().fg(theme::RED)),
                chunks[1],
            );
        }
        Loadable::Ready(processes) if processes.is_empty() => {
            frame.render_widget(
                Paragraph::new("No processes reported.").style(theme::muted()),
                chunks[1],
            );
        }
        Loadable::Ready(processes) => {
            let indices = matching_indices(processes, &manager.filter);
            if indices.is_empty() {
                frame.render_widget(
                    Paragraph::new("No processes match the filter.").style(theme::muted()),
                    chunks[1],
                );
                return;
            }
            let visible = manager.visible_rows.max(1);
            let window = indices
                .iter()
                .skip(manager.scroll_offset)
                .take(visible)
                .enumerate();
            let rows = window.map(|(row_index, process_index)| {
                let process = &processes[*process_index];
                let selected = row_index + manager.scroll_offset == manager.selected_index;
                let style = if selected {
                    theme::active()
                } else {
                    Style::default().fg(theme::TEXT)
                };
                Row::new(vec![
                    process.pid.to_string(),
                    process.name.clone(),
                    format!("{:>5.1}%", process.cpu_percent),
                    format!("{:.0}M", process.memory_bytes as f64 / 1_000_000.0),
                    process.state.clone(),
                ])
                .style(style)
            });
            let table_area = chunks[1];
            frame.render_widget(
                Table::new(
                    rows,
                    [
                        ratatui::layout::Constraint::Length(7),
                        ratatui::layout::Constraint::Min(10),
                        ratatui::layout::Constraint::Length(8),
                        ratatui::layout::Constraint::Length(10),
                        ratatui::layout::Constraint::Length(8),
                    ],
                )
                .header(
                    Row::new(vec!["PID", "NAME", "CPU", "MEM", "STATE"])
                        .style(theme::muted().add_modifier(Modifier::BOLD)),
                )
                .column_spacing(1),
                table_area,
            );
            let row_height = 1u16;
            for (display_row, _) in indices
                .iter()
                .skip(manager.scroll_offset)
                .take(visible)
                .enumerate()
            {
                let y = table_area.y + 1 + display_row as u16;
                if y >= table_area.y + table_area.height {
                    break;
                }
                let row_rect = Rect {
                    x: table_area.x,
                    y,
                    width: table_area.width,
                    height: row_height,
                };
                let list_index = manager.scroll_offset + display_row;
                interactions.register(
                    InteractionLayer::Content,
                    row_rect,
                    Action::Process(ProcessAction::SelectProcessRow(window_id, list_index)),
                );
            }
        }
    }
}
