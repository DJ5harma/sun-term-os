use crate::{
    actions::{Action, AsyncAction},
    app::{
        effects::Effect,
        reducer,
        state::{AppState, Loadable},
    },
    apps,
    machine::Machine,
    terminal::TerminalManager,
};

pub struct EffectExecutor {
    pub machine: Machine,
    pub terminal_manager: TerminalManager,
}

impl EffectExecutor {
    pub async fn run(&mut self, state: &mut AppState, effects: Vec<Effect>) {
        for effect in effects {
            apps::run_effect(self, state, effect).await;
        }
    }

    pub(crate) async fn read_directory(
        &mut self,
        state: &mut AppState,
        window_id: u64,
        path: std::path::PathBuf,
    ) {
        if let Some(manager) = state.file_manager_mut(window_id) {
            manager.listing = Loadable::Loading;
        }
        let show_hidden = state
            .file_manager(window_id)
            .map(|manager| manager.show_hidden)
            .unwrap_or(false);
        let result = self
            .machine
            .filesystem
            .list_directory(&path, show_hidden)
            .await
            .map_err(|error| error.to_string());
        reducer::reduce(
            state,
            Action::Async(AsyncAction::DirectoryReady(window_id, result)),
        );
    }

    pub(crate) async fn refresh_capabilities(&mut self, state: &mut AppState) {
        let (system, processes) = tokio::join!(
            self.machine.system.snapshot(),
            self.machine.processes.processes()
        );
        reducer::reduce(
            state,
            Action::Async(AsyncAction::SystemInfoReady(
                system.map_err(|error| error.to_string()),
            )),
        );
        reducer::reduce(
            state,
            Action::Async(AsyncAction::ProcessesReady(
                processes.map_err(|error| error.to_string()),
            )),
        );
        state.status = "Live · refreshed just now".to_owned();
    }
}
