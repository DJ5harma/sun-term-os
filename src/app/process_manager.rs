use crate::{app::Loadable, machine::ProcessInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessSortColumn {
    #[default]
    Cpu,
    Memory,
    Name,
    Pid,
}

impl ProcessSortColumn {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "memory",
            Self::Name => "name",
            Self::Pid => "PID",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessManagerState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub filter_active: bool,
    pub visible_rows: usize,
    pub listing: Loadable<Vec<ProcessInfo>>,
    pub sort: ProcessSortColumn,
    pub last_refreshed_at: Option<u64>,
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
            sort: ProcessSortColumn::default(),
            last_refreshed_at: None,
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

pub fn matching_indices(
    processes: &[ProcessInfo],
    filter: &str,
    sort: ProcessSortColumn,
) -> Vec<usize> {
    let mut indices: Vec<usize> = if filter.is_empty() {
        processes
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect()
    } else {
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
    };
    indices.sort_by(|left, right| {
        let left = &processes[*left];
        let right = &processes[*right];
        match sort {
            ProcessSortColumn::Cpu => right
                .cpu_percent
                .total_cmp(&left.cpu_percent)
                .then_with(|| left.pid.cmp(&right.pid)),
            ProcessSortColumn::Memory => right
                .memory_bytes
                .cmp(&left.memory_bytes)
                .then_with(|| left.pid.cmp(&right.pid)),
            ProcessSortColumn::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            ProcessSortColumn::Pid => left.pid.cmp(&right.pid),
        }
    });
    indices
}

pub fn selected_process<'a>(
    processes: &'a [ProcessInfo],
    filter: &str,
    sort: ProcessSortColumn,
    selected_index: usize,
) -> Option<&'a ProcessInfo> {
    matching_indices(processes, filter, sort)
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
        assert_eq!(
            matching_indices(&processes, "fire", ProcessSortColumn::Cpu),
            vec![1]
        );
        assert_eq!(
            matching_indices(&processes, "42", ProcessSortColumn::Cpu),
            vec![1]
        );
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
