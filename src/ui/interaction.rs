//! Pointer targets registered during render — same frame, same coordinates, layered like input routing.
//!
//! Register every clickable region with an [InteractionLayer]. Resolution prefers
//! **Modal** (launcher) over **Content** (focused app). Top/bottom chrome uses
//! [geometry::shell_action_at] instead of this map.

use ratatui::layout::{Position, Rect};

use crate::actions::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractionLayer {
    Content = 0,
    Modal = 1,
}

#[derive(Debug, Clone, Copy)]
struct Region {
    rect: Rect,
    layer: InteractionLayer,
    action: Action,
}

#[derive(Debug, Default)]
pub struct InteractionMap {
    regions: Vec<Region>,
}

impl InteractionMap {
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    pub fn register(&mut self, layer: InteractionLayer, rect: Rect, action: Action) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        self.regions.push(Region {
            rect,
            layer,
            action,
        });
    }

    /// Highest layer wins; within a layer, last registration wins (topmost widget).
    pub fn resolve(&self, column: u16, row: u16) -> Option<Action> {
        let position = Position::new(column, row);
        self.regions
            .iter()
            .rev()
            .filter(|region| region.rect.contains(position))
            .max_by_key(|region| region.layer)
            .map(|region| region.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_beats_content_on_overlap() {
        let mut map = InteractionMap::default();
        let rect = Rect::new(0, 0, 10, 1);
        map.register(InteractionLayer::Content, rect, Action::Refresh);
        map.register(InteractionLayer::Modal, rect, Action::ToggleLauncher);
        assert_eq!(map.resolve(5, 0), Some(Action::ToggleLauncher));
    }
}
