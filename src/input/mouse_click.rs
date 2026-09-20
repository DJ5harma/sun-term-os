use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{actions::Action, domain::WindowId};

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClickTarget {
    window_id: WindowId,
    row_index: usize,
    column: u16,
    row: u16,
}

#[derive(Debug, Default)]
pub struct DoubleClickState {
    last: Option<(ClickTarget, Instant)>,
}

impl DoubleClickState {
    pub fn actions_after_primary(&mut self, mouse: &MouseEvent, primary: Action) -> Vec<Action> {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            self.last = None;
            return vec![primary];
        }

        if let Action::SelectFileManagerRow(window_id, index) = primary {
            let target = ClickTarget {
                window_id,
                row_index: index,
                column: mouse.column,
                row: mouse.row,
            };
            if self.is_double_click(target) {
                self.last = None;
                return vec![
                    Action::SelectFileManagerRow(window_id, index),
                    Action::OpenSelectedEntry,
                ];
            }
            self.last = Some((target, Instant::now()));
            return vec![primary];
        }

        self.last = None;
        vec![primary]
    }

    fn is_double_click(&self, target: ClickTarget) -> bool {
        let Some((last, at)) = self.last else {
            return false;
        };
        last == target && at.elapsed() <= DOUBLE_CLICK_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

    fn left_down(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn second_click_on_same_row_opens_entry() {
        let mut state = DoubleClickState::default();
        let mouse = left_down(10, 5);
        let select = Action::SelectFileManagerRow(1, 3);

        assert_eq!(
            state.actions_after_primary(&mouse, select),
            vec![Action::SelectFileManagerRow(1, 3)]
        );
        assert_eq!(
            state.actions_after_primary(&mouse, select),
            vec![
                Action::SelectFileManagerRow(1, 3),
                Action::OpenSelectedEntry,
            ]
        );
    }

    #[test]
    fn different_row_resets_double_click() {
        let mut state = DoubleClickState::default();
        state.actions_after_primary(&left_down(1, 1), Action::SelectFileManagerRow(1, 0));
        let actions =
            state.actions_after_primary(&left_down(2, 1), Action::SelectFileManagerRow(1, 1));
        assert_eq!(actions, vec![Action::SelectFileManagerRow(1, 1)]);
    }
}
