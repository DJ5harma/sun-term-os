use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Paragraph, Row, Table},
};

use crate::app::Loadable;

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, listing: &Loadable<crate::machine::SystemSnapshot>) {
    match listing {
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
                .chain(system.disks.iter().take(4).map(|disk| {
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
            .style(Style::default().fg(theme::TEXT)),
            area,
        ),
    }
}
