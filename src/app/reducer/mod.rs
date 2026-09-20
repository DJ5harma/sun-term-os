mod async_completion;
mod file_manager;
mod home_screen;
mod launcher;
mod machines;
mod palette;
mod process;
mod services;
mod settings;
mod shell;
mod terminal;
mod text_viewer;

use crate::{actions::Action, app::AppState};

use super::effects::Effect;

pub(crate) use shell::{open_application, open_application_effects};

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Shell(action) => shell::reduce(state, action),
        Action::Palette(action) => palette::reduce(state, action),
        Action::FileManager(action) => file_manager::reduce(state, action),
        Action::Process(action) => process::reduce(state, action),
        Action::Terminal(action) => terminal::reduce(state, action),
        Action::Settings(action) => settings::reduce(state, action),
        Action::Machines(action) => machines::reduce(state, action),
        Action::Launcher(action) => launcher::reduce(state, action),
        Action::HomeScreen(action) => home_screen::reduce(state, action),
        Action::TextViewer(action) => text_viewer::reduce(state, action),
        Action::Services(action) => services::reduce(state, action),
        Action::Async(action) => async_completion::reduce(state, action),
    }
}

#[cfg(test)]
mod tests;
