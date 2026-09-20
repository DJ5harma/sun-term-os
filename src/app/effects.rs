use std::path::PathBuf;

use crate::app::file_manager::CreateKind;
use crate::domain::WindowId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    RefreshCapabilities,
    Terminal(TerminalEffect),
    FileManager(FileManagerEffect),
    Process(ProcessEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEffect {
    Start(WindowId),
    Stop(WindowId),
    Write(WindowId, Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileManagerEffect {
    ReadDirectory(WindowId, PathBuf),
    DeletePath(WindowId, PathBuf),
    RenamePath(WindowId, PathBuf, PathBuf),
    CreateEntry(WindowId, PathBuf, CreateKind),
    OpenWithSystem(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEffect {
    Kill(u32),
    KillForce(u32),
}
