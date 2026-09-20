use async_trait::async_trait;

use crate::machine::CapabilityError;
use crate::machine::applications::desktop::parse_remote_discover_output;
use crate::machine::applications::{ApplicationEntry, ApplicationProvider};

use super::exec::ExecSession;

const REMOTE_DISCOVER_SCRIPT: &str = r#"sh -c 'for dir in /usr/share/applications /usr/local/share/applications "$HOME/.local/share/applications"; do
  [ -d "$dir" ] || continue
  for f in "$dir"/*.desktop; do
    [ -f "$f" ] || continue
    printf "@PATH@%s\n" "$f"
    grep -E "^(Name|Comment|Exec|Terminal|NoDisplay|Hidden)=" "$f" 2>/dev/null || true
    printf "@END@\n"
  done
done'"#;

pub struct RemoteApplicationProvider {
    exec: ExecSession,
}

impl RemoteApplicationProvider {
    pub fn new(exec: ExecSession) -> Self {
        Self { exec }
    }
}

#[async_trait]
impl ApplicationProvider for RemoteApplicationProvider {
    async fn discover(&self) -> Result<Vec<ApplicationEntry>, CapabilityError> {
        let output = self
            .exec
            .run(REMOTE_DISCOVER_SCRIPT)
            .await
            .map_err(CapabilityError::Failed)?;
        Ok(parse_remote_discover_output(&output))
    }
}
