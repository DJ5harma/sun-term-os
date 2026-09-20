use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(13, 16, 20);
pub const SURFACE: Color = Color::Rgb(22, 27, 34);
pub const SURFACE_ALT: Color = Color::Rgb(31, 39, 49);
pub const TEXT: Color = Color::Rgb(224, 229, 236);
pub const MUTED: Color = Color::Rgb(126, 140, 157);
pub const AMBER: Color = Color::Rgb(243, 184, 76);
pub const BLUE: Color = Color::Rgb(100, 190, 255);
pub const GREEN: Color = Color::Rgb(113, 213, 148);
pub const RED: Color = Color::Rgb(237, 116, 116);

pub fn base() -> Style {
    Style::default().bg(BG)
}
pub fn active() -> Style {
    Style::default()
        .fg(BG)
        .bg(AMBER)
        .add_modifier(Modifier::BOLD)
}
pub fn muted() -> Style {
    Style::default().fg(MUTED)
}
