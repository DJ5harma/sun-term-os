use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Paragraph, Row, Table},
};

use crate::app::{
    machines::{MachinesDialog, MachinesState},
    state::AppState,
};
use crate::machine::{MachineId, registry::ConnectionState};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, view: &MachinesState) {
    if matches!(view.dialog, MachinesDialog::None) {
        render_list(frame, area, state, view);
    } else {
        render_dialog(frame, area, state, view);
    }
}

fn render_list(frame: &mut Frame, area: Rect, state: &AppState, view: &MachinesState) {
    let help = " ↑↓ · Enter active · c connect · d disconnect · a add · e edit · x delete";
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
    let mut rows = vec![Row::new(vec![
        "●".to_owned(),
        "Local".to_owned(),
        "this computer".to_owned(),
        connection_label(&ConnectionState::Connected).to_owned(),
    ])];
    for profile in &state.config.machines {
        let connection = state
            .machine_connections
            .get(&profile.id)
            .cloned()
            .unwrap_or(ConnectionState::Disconnected);
        let active = state.active_machine_id == MachineId::Named(profile.id.clone());
        rows.push(Row::new(vec![
            if active { "●" } else { " " }.to_owned(),
            profile.display_label().to_owned(),
            profile.host.clone(),
            connection_label(&connection).to_owned(),
        ]));
    }
    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(2),
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Percentage(20),
        ],
    )
    .header(Row::new(vec!["", "Name", "Host", "State"]).style(theme::muted()))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut table_state =
        ratatui::widgets::TableState::default().with_selected(Some(view.selected));
    frame.render_stateful_widget(table, table_area, &mut table_state);
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

fn connection_label(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected => "connected",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Disconnected => "offline",
        ConnectionState::Failed(_) => "failed",
    }
}

fn render_dialog(frame: &mut Frame, area: Rect, state: &AppState, view: &MachinesState) {
    let text = match &view.dialog {
        MachinesDialog::None => String::new(),
        MachinesDialog::AddProfile { input } => format!("Add host: {input}▌"),
        MachinesDialog::EditProfile { input, .. } => format!("Edit host: {input}▌"),
        MachinesDialog::DeleteConfirm { label, .. } => {
            format!("Delete {label}? Enter confirm · Esc cancel")
        }
        MachinesDialog::HostKeyConfirm {
            fingerprint, host, ..
        } => {
            format!("Trust host key for {host}?\n{fingerprint}\n y accept · n reject")
        }
    };
    frame.render_widget(Paragraph::new(text), area);
    let _ = state;
}
