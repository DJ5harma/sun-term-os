use crate::domain::{ApplicationKind, WindowId};
use crate::machine::{DirectoryListing, ProcessInfo, SystemSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Shell(ShellAction),
    Palette(PaletteAction),
    FileManager(FileManagerAction),
    Process(ProcessAction),
    Terminal(TerminalAction),
    Settings(SettingsAction),
    Async(AsyncAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction {
    Quit,
    Refresh,
    OpenApplication(ApplicationKind),
    CloseWindow,
    FocusWindowSlot(u8),
    BeginWindowPick,
    CancelWindowPick,
    FocusWindow(WindowId),
    MinimizeWindow,
    ToggleMaximizeWindow,
    SwitchWorkspace(usize),
    ToggleInputDebug,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteAction {
    ToggleLauncher,
    CloseLauncher,
    MoveLauncherUp,
    MoveLauncherDown,
    ExecuteLauncherSelection,
    PaletteQueryPush(char),
    PaletteQueryBackspace,
    RunPaletteShell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileManagerAction {
    MoveFileSelection(i32),
    OpenSelectedEntry,
    ToggleFileManagerHidden,
    ReloadFileManager,
    SelectFileManagerRow(WindowId, usize),
    FileManagerTogglePane,
    FileManagerGoHome,
    FileManagerGoBack,
    FileManagerGoUp,
    FileManagerPageScroll(i32),
    FileManagerSetSort(crate::app::SortColumn),
    SelectFileManagerPlace(WindowId, usize),
    FileManagerOpenInTerminal,
    FileManagerRequestDelete,
    FileManagerConfirmDelete,
    FileManagerCancelDialog,
    FileManagerBeginRename,
    FileManagerBeginCreate(crate::app::file_manager::CreateKind),
    FileManagerDialogPush(char),
    FileManagerDialogBackspace,
    FileManagerDialogCommit,
    FileManagerBeginGoToPath,
    FileManagerOpenWithSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    MoveProcessSelection(i32),
    ProcessPageScroll(i32),
    ProcessFilterBegin,
    ProcessFilterPush(char),
    ProcessFilterBackspace,
    ProcessFilterEnd,
    ProcessKillSelected,
    ProcessKillForceSelected,
    ProcessSetSort(crate::app::process_manager::ProcessSortColumn),
    SelectProcessRow(WindowId, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalAction {
    ScrollOutput(i32),
    ScrollToEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    MoveSelection(i32),
    ActivateRow,
    AdjustRefresh(i32),
    Save,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum AsyncAction {
    SystemInfoReady(Result<SystemSnapshot, String>),
    ProcessesReady(Result<Vec<ProcessInfo>, String>),
    DirectoryReady(WindowId, Result<DirectoryListing, String>),
}
