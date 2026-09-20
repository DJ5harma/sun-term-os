use crate::{
    actions::{Action, AsyncAction},
    app::{
        effects::Effect,
        reducer,
        runtime::watches::DirectoryWatchHub,
        state::{AppState, Loadable},
    },
    apps,
    machine::{MachineId, MachineRegistry},
    terminal::TerminalManager,
};

pub struct EffectExecutor {
    pub registry: MachineRegistry,
    pub terminal_manager: TerminalManager,
    pub directory_watches: DirectoryWatchHub,
}

impl EffectExecutor {
    pub fn machine_for_window(
        &self,
        state: &AppState,
        window_id: WindowId,
    ) -> Option<&crate::machine::Machine> {
        let machine_id = state
            .window_machine_id(window_id)
            .unwrap_or_else(|| state.active_machine_id.clone());
        self.registry.machine(&machine_id)
    }

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
        let result = match self.machine_for_window(state, window_id) {
            Some(machine) => machine
                .filesystem
                .list_directory(&path, show_hidden)
                .await
                .map_err(|error| error.to_string()),
            None => {
                let machine_id = state
                    .window_machine_id(window_id)
                    .unwrap_or_else(|| state.active_machine_id.clone());
                Err(not_connected_message(&machine_id))
            }
        };
        let ok = result.is_ok();
        reducer::reduce(
            state,
            Action::Async(AsyncAction::DirectoryReady(window_id, result)),
        );
        if ok {
            let machine_id = state
                .window_machine_id(window_id)
                .unwrap_or_else(|| state.active_machine_id.clone());
            if machine_id.is_local() {
                self.directory_watches.watch_directory(window_id, path);
            }
        }
    }

    pub(crate) async fn refresh_machine(&mut self, state: &mut AppState, machine_id: MachineId) {
        let machine = match self.registry.require_machine(&machine_id) {
            Ok(machine) => machine,
            Err(message) => {
                reducer::reduce(
                    state,
                    Action::Async(AsyncAction::SystemInfoReady(
                        machine_id.clone(),
                        Err(message.clone()),
                    )),
                );
                reducer::reduce(
                    state,
                    Action::Async(AsyncAction::ProcessesReady(machine_id, Err(message))),
                );
                return;
            }
        };
        let (system, processes) =
            tokio::join!(machine.system.snapshot(), machine.processes.processes());
        reducer::reduce(
            state,
            Action::Async(AsyncAction::SystemInfoReady(
                machine_id.clone(),
                system.map_err(|error| error.to_string()),
            )),
        );
        reducer::reduce(
            state,
            Action::Async(AsyncAction::ProcessesReady(
                machine_id,
                processes.map_err(|error| error.to_string()),
            )),
        );
    }

    pub(crate) async fn refresh_capabilities(&mut self, state: &mut AppState) {
        let mut targets = vec![MachineId::Local, state.active_machine_id.clone()];
        for workspace in &state.workspaces {
            for window in &workspace.windows {
                if !targets.contains(&window.machine_id) {
                    targets.push(window.machine_id.clone());
                }
            }
        }
        for machine_id in targets {
            if machine_id.is_local()
                || matches!(
                    self.registry.connection_state(&machine_id),
                    crate::machine::ConnectionState::Connected
                )
            {
                self.refresh_machine(state, machine_id).await;
            }
        }
        state.status = "Live · refreshed just now".to_owned();
    }
}

use crate::domain::WindowId;

fn not_connected_message(machine_id: &MachineId) -> String {
    match machine_id {
        MachineId::Local => "local machine unavailable".to_owned(),
        MachineId::Named(name) => format!("not connected to {name}"),
    }
}
