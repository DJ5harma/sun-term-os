mod async_completion;
mod file_manager;
mod palette;
mod process;
mod shell;

use crate::{actions::Action, app::AppState};

use super::effects::Effect;

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Shell(action) => shell::reduce(state, action),
        Action::Palette(action) => palette::reduce(state, action),
        Action::FileManager(action) => file_manager::reduce(state, action),
        Action::Process(action) => process::reduce(state, action),
        Action::Async(action) => async_completion::reduce(state, action),
    }
}

#[cfg(test)]
mod tests;
