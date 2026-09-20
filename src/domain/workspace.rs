use super::{Window, WindowId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: usize,
    pub windows: Vec<Window>,
    pub focused_window: Option<WindowId>,
    /// When true, the home screen is shown while windows stay open (Show desktop).
    pub show_desktop: bool,
    pub show_desktop_restore_focus: Option<WindowId>,
}
