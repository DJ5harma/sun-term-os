use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{bindings, normalize::normalize_key_event};

pub fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
    let key = normalize_key_event(key);
    if bindings::global_consumes(key) {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && let KeyCode::Char(character) = key.code
    {
        let character = character.to_ascii_lowercase();
        if character.is_ascii_lowercase() {
            return Some(vec![character as u8 - b'a' + 1]);
        }
    }
    let bytes = match key.code {
        KeyCode::Char(character) => character.to_string().into_bytes(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        _ => return None,
    };
    Some(bytes)
}
