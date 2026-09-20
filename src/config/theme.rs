use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub surface_alt: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub muted: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub red: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: Some("#0d1014".to_owned()),
            surface: Some("#161b22".to_owned()),
            surface_alt: Some("#1f2731".to_owned()),
            text: Some("#e0e5ec".to_owned()),
            muted: Some("#7e8c9d".to_owned()),
            accent: Some("#f3b84c".to_owned()),
            red: Some("#ed7474".to_owned()),
        }
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}
