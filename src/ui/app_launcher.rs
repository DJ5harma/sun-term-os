use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Row, Table},
};

use crate::app::{Loadable, launcher::LauncherState};

use super::{interaction::InteractionMap, theme};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    view: &LauncherState,
    _interactions: &mut InteractionMap,
) {
    let help = if view.filter_active {
        format!(" filter: {}▌ ", view.filter)
    } else {
        " ↑↓ move · / filter · Enter launch ".to_owned()
    };
    match &view.listing {
        Loadable::Loading => frame.render_widget(Paragraph::new("Discovering applications…"), area),
        Loadable::Failed(error) => {
            frame.render_widget(Paragraph::new(format!("Failed: {error}")), area);
        }
        Loadable::Ready(entries) => {
            let filter = view.filter.to_lowercase();
            let rows: Vec<Row> = entries
                .iter()
                .filter(|entry| {
                    filter.is_empty()
                        || format!("{} {}", entry.name, entry.detail)
                            .to_lowercase()
                            .contains(&filter)
                })
                .map(|entry| Row::new(vec![entry.name.clone(), entry.detail.clone()]))
                .collect();
            let table = Table::new(
                rows,
                [
                    ratatui::layout::Constraint::Percentage(35),
                    ratatui::layout::Constraint::Percentage(65),
                ],
            )
            .header(Row::new(vec!["Name", "Detail"]).style(theme::muted()));
            let mut table_state =
                ratatui::widgets::TableState::default().with_selected(Some(view.selected_index));
            frame.render_stateful_widget(table, area, &mut table_state);
        }
    }
    if area.height > 1 {
        frame.render_widget(
            Paragraph::new(help).style(theme::muted()),
            Rect {
                x: area.x,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1,
            },
        );
    }
}
