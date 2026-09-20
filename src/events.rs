use crossterm::event::{KeyEvent, MouseEvent};

use crate::machine::{ProcessInfo, SystemSnapshot};

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    SystemInfoLoaded(Result<SystemSnapshot, String>),
    ProcessesLoaded(Result<Vec<ProcessInfo>, String>),
}
