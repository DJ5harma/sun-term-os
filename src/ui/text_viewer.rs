use ratatui::{
    Frame,
    layout::{Position, Rect},
    widgets::Paragraph,
};

use crate::app::{
    Loadable,
    offline::window_machine_offline_hint,
    text_viewer::{TextViewerDialog, TextViewerState},
};
use crate::domain::WindowId;

use super::theme;

pub const GUTTER_WIDTH: u16 = 7;

pub struct EditorLayout {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
}

pub fn layout(area: Rect) -> EditorLayout {
    let header = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let body = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(2).max(1),
    };
    EditorLayout {
        header,
        body,
        footer,
    }
}

pub fn click_to_caret(
    area: Rect,
    view: &TextViewerState,
    column: u16,
    row: u16,
) -> Option<(usize, usize)> {
    let layout = layout(area);
    let position = Position::new(column, row);
    if !layout.body.contains(position) {
        return None;
    }
    let rel_y = row.saturating_sub(layout.body.y) as usize;
    let rel_x = column.saturating_sub(layout.body.x + GUTTER_WIDTH) as usize;
    let line = view.scroll_offset + rel_y;
    if line >= view.line_count() {
        return None;
    }
    let line_len = line_char_len(&view.buffer, line);
    let col = rel_x.min(line_len);
    Some((line, col))
}

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

    if matches!(view.dialog, TextViewerDialog::ConfirmDiscardClose) {
        let body = "  Discard unsaved changes and close?\n\n  D or Ctrl+W discard · Ctrl+S save · Esc stay";
        frame.render_widget(Paragraph::new(body).style(theme::muted()), area);
        return;
    }

    if let Some(hint) = window_machine_offline_hint(state, window_id) {
        frame.render_widget(Paragraph::new(hint).style(theme::muted()), area);
        return;
    }

    let layout = layout(area);

    if !view.has_path() {
        let help = "  No file open.\n\n  : type a path · or press e on a file in the file manager";
        frame.render_widget(Paragraph::new(help).style(theme::muted()), area);
        return;
    }

    let dirty = if view.is_dirty() { " *" } else { "" };
    let header = format!(" {}{} ", view.path.display(), dirty);
    frame.render_widget(Paragraph::new(header), layout.header);

    match &view.load {
        Loadable::Loading => {
            frame.render_widget(Paragraph::new("Loading…"), layout.body);
        }
        Loadable::Failed(error) => {
            frame.render_widget(
                Paragraph::new(error.clone()).style(theme::muted()),
                layout.body,
            );
        }
        Loadable::Ready(()) => {
            let visible = layout.body.height as usize;
            let mut lines = Vec::new();
            for row in 0..visible {
                let line_index = view.scroll_offset + row;
                if line_index >= view.line_count() {
                    break;
                }
                let line = line_text_at(&view.buffer, line_index);
                lines.push(format!("{:>4} │ {}", line_index + 1, line));
            }
            let body_text = lines.join("\n");
            frame.render_widget(Paragraph::new(body_text), layout.body);

            if view.is_editable() {
                let cursor_screen_y =
                    layout.body.y + (view.cursor_line.saturating_sub(view.scroll_offset)) as u16;
                let line = line_text_at(&view.buffer, view.cursor_line);
                let cursor_screen_x =
                    layout.body.x + GUTTER_WIDTH + display_width_before_col(line, view.cursor_col);
                if cursor_screen_y < layout.body.y + layout.body.height {
                    frame.set_cursor_position((cursor_screen_x, cursor_screen_y));
                }
            }
        }
    }

    let footer = if view.is_dirty() {
        "  arrows · edit · Ctrl+S save · Ctrl+W close · : path"
    } else {
        "  arrows · type to edit · Ctrl+S save · : open path"
    };
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), layout.footer);
}

fn line_ranges(text: &str) -> Vec<(usize, usize)> {
    if text.is_empty() {
        return vec![(0, 0)];
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            ranges.push((start, index));
            start = index + 1;
        }
    }
    ranges.push((start, text.len()));
    ranges
}

fn line_text_at(buffer: &str, line: usize) -> &str {
    let ranges = line_ranges(buffer);
    let Some(&(start, end)) = ranges.get(line) else {
        return "";
    };
    &buffer[start..end]
}

fn line_char_len(buffer: &str, line: usize) -> usize {
    line_text_at(buffer, line).chars().count()
}

fn display_width_before_col(line: &str, col: usize) -> u16 {
    line.chars().take(col).count() as u16
}
