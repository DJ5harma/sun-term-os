#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationKind {
    Terminal,
    FileManager,
    SystemInfo,
    Processes,
}

impl ApplicationKind {
    pub const ALL: [Self; 4] = [
        Self::Terminal,
        Self::FileManager,
        Self::SystemInfo,
        Self::Processes,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::FileManager => "File Manager",
            Self::SystemInfo => "System Information",
            Self::Processes => "Process Manager",
        }
    }

    pub fn launcher_description(self) -> &'static str {
        match self {
            Self::Terminal => "Run commands and manage shells",
            Self::FileManager => "Browse files and folders",
            Self::SystemInfo => "Inspect this machine",
            Self::Processes => "View running processes",
        }
    }
}
