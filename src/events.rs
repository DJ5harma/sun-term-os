use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}
