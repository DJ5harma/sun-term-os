/// Identifier for a built-in application window.
///
/// When adding a variant, register the app in [`crate::apps`] (`BUILT_INS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationKind {
    Terminal,
    FileManager,
    SystemInfo,
    Processes,
}
