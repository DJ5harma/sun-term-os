use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Paragraph, Row, Table},
};

use crate::app::Loadable;

use super::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &crate::app::AppState,
    window_id: u64,
    listing: &Loadable<crate::machine::SystemSnapshot>,
    refreshed_at: Option<u64>,
) {
    let footer = refreshed_at
        .map(format_age)
        .unwrap_or_else(|| "Press r to refresh".to_owned());
    if area.height < 3 {
        frame.render_widget(Paragraph::new(footer).style(theme::muted()), area);
        return;
    }
    let chunks = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Min(1),
        ratatui::layout::Constraint::Length(1),
    ])
    .split(area);
    if let Some(hint) = crate::app::offline::window_machine_offline_hint(state, window_id) {
        frame.render_widget(Paragraph::new(hint).style(theme::muted()), chunks[0]);
        frame.render_widget(Paragraph::new(footer).style(theme::muted()), chunks[1]);
        return;
    }
    match listing {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("Collecting system telemetry…").style(theme::muted()),
            chunks[0],
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(theme::red())),
            chunks[0],
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
                        "Load".to_owned(),
                        format!(
                            "{:.2} {:.2} {:.2}",
                            system.load_one, system.load_five, system.load_fifteen
                        ),
                    ]),
                    Row::new(vec![
                        "Memory".to_owned(),
                        format!(
                            "{:.1}% used ({:.1} / {:.1} GiB)",
                            if system.memory_total == 0 {
                                0.0
                            } else {
                                system.memory_used as f64 / system.memory_total as f64 * 100.0
                            },
                            system.memory_used as f64 / 1_073_741_824.0,
                            system.memory_total as f64 / 1_073_741_824.0,
                        ),
                    ]),
                    Row::new(vec!["CPUs".to_owned(), system.cpu_count.to_string()]),
                ]
                .into_iter()
                .chain(system.disks.iter().take(6).map(|disk| {
                    let used_pct = if disk.total_bytes == 0 {
                        0.0
                    } else {
                        (disk.total_bytes - disk.available_bytes) as f64 / disk.total_bytes as f64
                            * 100.0
                    };
                    Row::new(vec![
                        format!("Disk {}", disk.mount_point),
                        format!(
                            "{:.0}% free · {:.1}/{:.1} GiB",
                            100.0 - used_pct,
                            disk.available_bytes as f64 / 1_073_741_824.0,
                            disk.total_bytes as f64 / 1_073_741_824.0,
                        ),
                    ])
                }))
                .collect::<Vec<Row>>(),
                [Constraint::Length(16), Constraint::Min(12)],
            )
            .column_spacing(1)
            .style(Style::default().fg(theme::text())),
            chunks[0],
        ),
    }
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), chunks[1]);
}

fn format_age(refreshed_at: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let age = now.saturating_sub(refreshed_at);
    if age < 5 {
        "Updated just now · r refresh".to_owned()
    } else {
        format!("Updated {age}s ago · r refresh")
    }
}
