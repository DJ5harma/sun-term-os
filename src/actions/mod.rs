use crate::domain::{ApplicationKind, WindowId};
use crate::machine::id::MachineId;
use crate::machine::{DirectoryListing, ProcessInfo, SystemSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Shell(ShellAction),
    Palette(PaletteAction),
    FileManager(FileManagerAction),
    Process(ProcessAction),
    Terminal(TerminalAction),
    Settings(SettingsAction),
    Machines(MachinesAction),
    Launcher(LauncherAction),
    TextViewer(TextViewerAction),
    HomeScreen(HomeScreenAction),
    Services(ServicesAction),
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
    FileManagerOpenInViewer,
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
    SystemInfoReady(MachineId, Result<SystemSnapshot, String>),
    ProcessesReady(MachineId, Result<Vec<ProcessInfo>, String>),
    DirectoryReady(WindowId, Result<DirectoryListing, String>),
    LauncherReady(
        MachineId,
        Result<Vec<crate::machine::applications::ApplicationEntry>, String>,
    ),
    TextFileReady(WindowId, Result<String, String>),
    ServicesReady(
        MachineId,
        Result<Vec<crate::machine::services::ServiceInfo>, String>,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachinesAction {
    MoveSelection(i32),
    SetActiveMachine,
    ConnectSelected,
    DisconnectSelected,
    BeginAddProfile,
    BeginEditProfile,
    DeleteSelected,
    AcceptHostKey,
    RejectHostKey,
    DialogPush(char),
    DialogBackspace,
    DialogCommit,
    CancelDialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherAction {
    MoveSelection(i32),
    LauncherPageScroll(i32),
    LauncherFilterBegin,
    LauncherFilterPush(char),
    LauncherFilterBackspace,
    LauncherFilterEnd,
    LaunchSelected,
    ToggleFavorite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeScreenAction {
    MoveSelection(i32),
    MoveRow(i32),
    PageScroll(i32),
    ActivateSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextViewerAction {
    Scroll(i32),
    PageScroll(i32),
    BeginOpenPath,
    DialogPush(char),
    DialogBackspace,
    DialogCommit,
    DialogCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicesAction {
    MoveSelection(i32),
    ServicesPageScroll(i32),
    ServicesFilterBegin,
    ServicesFilterPush(char),
    ServicesFilterBackspace,
    ServicesFilterEnd,
    ServiceStartSelected,
    ServiceStopSelected,
    ServiceRestartSelected,
}
