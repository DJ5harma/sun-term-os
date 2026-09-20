use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{AppState, Loadable, Panel};

const BG: Color = Color::Rgb(15, 18, 22);
const SURFACE: Color = Color::Rgb(24, 29, 36);
const SURFACE_ALT: Color = Color::Rgb(31, 38, 47);
const TEXT: Color = Color::Rgb(221, 226, 232);
const MUTED: Color = Color::Rgb(130, 143, 158);
const AMBER: Color = Color::Rgb(241, 181, 76);
const BLUE: Color = Color::Rgb(97, 183, 255);
const RED: Color = Color::Rgb(237, 116, 116);

pub fn render(frame: &mut Frame, state: &AppState) {
    frame.render_widget(
        Block::default().style(Style::default().bg(BG)),
        frame.area(),
    );
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());
    render_header(frame, root[0], state);
    render_body(frame, root[1], state);
    render_footer(frame, root[2], state);
    if state.palette_open {
        render_palette(frame, state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(10),
            Constraint::Length(24),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " TDE ",
                Style::default()
                    .fg(BG)
                    .bg(AMBER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  terminal desktop",
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ])),
        chunks[0],
    );
    let workspace = state
        .workspaces
        .get(state.active_workspace)
        .map_or("Workspace", |item| item.name.as_str());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("/ ", Style::default().fg(MUTED)),
            Span::styled(
                workspace,
                Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ·  ", Style::default().fg(MUTED)),
            Span::styled(&state.machine.name, Style::default().fg(TEXT)),
        ]))
        .alignment(ratatui::layout::Alignment::Center),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new("LOCAL  ●")
            .style(Style::default().fg(BLUE))
            .alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );
}

fn render_body(frame: &mut Frame, area: Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(32)])
        .split(area);
    render_workspace_rail(frame, columns[0], state);

    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(columns[1]);
    render_overview(frame, panels[0], state);
    let lower = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(panels[1]);
    render_system(frame, lower[0], state);
    render_processes(frame, lower[1], state);
}

fn render_workspace_rail(frame: &mut Frame, area: Rect, state: &AppState) {
    let items = state
        .workspaces
        .iter()
        .enumerate()
        .map(|(index, workspace)| {
            let marker = if index == state.active_workspace {
                "▌"
            } else {
                " "
            };
            let style = if index == state.active_workspace {
                Style::default().fg(AMBER).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("  {}  {}", index + 1, workspace.name), style),
            ]))
        });
    let block = Block::default()
        .title(" WORKSPACES ")
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(SURFACE_ALT));
    frame.render_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(SURFACE)),
        area,
    );
}

fn render_overview(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = format!(" {}  /  {} ", state.focused_panel.title(), state.status);
    let block = panel_block(title, state.focused_panel == Panel::Overview);
    let text = match (&state.system, &state.processes) {
        (Loadable::Ready(system), Loadable::Ready(processes)) => format!(
            "{} CPU cores   ·   {} processes   ·   {} uptime\nMemory {:>5.1}% used   ·   kernel {}",
            system.cpu_count,
            processes.len(),
            format_uptime(system.uptime_seconds),
            memory_percent(system.memory_used, system.memory_total),
            system.kernel
        ),
        _ => "Capabilities are warming up…".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(TEXT))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_system(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = panel_block(" SYSTEM ", state.focused_panel == Panel::System);
    match &state.system {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("Collecting system telemetry…")
                .block(block)
                .style(Style::default().fg(MUTED)),
            area,
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str())
                .block(block)
                .style(Style::default().fg(RED)),
            area,
        ),
        Loadable::Ready(system) => {
            let rows = vec![
                Row::new(vec!["Host".to_owned(), system.hostname.clone()]),
                Row::new(vec!["OS".to_owned(), system.os.clone()]),
                Row::new(vec!["Kernel".to_owned(), system.kernel.clone()]),
                Row::new(vec![
                    "Uptime".to_owned(),
                    format_uptime(system.uptime_seconds),
                ]),
            ];
            frame.render_widget(
                Table::new(rows, [Constraint::Length(10), Constraint::Min(12)])
                    .block(block)
                    .column_spacing(1)
                    .style(Style::default().fg(TEXT)),
                area,
            );
        }
    }
}

fn render_processes(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = panel_block(
        " PROCESSES  /  CPU ",
        state.focused_panel == Panel::Processes,
    );
    match &state.processes {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("Reading process table…")
                .block(block)
                .style(Style::default().fg(MUTED)),
            area,
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str())
                .block(block)
                .style(Style::default().fg(RED)),
            area,
        ),
        Loadable::Ready(processes) if processes.is_empty() => frame.render_widget(
            Paragraph::new("No processes reported.")
                .block(block)
                .style(Style::default().fg(MUTED)),
            area,
        ),
        Loadable::Ready(processes) => {
            let rows = processes
                .iter()
                .take(area.height.saturating_sub(4) as usize)
                .map(|process| {
                    Row::new(vec![
                        process.pid.to_string(),
                        truncate(&process.name, 20),
                        format!("{:>5.1}%", process.cpu_percent),
                        format_bytes(process.memory_bytes),
                    ])
                });
            let table = Table::new(
                rows,
                [
                    Constraint::Length(7),
                    Constraint::Min(12),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ],
            )
            .header(Row::new(vec!["PID", "NAME", "CPU", "MEM"]).style(Style::default().fg(MUTED)))
            .block(block)
            .column_spacing(1)
            .style(Style::default().fg(TEXT));
            frame.render_widget(table, area);
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let help = if state.palette_open {
        "↑↓ select   enter run   esc close"
    } else {
        "p command palette   tab focus   1–3 workspace   r refresh   q quit"
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(help, Style::default().fg(MUTED)),
            Span::raw("  "),
            Span::styled("TDE/0.1", Style::default().fg(AMBER)),
        ])),
        area,
    );
}

fn render_palette(frame: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 44, frame.area());
    frame.render_widget(Clear, area);
    let items = [
        "Refresh capabilities",
        "Open Main workspace",
        "Open Monitor workspace",
        "Quit TDE",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, label)| {
        let style = if index == state.palette_selection {
            Style::default()
                .fg(BG)
                .bg(AMBER)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT)
        };
        ListItem::new(Span::styled(format!("  {}  ", label), style))
    });
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(" COMMAND PALETTE ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(AMBER))
                .style(Style::default().bg(SURFACE)),
        ),
        area,
    );
}

fn panel_block(title: impl Into<ratatui::text::Line<'static>>, active: bool) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if active { AMBER } else { SURFACE_ALT }))
        .style(Style::default().bg(SURFACE))
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn format_uptime(seconds: u64) -> String {
    format!("{}h {:02}m", seconds / 3600, (seconds % 3600) / 60)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
    } else {
        format!("{:.0}M", bytes as f64 / 1_000_000.0)
    }
}

fn memory_percent(used: u64, total: u64) -> String {
    if total == 0 {
        "0.0%".to_owned()
    } else {
        format!("{:.1}%", used as f64 / total as f64 * 100.0)
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_owned()
    } else {
        format!(
            "{}…",
            value
                .chars()
                .take(max.saturating_sub(1))
                .collect::<String>()
        )
    }
}
