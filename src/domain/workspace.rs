use super::{Window, WindowId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: usize,
    pub windows: Vec<Window>,
    pub focused_window: Option<WindowId>,
}
