use std::sync::Arc;

use russh::ChannelMsg;
use russh::client;
use tokio::sync::Mutex;

use super::session::SshHandler;

#[derive(Clone)]
pub struct ExecSession {
    handle: Arc<Mutex<client::Handle<SshHandler>>>,
}

impl ExecSession {
    pub fn new(handle: Arc<Mutex<client::Handle<SshHandler>>>) -> Self {
        Self { handle }
    }

    pub async fn run(&self, command: &str) -> Result<String, String> {
        let handle = self.handle.lock().await;
        let mut channel = handle
            .channel_open_session()
            .await
            .map_err(|error| error.to_string())?;
        channel
            .exec(true, command)
            .await
            .map_err(|error| error.to_string())?;
        let mut output = Vec::new();
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => output.extend_from_slice(&data),
                ChannelMsg::ExitStatus { .. } => break,
                ChannelMsg::ExitSignal { .. } => break,
                _ => {}
            }
        }
        Ok(String::from_utf8_lossy(&output).into_owned())
    }
}
