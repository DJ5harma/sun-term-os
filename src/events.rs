use crossterm::event::KeyEvent;

use crate::machine::{ProcessInfo, SystemSnapshot};

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    SystemInfoLoaded(Result<SystemSnapshot, String>),
    ProcessesLoaded(Result<Vec<ProcessInfo>, String>),
}
