use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::{AppState, ApplicationKind, Loadable, Window};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, window: &Window) {
    let block = Block::default()
        .title(format!(
            " {}  ·  {} ",
            window.application.title(),
            state.machine.name
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::AMBER))
        .style(Style::default().bg(theme::SURFACE));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    match window.application {
        ApplicationKind::Terminal => terminal(frame, inner, state, window.id),
        ApplicationKind::SystemInfo => system(frame, inner, state),
        ApplicationKind::Processes => processes(frame, inner, state),
    }
}

fn terminal(frame: &mut Frame, area: Rect, state: &AppState, window_id: u64) {
    let content = state.terminal_content(window_id);
    if content.is_empty() {
        frame.render_widget(
            Paragraph::new("Starting shell…").style(Style::default().fg(theme::MUTED)),
            area,
        );
    } else {
        frame.render_widget(
            Paragraph::new(content).style(Style::default().fg(theme::TEXT)),
            area,
        );
    }
}

fn system(frame: &mut Frame, area: Rect, state: &AppState) {
    match &state.system {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("Collecting system telemetry…").style(theme::muted()),
            area,
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(theme::RED)),
            area,
        ),
        Loadable::Ready(system) => frame.render_widget(
            Table::new(
                vec![
                    Row::new(vec!["Host".to_owned(), system.hostname.clone()]),
                    Row::new(vec!["OS".to_owned(), system.os.clone()]),
                    Row::new(vec!["Kernel".to_owned(), system.kernel.clone()]),
                    Row::new(vec![
                        "Uptime".to_owned(),
                        format!(
                            "{}h {:02}m",
                            system.uptime_seconds / 3600,
                            (system.uptime_seconds % 3600) / 60
                        ),
                    ]),
                    Row::new(vec![
                        "Memory".to_owned(),
                        format!(
                            "{:.1}% used",
                            if system.memory_total == 0 {
                                0.0
                            } else {
                                system.memory_used as f64 / system.memory_total as f64 * 100.0
                            }
                        ),
                    ]),
                ],
                [Constraint::Length(12), Constraint::Min(12)],
            )
            .column_spacing(1)
            .style(Style::default().fg(theme::TEXT)),
            area,
        ),
    }
}

fn processes(frame: &mut Frame, area: Rect, state: &AppState) {
    match &state.processes {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("Reading process table…").style(theme::muted()),
            area,
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(theme::RED)),
            area,
        ),
        Loadable::Ready(processes) if processes.is_empty() => frame.render_widget(
            Paragraph::new("No processes reported.").style(theme::muted()),
            area,
        ),
        Loadable::Ready(processes) => {
            let rows = processes
                .iter()
                .take(area.height.saturating_sub(2) as usize)
                .map(|process| {
                    Row::new(vec![
                        process.pid.to_string(),
                        process.name.clone(),
                        format!("{:>5.1}%", process.cpu_percent),
                        format!("{:.0}M", process.memory_bytes as f64 / 1_000_000.0),
                    ])
                });
            frame.render_widget(
                Table::new(
                    rows,
                    [
                        Constraint::Length(7),
                        Constraint::Min(12),
                        Constraint::Length(8),
                        Constraint::Length(10),
                    ],
                )
                .header(Row::new(vec!["PID", "NAME", "CPU", "MEM"]).style(theme::muted()))
                .column_spacing(1)
                .style(Style::default().fg(theme::TEXT)),
                area,
            );
        }
    }
}
