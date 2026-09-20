use std::path::PathBuf;

use crate::domain::WindowId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    RefreshCapabilities,
    StartTerminal(WindowId),
    StopTerminal(WindowId),
    ReadDirectory(WindowId, PathBuf),
}
