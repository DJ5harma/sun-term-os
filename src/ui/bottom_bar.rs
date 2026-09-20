use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::{AppState, Loadable},
    apps,
    domain::WindowState,
    input::{LAUNCHER_SHORTCUT_HINT, WINDOW_FOCUS_HINT},
};

use super::{
    geometry::{BottomBarGeometry, window_tab_cells},
    theme,
};

pub fn render(frame: &mut Frame, layout: BottomBarGeometry, state: &AppState) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ⊞ ", Style::default().fg(theme::BG).bg(theme::AMBER)),
            Span::styled("Palette", Style::default().fg(theme::TEXT)),
            Span::styled(
                format!(" {LAUNCHER_SHORTCUT_HINT}"),
                Style::default().fg(theme::MUTED),
            ),
        ])),
        layout.launcher,
    );

    let workspace_windows = &state.current_workspace().windows;
    if workspace_windows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(" — ", theme::muted()))),
            layout.windows,
        );
    } else {
        let cells = window_tab_cells(layout.windows, workspace_windows.len());
        for (index, window) in workspace_windows.iter().enumerate() {
            let cell = cells[index];
            let active = Some(window.id) == state.current_workspace().focused_window;
            let slot = index + 1;
            let minimized = window.state == WindowState::Minimized;
            let label = if slot <= 9 {
                let title = apps::short_title(window.application);
                if minimized {
                    format!("{slot}:_{title}")
                } else {
                    format!("{slot}:{title}")
                }
            } else {
                let title = apps::title(window.application);
                if minimized {
                    format!("_{title}")
                } else {
                    title.to_owned()
                }
            };
            let style = if active {
                theme::active()
            } else {
                Style::default().fg(theme::MUTED)
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(format!(" {label} "), style)))
                    .alignment(Alignment::Center),
                cell,
            );
        }
    }

    let process_count = match &state.processes {
        Loadable::Ready(processes) => format!("{} proc", processes.len()),
        Loadable::Loading => "proc…".to_owned(),
        Loadable::Failed(_) => "proc —".to_owned(),
    };
    let status_text = if state.input_debug && !state.input_debug_line.is_empty() {
        state.input_debug_line.clone()
    } else if state.window_pick_mode {
        "pick 1–9 · Esc cancel".to_owned()
    } else {
        format!("{process_count} · {WINDOW_FOCUS_HINT} · {}", state.status)
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(status_text, theme::muted())))
            .alignment(Alignment::Right),
        layout.status,
    );
}
