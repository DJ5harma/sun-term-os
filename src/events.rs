use crossterm::event::{KeyEvent, MouseEvent};

use crate::domain::WindowId;
use crate::machine::{DirectoryListing, ProcessInfo, SystemSnapshot};

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    SystemInfoLoaded(Result<SystemSnapshot, String>),
    ProcessesLoaded(Result<Vec<ProcessInfo>, String>),
    DirectoryLoaded(WindowId, Result<DirectoryListing, String>),
}
