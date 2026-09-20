#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Refresh,
    ToggleCommandPalette,
    CloseCommandPalette,
    MovePaletteUp,
    MovePaletteDown,
    ExecutePaletteSelection,
    FocusNext,
    FocusPrevious,
    SwitchWorkspace(usize),
}
