pub(crate) mod effects;
pub mod file_manager;
pub mod palette;
pub mod process_manager;
pub(crate) mod reducer;
pub mod runtime;
pub mod state;

pub use crate::domain::{ApplicationKind, Window, WindowState};
pub use runtime::AppRuntime;
pub use state::{AppState, FileManagerFocus, Loadable, SortColumn, TerminalStatus};

use anyhow::Result;
use ratatui::DefaultTerminal;

use crate::{config::Config, machine::Machine};

/// Application entry: owns runtime loop and effect execution.
pub struct App {
    runtime: AppRuntime,
}

impl App {
    pub fn new(config: Config, machine: Machine) -> Self {
        Self {
            runtime: AppRuntime::new(config, machine),
        }
    }

    pub async fn run(self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.runtime.run(terminal).await
    }
}
