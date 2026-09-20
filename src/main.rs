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
use std::path::PathBuf;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{
        EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use machine::Machine;
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Debug, Parser)]
#[command(name = "tde", about = "Terminal-native desktop environment")]
struct Cli {
    /// Path to the TDE config file (TOML).
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = crate::config::load(cli.config.as_deref())?;
    enable_terminal()?;
    let result = run(config).await;
    disable_terminal()?;
    result
}

async fn run(config: crate::config::Config) -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(config, Machine::local());
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
