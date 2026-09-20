#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationKind {
    Terminal,
    SystemInfo,
    Processes,
}

impl ApplicationKind {
    pub const ALL: [Self; 3] = [Self::Terminal, Self::SystemInfo, Self::Processes];

    pub fn title(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::SystemInfo => "System Information",
            Self::Processes => "Process Manager",
        }
    }

    pub fn launcher_description(self) -> &'static str {
        match self {
            Self::Terminal => "Run commands and manage shells",
            Self::SystemInfo => "Inspect this machine",
            Self::Processes => "View running processes",
        }
    }
}
