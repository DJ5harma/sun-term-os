use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::app::{Loadable, text_viewer::TextViewerState};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, view: &TextViewerState) {
    let header = format!(" {} ", view.path.display());
    match &view.content {
        Loadable::Loading => frame.render_widget(Paragraph::new("Loading…"), area),
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
            frame.render_widget(Paragraph::new(format!("{header}\n{body}")), area);
        }
    }
}
