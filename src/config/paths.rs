use std::path::PathBuf;

/// Config directory under `$XDG_CONFIG_HOME` / `~/.config`.
pub const APP_CONFIG_DIR: &str = "sun-term-os";

pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join(APP_CONFIG_DIR);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(format!(".config/{}", APP_CONFIG_DIR));
    }
    PathBuf::from(APP_CONFIG_DIR)
}

pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn default_session_path() -> PathBuf {
    config_dir().join("session.toml")
}
