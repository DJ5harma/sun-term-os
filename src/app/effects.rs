use std::path::PathBuf;

use crate::app::file_manager::CreateKind;
use crate::domain::WindowId;
use crate::machine::applications::ApplicationEntry;
use crate::machine::id::MachineId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    RefreshCapabilities,
    PersistConfig,
    Terminal(TerminalEffect),
    FileManager(FileManagerEffect),
    Process(ProcessEffect),
    Machines(MachinesEffect),
    Launcher(LauncherEffect),
    TextViewer(TextViewerEffect),
    Services(ServicesEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEffect {
    Start(WindowId),
    StartSsh(WindowId, Vec<String>),
    Stop(WindowId),
    Write(WindowId, Vec<u8>),
    StartWithCommand(WindowId, String),
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
    Kill(WindowId, u32),
    KillForce(WindowId, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachinesEffect {
    Connect(String),
    Disconnect(String),
    PersistConfig,
    TrustHostKey {
        profile_id: String,
        host: String,
        port: u16,
        fingerprint: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherEffect {
    Discover(MachineId),
    Launch(ApplicationEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextViewerEffect {
    Read(WindowId, PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServicesEffect {
    List(MachineId),
    Start(MachineId, String),
    Stop(MachineId, String),
    Restart(MachineId, String),
}
