use crate::domain::{ApplicationKind, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Refresh,
    ToggleLauncher,
    CloseLauncher,
    MoveLauncherUp,
    MoveLauncherDown,
    ExecuteLauncherSelection,
    OpenApplication(ApplicationKind),
    CloseWindow,
    /// Focus the Nth window on the current workspace (1–9, left-to-right in the bar).
    FocusWindowSlot(u8),
    /// Wait for a follow-up digit (prefix chord — works inside host terminals).
    BeginWindowPick,
    CancelWindowPick,
    FocusWindow(WindowId),
    MinimizeWindow,
    ToggleMaximizeWindow,
    SwitchWorkspace(usize),
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
    ToggleInputDebug,
}
