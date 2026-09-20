use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    thread,
};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::domain::WindowId;

#[derive(Debug)]
pub enum TerminalEvent {
    Output { window_id: WindowId, bytes: Vec<u8> },
    Exited { window_id: WindowId },
}

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    parser: vt100::Parser,
    child: Box<dyn Child + Send + Sync>,
}

pub struct TerminalManager {
    sessions: HashMap<WindowId, Session>,
    sender: UnboundedSender<TerminalEvent>,
    pub events: UnboundedReceiver<TerminalEvent>,
}

impl Default for TerminalManager {
    fn default() -> Self {
        let (sender, events) = mpsc::unbounded_channel();
        Self {
            sessions: HashMap::new(),
            sender,
            events,
        }
    }
}

impl TerminalManager {
    pub fn open(&mut self, window_id: WindowId, columns: u16, rows: u16) -> anyhow::Result<()> {
        if self.sessions.contains_key(&window_id) {
            return Ok(());
        }
        let size = PtySize {
            rows: rows.max(2),
            cols: columns.max(2),
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = native_pty_system().openpty(size)?;
        let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_owned());
        let mut command = CommandBuilder::new(shell);
        command.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(command)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        self.spawn_reader(window_id, reader);
        self.sessions.insert(
            window_id,
            Session {
                master: pair.master,
                writer,
                parser: vt100::Parser::new(size.rows, size.cols, 0),
                child,
            },
        );
        Ok(())
    }

    pub fn write_input(&mut self, window_id: WindowId, input: &[u8]) -> anyhow::Result<()> {
        if let Some(session) = self.sessions.get_mut(&window_id) {
            session.writer.write_all(input)?;
            session.writer.flush()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, window_id: WindowId, columns: u16, rows: u16) -> anyhow::Result<()> {
        if let Some(session) = self.sessions.get_mut(&window_id) {
            let size = PtySize {
                rows: rows.max(2),
                cols: columns.max(2),
                pixel_width: 0,
                pixel_height: 0,
            };
            session.master.resize(size)?;
            session.parser.set_size(size.rows, size.cols);
        }
        Ok(())
    }

    pub fn consume_output(&mut self, window_id: WindowId, bytes: &[u8]) -> Option<String> {
        let session = self.sessions.get_mut(&window_id)?;
        session.parser.process(bytes);
        Some(session.parser.screen().contents())
    }

    pub fn close(&mut self, window_id: WindowId) {
        if let Some(mut session) = self.sessions.remove(&window_id) {
            let _ = session.child.kill();
        }
    }

    fn spawn_reader(&self, window_id: WindowId, mut reader: Box<dyn Read + Send>) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => {
                        if sender
                            .send(TerminalEvent::Output {
                                window_id,
                                bytes: buffer[..count].to_vec(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
            let _ = sender.send(TerminalEvent::Exited { window_id });
        });
    }
}
