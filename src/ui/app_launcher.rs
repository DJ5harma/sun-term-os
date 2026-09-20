use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Row, Table},
};

use crate::app::{AppState, Loadable, launcher::LauncherState};

use super::{interaction::InteractionMap, theme};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    view: &LauncherState,
    _interactions: &mut InteractionMap,
) {
    let help = if view.filter_active {
        format!(" filter: {}▌ ", view.filter)
    } else {
        " ↑↓ · / filter · Enter launch · Shift+F pin".to_owned()
    };
    let favorites = &state.config.launcher.favorites;
    let table_area = if area.height > 1 {
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height - 1,
        }
    } else {
        area
    };
    match &view.listing {
        Loadable::Loading => {
            frame.render_widget(Paragraph::new("Discovering applications…"), table_area)
        }
        Loadable::Failed(error) => {
            frame.render_widget(Paragraph::new(format!("Failed: {error}")), table_area);
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
                .map(|entry| {
                    let pin = if favorites.contains(&entry.id) {
                        "★ "
                    } else {
                        "  "
                    };
                    Row::new(vec![format!("{pin}{}", entry.name), entry.detail.clone()])
                })
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
            frame.render_stateful_widget(table, table_area, &mut table_state);
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
