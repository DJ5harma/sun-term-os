use crate::{
    app::{
        AppState,
        effects::Effect,
        reducer::{open_application, open_application_effects},
    },
    config::{SessionConfig, session},
    domain::ApplicationKind,
    machine::local::default_start_path,
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
    let mut effects = Vec::new();
    for (index, saved) in file.workspaces.iter().enumerate() {
        if index >= state.workspaces.len() {
            break;
        }
        state.active_workspace = index;
        for window in &saved.windows {
            let window_id = open_application(state, window.application);
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
        }
    }
    state.active_workspace = file.active_workspace.min(state.workspaces.len() - 1);
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
        workspaces: state
            .workspaces
            .iter()
            .map(|workspace| session::SessionWorkspace {
                windows: workspace
                    .windows
                    .iter()
                    .map(|window| session::SessionWindow {
                        application: window.application,
                        file_manager_path: if window.application == ApplicationKind::FileManager {
                            state
                                .file_manager(window.id)
                                .map(|manager| manager.current_path.clone())
                        } else {
                            None
                        },
                    })
                    .collect(),
            })
            .collect(),
    }
}
