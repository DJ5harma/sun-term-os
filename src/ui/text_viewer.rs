use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::app::{
    Loadable,
    offline::window_machine_offline_hint,
    text_viewer::{TextViewerDialog, TextViewerState},
};
use crate::domain::WindowId;

use super::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &crate::app::AppState,
    window_id: WindowId,
    view: &TextViewerState,
) {
    if let TextViewerDialog::OpenPath { input } = &view.dialog {
        let body = format!("  Open file: {input}▌  Enter · Esc cancel");
        frame.render_widget(Paragraph::new(body).style(theme::muted()), area);
        return;
    }

    if let Some(hint) = window_machine_offline_hint(state, window_id) {
        frame.render_widget(Paragraph::new(hint).style(theme::muted()), area);
        return;
    }

    if !view.has_path() {
        let help = "  No file open.\n\n  : type a path · or open a file in the file manager (e)";
        frame.render_widget(Paragraph::new(help).style(theme::muted()), area);
        return;
    }

    let header = format!(" {} ", view.path.display());
    match &view.content {
        Loadable::Loading => {
            frame.render_widget(Paragraph::new(format!("{header}\nLoading…")), area)
        }
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(format!("{header}\n\n{error}")).style(theme::muted()),
            area,
        ),
        Loadable::Ready(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let body = lines
                .iter()
                .enumerate()
                .skip(view.scroll_offset)
                .take(view.visible_rows.max(1))
                .map(|(index, line)| format!("{:>4} │ {}", index + 1, line))
                .collect::<Vec<_>>()
                .join("\n");
            let footer = "\n  ↑↓ scroll · : open path";
            frame.render_widget(Paragraph::new(format!("{header}\n{body}{footer}")), area);
        }
    }
}
