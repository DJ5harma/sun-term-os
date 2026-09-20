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
    FocusNextWindow,
    FocusPreviousWindow,
    FocusWindow(WindowId),
    MinimizeWindow,
    ToggleMaximizeWindow,
    SwitchWorkspace(usize),
    MoveFileSelection(i32),
    OpenSelectedEntry,
    FileManagerParent,
    ToggleFileManagerHidden,
    ReloadFileManager,
    SelectFileManagerRow(WindowId, usize),
}
