use std::path::PathBuf;

use crate::app::file_manager::CreateKind;
use crate::domain::WindowId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    RefreshCapabilities,
    StartTerminal(WindowId),
    StopTerminal(WindowId),
    ReadDirectory(WindowId, PathBuf),
    WriteTerminal(WindowId, Vec<u8>),
    KillProcess(u32),
    DeletePath(WindowId, PathBuf),
    RenamePath(WindowId, PathBuf, PathBuf),
    CreateEntry(WindowId, PathBuf, CreateKind),
}
