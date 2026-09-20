#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    RefreshCapabilities,
    StartTerminal(crate::domain::WindowId),
    StopTerminal(crate::domain::WindowId),
}
