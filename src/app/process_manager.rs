use crate::{app::Loadable, machine::ProcessInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessManagerState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub filter_active: bool,
    pub visible_rows: usize,
    pub listing: Loadable<Vec<ProcessInfo>>,
}

impl ProcessManagerState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            filter: String::new(),
            filter_active: false,
            visible_rows: 1,
            listing: Loadable::Loading,
        }
    }

    pub fn clamp_selection(&mut self, match_count: usize) {
        if match_count == 0 {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }
        self.selected_index = self.selected_index.min(match_count - 1);
        if self.visible_rows == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        if self.selected_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_index + 1 - self.visible_rows;
        }
    }
}

pub fn matching_indices(processes: &[ProcessInfo], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        return processes
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();
    }
    let needle = filter.to_lowercase();
    processes
        .iter()
        .enumerate()
        .filter(|(_, process)| {
            process.name.to_lowercase().contains(&needle)
                || process.pid.to_string().contains(&needle)
        })
        .map(|(index, _)| index)
        .collect()
}

pub fn selected_process<'a>(
    processes: &'a [ProcessInfo],
    filter: &str,
    selected_index: usize,
) -> Option<&'a ProcessInfo> {
    matching_indices(processes, filter)
        .get(selected_index)
        .and_then(|index| processes.get(*index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<ProcessInfo> {
        vec![
            ProcessInfo {
                pid: 1,
                name: "init".to_owned(),
                cpu_percent: 0.0,
                memory_bytes: 0,
                state: "Run".to_owned(),
            },
            ProcessInfo {
                pid: 42,
                name: "firefox".to_owned(),
                cpu_percent: 5.0,
                memory_bytes: 0,
                state: "Sleep".to_owned(),
            },
        ]
    }

    #[test]
    fn filter_matches_name_or_pid() {
        let processes = sample();
        assert_eq!(matching_indices(&processes, "fire"), vec![1]);
        assert_eq!(matching_indices(&processes, "42"), vec![1]);
    }

    #[test]
    fn clamp_keeps_selection_visible() {
        let mut manager = ProcessManagerState::new();
        manager.visible_rows = 3;
        manager.selected_index = 5;
        manager.clamp_selection(10);
        assert_eq!(manager.scroll_offset, 3);
    }
}
