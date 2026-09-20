//! Named color combos for the Settings UI only — applied as hex fields in [`ThemeConfig`].

use crate::config::ThemeConfig;

#[derive(Debug, Clone, Copy)]
struct Preset {
    label: &'static str,
    background: &'static str,
    surface: &'static str,
    surface_alt: &'static str,
    text: &'static str,
    muted: &'static str,
    accent: &'static str,
    red: &'static str,
}

const PRESETS: [Preset; 6] = [
    Preset {
        label: "Default (amber)",
        background: "#0d1014",
        surface: "#161b22",
        surface_alt: "#1f2731",
        text: "#e0e5ec",
        muted: "#7e8c9d",
        accent: "#f3b84c",
        red: "#ed7474",
    },
    Preset {
        label: "Midnight (blue)",
        background: "#0a0e14",
        surface: "#121820",
        surface_alt: "#1a2330",
        text: "#d4e4f7",
        muted: "#6b7d94",
        accent: "#64beff",
        red: "#f07178",
    },
    Preset {
        label: "Forest (green)",
        background: "#0c1210",
        surface: "#152019",
        surface_alt: "#1e2b24",
        text: "#dce8e0",
        muted: "#7a9186",
        accent: "#71d594",
        red: "#e06c75",
    },
    Preset {
        label: "Rose (warm)",
        background: "#141018",
        surface: "#1e1824",
        surface_alt: "#2a2230",
        text: "#f0e6ee",
        muted: "#9a8a96",
        accent: "#e8a0bf",
        red: "#ff6b6b",
    },
    Preset {
        label: "Slate (neutral)",
        background: "#111318",
        surface: "#1a1d24",
        surface_alt: "#252a33",
        text: "#e2e4e9",
        muted: "#8b929e",
        accent: "#a8b0bd",
        red: "#d97373",
    },
    Preset {
        label: "High contrast",
        background: "#000000",
        surface: "#121212",
        surface_alt: "#2a2a2a",
        text: "#ffffff",
        muted: "#b0b0b0",
        accent: "#ffcc00",
        red: "#ff5555",
    },
];

pub fn len() -> usize {
    PRESETS.len()
}

pub fn label(index: usize) -> &'static str {
    PRESETS
        .get(index % PRESETS.len())
        .map(|preset| preset.label)
        .unwrap_or(PRESETS[0].label)
}

pub fn index_for_theme(theme: &ThemeConfig) -> usize {
    PRESETS
        .iter()
        .position(|preset| preset.matches(theme))
        .unwrap_or(0)
}

pub fn apply(index: usize, theme: &mut ThemeConfig) {
    let preset = &PRESETS[index % PRESETS.len()];
    preset.write_into(theme);
}

impl Preset {
    fn matches(&self, theme: &ThemeConfig) -> bool {
        theme.background.as_deref() == Some(self.background)
            && theme.surface.as_deref() == Some(self.surface)
            && theme.surface_alt.as_deref() == Some(self.surface_alt)
            && theme.text.as_deref() == Some(self.text)
            && theme.muted.as_deref() == Some(self.muted)
            && theme.accent.as_deref() == Some(self.accent)
            && theme.red.as_deref() == Some(self.red)
    }

    fn write_into(&self, theme: &mut ThemeConfig) {
        theme.background = Some(self.background.to_owned());
        theme.surface = Some(self.surface.to_owned());
        theme.surface_alt = Some(self.surface_alt.to_owned());
        theme.text = Some(self.text.to_owned());
        theme.muted = Some(self.muted.to_owned());
        theme.accent = Some(self.accent.to_owned());
        theme.red = Some(self.red.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_round_trip() {
        let mut theme = ThemeConfig::default();
        apply(1, &mut theme);
        assert_eq!(index_for_theme(&theme), 1);
    }
}
