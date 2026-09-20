mod actions;
mod app;
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
    event::EnableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use machine::local::{LocalFilesystemProvider, LocalProcessProvider, LocalSystemInfoProvider};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    enable_terminal()?;
    let result = run().await;
    disable_terminal()?;
    result
}

async fn run() -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(
        Arc::new(LocalSystemInfoProvider),
        Arc::new(LocalProcessProvider),
        Arc::new(LocalFilesystemProvider),
    );
    app.run(&mut terminal).await
}

fn enable_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn disable_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}
