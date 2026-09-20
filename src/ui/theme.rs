use std::sync::{LazyLock, RwLock};

use ratatui::style::{Color, Modifier, Style};

use crate::config::{ThemeConfig, parse_hex_color};

const DEFAULT_BG: Color = Color::Rgb(13, 16, 20);
const DEFAULT_SURFACE: Color = Color::Rgb(22, 27, 34);
const DEFAULT_SURFACE_ALT: Color = Color::Rgb(31, 39, 49);
const DEFAULT_TEXT: Color = Color::Rgb(224, 229, 236);
const DEFAULT_MUTED: Color = Color::Rgb(126, 140, 157);
const DEFAULT_AMBER: Color = Color::Rgb(243, 184, 76);
const DEFAULT_BLUE: Color = Color::Rgb(100, 190, 255);
const DEFAULT_GREEN: Color = Color::Rgb(113, 213, 148);
const DEFAULT_RED: Color = Color::Rgb(237, 116, 116);

#[derive(Clone, Copy)]
struct Palette {
    bg: Color,
    surface: Color,
    surface_alt: Color,
    text: Color,
    muted: Color,
    amber: Color,
    red: Color,
}

fn default_palette() -> Palette {
    Palette {
        bg: DEFAULT_BG,
        surface: DEFAULT_SURFACE,
        surface_alt: DEFAULT_SURFACE_ALT,
        text: DEFAULT_TEXT,
        muted: DEFAULT_MUTED,
        amber: DEFAULT_AMBER,
        red: DEFAULT_RED,
    }
}

fn color_field(value: &Option<String>, fallback: Color) -> Color {
    value
        .as_deref()
        .and_then(parse_hex_color)
        .unwrap_or(fallback)
}

fn build_palette(theme: &ThemeConfig) -> Palette {
    Palette {
        bg: color_field(&theme.background, DEFAULT_BG),
        surface: color_field(&theme.surface, DEFAULT_SURFACE),
        surface_alt: color_field(&theme.surface_alt, DEFAULT_SURFACE_ALT),
        text: color_field(&theme.text, DEFAULT_TEXT),
        muted: color_field(&theme.muted, DEFAULT_MUTED),
        amber: color_field(&theme.accent, DEFAULT_AMBER),
        red: color_field(&theme.red, DEFAULT_RED),
    }
}

static PALETTE: LazyLock<RwLock<Palette>> = LazyLock::new(|| RwLock::new(default_palette()));

pub fn init(theme: &ThemeConfig) {
    reload(theme);
}

pub fn reload(theme: &ThemeConfig) {
    if let Ok(mut palette) = PALETTE.write() {
        *palette = build_palette(theme);
    }
}

fn p() -> Palette {
    PALETTE
        .read()
        .map(|palette| *palette)
        .unwrap_or_else(|_| default_palette())
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
