use serde::{Deserialize, Serialize};

use crate::domain::ApplicationKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeConfig {
    /// Built-in apps shown first on the empty desktop, in this order.
    #[serde(default = "default_pinned")]
    pub pinned: Vec<ApplicationKind>,
}

fn default_pinned() -> Vec<ApplicationKind> {
    vec![
        ApplicationKind::Terminal,
        ApplicationKind::FileManager,
        ApplicationKind::Machines,
        ApplicationKind::Launcher,
    ]
}

impl Default for HomeConfig {
    fn default() -> Self {
        Self {
            pinned: default_pinned(),
        }
    }
}
