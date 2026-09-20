mod effects;
mod reducer;
pub mod state;

pub use crate::domain::{ApplicationKind, Window, WindowState};
pub use state::{AppState, Loadable};

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use ratatui::DefaultTerminal;
use tokio::time::{self, MissedTickBehavior};

use crate::{
    actions::Action,
    events::Event,
    input,
    machine::{ProcessProvider, SystemInfoProvider},
    ui,
};

use effects::Effect;

pub struct App {
    pub state: AppState,
    geometry: ui::geometry::UiGeometry,
    system_provider: Arc<dyn SystemInfoProvider>,
    process_provider: Arc<dyn ProcessProvider>,
}

impl App {
    pub fn new(
        system_provider: Arc<dyn SystemInfoProvider>,
        process_provider: Arc<dyn ProcessProvider>,
    ) -> Self {
        Self {
            state: AppState::default(),
            geometry: ui::geometry::UiGeometry::default(),
            system_provider,
            process_provider,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut ticker = time::interval(Duration::from_secs(3));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        self.dispatch(Action::Refresh).await;

        while !self.state.should_quit {
            let size = terminal.size()?;
            self.geometry = ui::geometry::calculate(
                ratatui::layout::Rect::new(0, 0, size.width, size.height),
                &self.state,
            );
            terminal.draw(|frame| ui::render(frame, &self.state, &self.geometry))?;

            tokio::select! {
                _ = ticker.tick() => self.handle_event(Event::Tick).await,
                event = Self::read_event() => {
                    if let Some(event) = event? {
                        self.handle_event(event).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn read_event() -> Result<Option<Event>> {
        let event = tokio::task::spawn_blocking(|| -> std::io::Result<Option<Event>> {
            if event::poll(Duration::from_millis(100))? {
                Ok(Some(match event::read()? {
                    CrosstermEvent::Key(key) => Event::Key(key),
                    CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                    _ => return Ok(None),
                }))
            } else {
                Ok(None)
            }
        })
        .await??;
        Ok(event)
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                if let Some(action) = input::action_for_key(key, self.state.launcher_open) {
                    self.dispatch(action).await;
                }
            }
            Event::Mouse(mouse) => {
                if let Some(action) = input::action_for_mouse(mouse, &self.state, &self.geometry) {
                    self.dispatch(action).await;
                }
            }
            Event::SystemInfoLoaded(result) => {
                self.state.apply_event(Event::SystemInfoLoaded(result))
            }
            Event::ProcessesLoaded(result) => {
                self.state.apply_event(Event::ProcessesLoaded(result))
            }
            Event::Tick => self.dispatch(Action::Refresh).await,
        }
    }

    async fn dispatch(&mut self, action: Action) {
        let effects = reducer::reduce(&mut self.state, action);
        self.run_effects(effects).await;
    }

    async fn run_effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::RefreshCapabilities => self.refresh_capabilities().await,
            }
        }
    }

    async fn refresh_capabilities(&mut self) {
        let (system, processes) = tokio::join!(
            self.system_provider.snapshot(),
            self.process_provider.processes()
        );
        self.state.apply_event(Event::SystemInfoLoaded(
            system.map_err(|error| error.to_string()),
        ));
        self.state.apply_event(Event::ProcessesLoaded(
            processes.map_err(|error| error.to_string()),
        ));
        self.state.status = "Live · refreshed just now".to_owned();
    }
}
