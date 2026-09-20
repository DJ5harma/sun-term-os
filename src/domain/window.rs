pub type WindowId = u64;

use super::ApplicationKind;
use crate::machine::MachineId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub application: ApplicationKind,
    pub state: WindowState,
    pub machine_id: MachineId,
}
