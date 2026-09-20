//! Mouse hit regions registered during render so coordinates always match pixels on screen.

use ratatui::layout::{Position, Rect};

use crate::actions::Action;

#[derive(Debug, Default)]
pub struct HitMap {
    regions: Vec<(Rect, Action)>,
}

impl HitMap {
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Register a clickable region. Later registrations win when regions overlap.
    pub fn register(&mut self, rect: Rect, action: Action) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        self.regions.push((rect, action));
    }

    pub fn hit(&self, column: u16, row: u16) -> Option<Action> {
        let position = Position::new(column, row);
        self.regions
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(position))
            .map(|(_, action)| *action)
    }
}
