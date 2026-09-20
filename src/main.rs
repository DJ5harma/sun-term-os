mod actions;
mod app;
mod apps;
mod config;
mod domain;
mod events;
mod input;
mod machine;
mod terminal;
mod ui;

use std::io::{self, stdout};

use anyhow::Result;
use app::App;
use crossterm::{
    event::{
        EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let config = crate::config::load()?;
    ui::theme::init(&config.theme);
    enable_terminal()?;
    let result = run(config).await;
    disable_terminal()?;
    result
}

async fn run(config: crate::config::Config) -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(config);
    app.run(&mut terminal).await
}

fn enable_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ),
    )?;
    Ok(())
}

fn disable_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        PopKeyboardEnhancementFlags,
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}
