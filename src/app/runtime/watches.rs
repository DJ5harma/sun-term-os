use std::{collections::HashMap, path::PathBuf, sync::mpsc, time::Duration};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};

use crate::domain::WindowId;

pub struct DirectoryWatchHub {
    sender: mpsc::Sender<WindowId>,
    receiver: mpsc::Receiver<WindowId>,
    watchers: HashMap<WindowId, RecommendedWatcher>,
    last_fired: HashMap<WindowId, std::time::Instant>,
}

impl DirectoryWatchHub {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            watchers: HashMap::new(),
            last_fired: HashMap::new(),
        }
    }

    pub fn poll_ready(&mut self) -> Option<WindowId> {
        while let Ok(window_id) = self.receiver.try_recv() {
            let now = std::time::Instant::now();
            let debounce = Duration::from_millis(250);
            if self
                .last_fired
                .get(&window_id)
                .is_some_and(|last| now.duration_since(*last) < debounce)
            {
                continue;
            }
            self.last_fired.insert(window_id, now);
            return Some(window_id);
        }
        None
    }

    pub fn watch_directory(&mut self, window_id: WindowId, path: PathBuf) {
        self.watchers.remove(&window_id);
        let sender = self.sender.clone();
        let Ok(mut watcher) = RecommendedWatcher::new(
            move |result: Result<Event>| {
                if let Ok(event) = result
                    && matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    )
                {
                    let _ = sender.send(window_id);
                }
            },
            Config::default(),
        ) else {
            return;
        };
        if watcher
            .watch(path.as_path(), RecursiveMode::NonRecursive)
            .is_ok()
        {
            self.watchers.insert(window_id, watcher);
        }
    }

}
