use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Row, Table},
};

use crate::app::{Loadable, services_manager::ServicesState};

use super::{interaction::InteractionMap, theme};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    view: &ServicesState,
    _interactions: &mut InteractionMap,
) {
    let help = if view.filter_active {
        format!(" filter: {}▌ ", view.filter)
    } else {
        " ↑↓ · / filter · s start · x stop · r restart ".to_owned()
    };
    match &view.listing {
        Loadable::Loading => frame.render_widget(Paragraph::new("Loading services…"), area),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(format!("Services unavailable\n{error}")).style(theme::muted()),
            area,
        ),
        Loadable::Ready(services) => {
            let filter = view.filter.to_lowercase();
            let rows: Vec<Row> = services
                .iter()
                .filter(|service| {
                    filter.is_empty() || service.name.to_lowercase().contains(&filter)
                })
                .map(|service| {
                    Row::new(vec![
                        service.name.clone(),
                        service.load_state.clone(),
                        service.active_state.clone(),
                        service.sub_state.clone(),
                    ])
                })
                .collect();
            let table = Table::new(
                rows,
                [
                    ratatui::layout::Constraint::Percentage(45),
                    ratatui::layout::Constraint::Percentage(20),
                    ratatui::layout::Constraint::Percentage(20),
                    ratatui::layout::Constraint::Percentage(15),
                ],
            )
            .header(Row::new(vec!["Unit", "Load", "Active", "Sub"]).style(theme::muted()));
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
