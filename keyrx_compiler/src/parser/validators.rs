use crate::error::ParseError;
use keyrx_core::config::{Condition, KeyCode};

pub const PHYSICAL_MODIFIERS: &[&str] = &[
    "LShift", "RShift", "LCtrl", "RCtrl", "LAlt", "RAlt", "LMeta", "RMeta",
];

pub fn parse_virtual_key(s: &str) -> Result<KeyCode, ParseError> {
    if !s.starts_with("VK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "virtual key".to_string(),
        });
    }
    parse_key_name(&s[3..])
}

pub fn parse_modifier_id(s: &str) -> Result<u8, ParseError> {
    if !s.starts_with("MD_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom modifier".to_string(),
        });
    }
    let id_part = &s[3..];
    if PHYSICAL_MODIFIERS.contains(&id_part) {
        return Err(ParseError::PhysicalModifierInMD {
            name: id_part.to_string(),
        });
    }
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "MD_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom modifier ID".to_string(),
    })?;
    if id > 0xFE {
        return Err(ParseError::ModifierIdOutOfRange { got: id, max: 0xFE });
    }
    Ok(id as u8)
}

pub fn parse_lock_id(s: &str) -> Result<u8, ParseError> {
    if !s.starts_with("LK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom lock".to_string(),
        });
    }
    let id_part = &s[3..];
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "LK_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom lock ID".to_string(),
    })?;
    if id > 0xFE {
        return Err(ParseError::LockIdOutOfRange { got: id, max: 0xFE });
    }
    Ok(id as u8)
}

pub fn parse_condition_string(s: &str) -> Result<Condition, ParseError> {
    if s.starts_with("MD_") {
        let id = parse_modifier_id(s)?;
        Ok(Condition::ModifierActive(id))
    } else if s.starts_with("LK_") {
        let id = parse_lock_id(s)?;
        Ok(Condition::LockActive(id))
    } else {
        Err(ParseError::InvalidPrefix {
            expected: "MD_XX or LK_XX".to_string(),
            got: s.to_string(),
            context: "condition".to_string(),
        })
    }
}

pub fn parse_key_name(name: &str) -> Result<KeyCode, ParseError> {
    match name {
        "A" => Ok(KeyCode::A),
        "B" => Ok(KeyCode::B),
        "C" => Ok(KeyCode::C),
        "D" => Ok(KeyCode::D),
        "E" => Ok(KeyCode::E),
        "F" => Ok(KeyCode::F),
        "G" => Ok(KeyCode::G),
        "H" => Ok(KeyCode::H),
        "I" => Ok(KeyCode::I),
        "J" => Ok(KeyCode::J),
        "K" => Ok(KeyCode::K),
        "L" => Ok(KeyCode::L),
        "M" => Ok(KeyCode::M),
        "N" => Ok(KeyCode::N),
        "O" => Ok(KeyCode::O),
        "P" => Ok(KeyCode::P),
        "Q" => Ok(KeyCode::Q),
        "R" => Ok(KeyCode::R),
        "S" => Ok(KeyCode::S),
        "T" => Ok(KeyCode::T),
        "U" => Ok(KeyCode::U),
        "V" => Ok(KeyCode::V),
        "W" => Ok(KeyCode::W),
        "X" => Ok(KeyCode::X),
        "Y" => Ok(KeyCode::Y),
        "Z" => Ok(KeyCode::Z),
        "Num0" | "0" => Ok(KeyCode::Num0),
        "Num1" | "1" => Ok(KeyCode::Num1),
        "Num2" | "2" => Ok(KeyCode::Num2),
        "Num3" | "3" => Ok(KeyCode::Num3),
        "Num4" | "4" => Ok(KeyCode::Num4),
        "Num5" | "5" => Ok(KeyCode::Num5),
        "Num6" | "6" => Ok(KeyCode::Num6),
        "Num7" | "7" => Ok(KeyCode::Num7),
        "Num8" | "8" => Ok(KeyCode::Num8),
        "Num9" | "9" => Ok(KeyCode::Num9),
        "F1" => Ok(KeyCode::F1),
        "F2" => Ok(KeyCode::F2),
        "F3" => Ok(KeyCode::F3),
        "F4" => Ok(KeyCode::F4),
        "F5" => Ok(KeyCode::F5),
        "F6" => Ok(KeyCode::F6),
        "F7" => Ok(KeyCode::F7),
        "F8" => Ok(KeyCode::F8),
        "F9" => Ok(KeyCode::F9),
        "F10" => Ok(KeyCode::F10),
        "F11" => Ok(KeyCode::F11),
        "F12" => Ok(KeyCode::F12),
        "LShift" => Ok(KeyCode::LShift),
        "RShift" => Ok(KeyCode::RShift),
        "LCtrl" => Ok(KeyCode::LCtrl),
        "RCtrl" => Ok(KeyCode::RCtrl),
        "LAlt" => Ok(KeyCode::LAlt),
        "RAlt" => Ok(KeyCode::RAlt),
        "LMeta" => Ok(KeyCode::LMeta),
        "RMeta" => Ok(KeyCode::RMeta),
        "Escape" | "Esc" => Ok(KeyCode::Escape),
        "Enter" | "Return" => Ok(KeyCode::Enter),
        "Backspace" => Ok(KeyCode::Backspace),
        "Tab" => Ok(KeyCode::Tab),
        "Space" => Ok(KeyCode::Space),
        "CapsLock" => Ok(KeyCode::CapsLock),
        "NumLock" => Ok(KeyCode::NumLock),
        "ScrollLock" => Ok(KeyCode::ScrollLock),
        "PrintScreen" => Ok(KeyCode::PrintScreen),
        "Pause" => Ok(KeyCode::Pause),
        "Insert" | "Ins" => Ok(KeyCode::Insert),
        "Delete" | "Del" => Ok(KeyCode::Delete),
        "Home" => Ok(KeyCode::Home),
        "End" => Ok(KeyCode::End),
        "PageUp" => Ok(KeyCode::PageUp),
        "PageDown" => Ok(KeyCode::PageDown),
        "Left" => Ok(KeyCode::Left),
        "Right" => Ok(KeyCode::Right),
        "Up" => Ok(KeyCode::Up),
        "Down" => Ok(KeyCode::Down),
        _ => Err(ParseError::SyntaxError {
            file: std::path::PathBuf::new(),
            line: 0,
            column: 0,
            message: format!("Unknown key name: {}", name),
        }),
    }
}
