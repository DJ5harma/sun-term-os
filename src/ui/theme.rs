use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};

use crate::config::ThemeConfig;

const DEFAULT_BG: Color = Color::Rgb(13, 16, 20);
const DEFAULT_SURFACE: Color = Color::Rgb(22, 27, 34);
const DEFAULT_SURFACE_ALT: Color = Color::Rgb(31, 39, 49);
const DEFAULT_TEXT: Color = Color::Rgb(224, 229, 236);
const DEFAULT_MUTED: Color = Color::Rgb(126, 140, 157);
const DEFAULT_AMBER: Color = Color::Rgb(243, 184, 76);
const DEFAULT_BLUE: Color = Color::Rgb(100, 190, 255);
const DEFAULT_GREEN: Color = Color::Rgb(113, 213, 148);
const DEFAULT_RED: Color = Color::Rgb(237, 116, 116);

struct Palette {
    bg: Color,
    surface: Color,
    surface_alt: Color,
    text: Color,
    muted: Color,
    amber: Color,
    red: Color,
}

static PALETTE: OnceLock<Palette> = OnceLock::new();

pub fn init(theme: &ThemeConfig) {
    use crate::config::theme::parse_hex_color;
    let _ = PALETTE.set(Palette {
        bg: theme
            .background
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or(DEFAULT_BG),
        surface: theme
            .surface
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or(DEFAULT_SURFACE),
        surface_alt: DEFAULT_SURFACE_ALT,
        text: theme
            .text
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or(DEFAULT_TEXT),
        muted: DEFAULT_MUTED,
        amber: theme
            .accent
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or(DEFAULT_AMBER),
        red: DEFAULT_RED,
    });
}

fn p() -> &'static Palette {
    PALETTE.get().unwrap_or(&Palette {
        bg: DEFAULT_BG,
        surface: DEFAULT_SURFACE,
        surface_alt: DEFAULT_SURFACE_ALT,
        text: DEFAULT_TEXT,
        muted: DEFAULT_MUTED,
        amber: DEFAULT_AMBER,
        red: DEFAULT_RED,
    })
}

pub const BG: Color = DEFAULT_BG;
pub const SURFACE: Color = DEFAULT_SURFACE;
pub const SURFACE_ALT: Color = DEFAULT_SURFACE_ALT;
pub const TEXT: Color = DEFAULT_TEXT;
pub const MUTED: Color = DEFAULT_MUTED;
pub const AMBER: Color = DEFAULT_AMBER;
pub const BLUE: Color = DEFAULT_BLUE;
pub const GREEN: Color = DEFAULT_GREEN;
pub const RED: Color = DEFAULT_RED;

pub fn surface() -> Color {
    p().surface
}
pub fn text() -> Color {
    p().text
}
pub fn muted_color() -> Color {
    p().muted
}
pub fn red() -> Color {
    p().red
}

pub fn base() -> Style {
    Style::default().bg(p().bg)
}
pub fn active() -> Style {
    Style::default()
        .fg(p().bg)
        .bg(p().amber)
        .add_modifier(Modifier::BOLD)
}
pub fn muted() -> Style {
    Style::default().fg(p().muted)
}

pub fn focused_border() -> Style {
    Style::default().fg(p().amber)
}
pub fn unfocused_border() -> Style {
    Style::default().fg(p().surface_alt)
}
