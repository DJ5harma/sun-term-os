//! Normalize key events so bindings match across terminals (kitty / xterm / VTE).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Apply before any input layer matching.
pub fn normalize_key_event(key: KeyEvent) -> KeyEvent {
    if !key.is_press() && !key.is_repeat() {
        return key;
    }

    let key = normalize_c0_control(key);
    normalize_bracket_aliases(key)
}

fn normalize_c0_control(key: KeyEvent) -> KeyEvent {
    if let KeyCode::Char(character) = key.code {
        // Disambiguated C0 controls often arrive without the CONTROL modifier set.
        if character < '\x20'
            && character != '\t'
            && let Some(letter) = control_character_to_letter(character)
        {
            return KeyEvent::new(KeyCode::Char(letter), key.modifiers | KeyModifiers::CONTROL);
        }
    }
    key
}

/// VTE often reports `Alt+[` as `Alt+{` or `Alt+Shift+[`.
fn normalize_bracket_aliases(key: KeyEvent) -> KeyEvent {
    if !key.modifiers.intersects(KeyModifiers::ALT)
        || key.modifiers.intersects(KeyModifiers::CONTROL)
    {
        return key;
    }
    match key.code {
        KeyCode::Char('{') => KeyEvent::new(KeyCode::Char('['), key.modifiers),
        KeyCode::Char('[') if key.modifiers.intersects(KeyModifiers::SHIFT) => {
            KeyEvent::new(KeyCode::Char('['), key.modifiers - KeyModifiers::SHIFT)
        }
        _ => key,
    }
}

fn control_character_to_letter(character: char) -> Option<char> {
    let byte = character as u8;
    if (1..=26).contains(&byte) {
        char::from_u32((byte - 1) as u32 + b'a' as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c0_control_adds_control_modifier() {
        let raw = KeyEvent::new(KeyCode::Char('\x10'), KeyModifiers::NONE);
        let normalized = normalize_key_event(raw);
        assert_eq!(normalized.code, KeyCode::Char('p'));
        assert!(normalized.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn alt_brace_becomes_alt_bracket() {
        let raw = KeyEvent::new(KeyCode::Char('{'), KeyModifiers::ALT);
        let normalized = normalize_key_event(raw);
        assert_eq!(normalized.code, KeyCode::Char('['));
        assert!(normalized.modifiers.contains(KeyModifiers::ALT));
    }
}
