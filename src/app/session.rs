use crate::{
    app::{
        AppState,
        effects::Effect,
        reducer::{open_application, open_application_effects},
    },
    config::{SessionConfig, session},
    domain::ApplicationKind,
    machine::{local::default_start_path, registry::ConnectionState},
};

pub fn restore_effects(state: &mut AppState, config: &SessionConfig) -> Vec<Effect> {
    if !config.restore_on_start {
        return Vec::new();
    }
    let path = crate::config::default_session_path();
    let file = match session::load_session(&path) {
        Ok(session) => session,
        Err(error) => {
            state.status = format!("Session not restored: {error}");
            return Vec::new();
        }
    };
    if file.workspaces.is_empty() {
        return Vec::new();
    }
    state.active_machine_id = session::decode_machine_id(file.active_machine_id.as_deref());
    let mut effects = Vec::new();
    for profile_id in &file.connected_machine_ids {
        effects.push(Effect::Machines(
            crate::app::effects::MachinesEffect::Connect(profile_id.clone()),
        ));
    }
    for (index, saved) in file.workspaces.iter().enumerate() {
        if index >= state.workspaces.len() {
            break;
        }
        state.active_workspace = index;
        for window in &saved.windows {
            let window_id = open_application(state, window.application);
            set_window_machine_id(
                state,
                window_id,
                session::decode_machine_id(window.machine_id.as_deref()),
            );
            effects.extend(open_application_effects(
                state,
                window.application,
                window_id,
            ));
            if window.application == ApplicationKind::FileManager {
                let path = window
                    .file_manager_path
                    .clone()
                    .unwrap_or_else(default_start_path);
                if let Some(manager) = state.file_manager_mut(window_id) {
                    manager.current_path = path.clone();
                }
                effects.push(crate::app::effects::Effect::FileManager(
                    crate::app::effects::FileManagerEffect::ReadDirectory(window_id, path),
                ));
            }
            if window.application == ApplicationKind::TextViewer
                && let Some(path) = window.text_viewer_path.clone()
            {
                state.init_text_viewer(window_id, path.clone());
                effects.push(crate::app::effects::Effect::TextViewer(
                    crate::app::effects::TextViewerEffect::Read(window_id, path),
                ));
            }
        }
    }
    state.active_workspace = file.active_workspace.min(state.workspaces.len() - 1);
    state.active_machine_id = session::decode_machine_id(file.active_machine_id.as_deref());
    state.status = "Restored previous session".to_owned();
    effects
}

pub fn save_if_configured(state: &AppState, config: &SessionConfig) {
    if !config.save_on_exit {
        return;
    }
    let file = capture_session(state);
    let _ = session::save_session(&crate::config::default_session_path(), &file);
}

fn capture_session(state: &AppState) -> session::SessionFile {
    session::SessionFile {
        active_workspace: state.active_workspace,
        active_machine_id: Some(session::encode_machine_id(&state.active_machine_id)),
        connected_machine_ids: state
            .machine_connections
            .iter()
            .filter(|(_, connection)| **connection == ConnectionState::Connected)
            .map(|(id, _)| id.clone())
            .collect(),
        workspaces: state
            .workspaces
            .iter()
            .map(|workspace| session::SessionWorkspace {
                windows: workspace
                    .windows
                    .iter()
                    .map(|window| session::SessionWindow {
                        application: window.application,
                        machine_id: Some(session::encode_machine_id(&window.machine_id)),
                        file_manager_path: if window.application == ApplicationKind::FileManager {
                            state
                                .file_manager(window.id)
                                .map(|manager| manager.current_path.clone())
                        } else {
                            None
                        },
                        text_viewer_path: if window.application == ApplicationKind::TextViewer {
                            state
                                .text_viewer(window.id)
                                .filter(|viewer| viewer.has_path())
                                .map(|viewer| viewer.path.clone())
                        } else {
                            None
                        },
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn set_window_machine_id(
    state: &mut AppState,
    window_id: u64,
    machine_id: crate::machine::MachineId,
) {
    if let Some(window) = state
        .workspaces
        .iter_mut()
        .flat_map(|workspace| workspace.windows.iter_mut())
        .find(|window| window.id == window_id)
    {
        window.machine_id = machine_id;
    }
}
