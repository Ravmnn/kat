use core::fmt;
use std::io::{stdin, Read};

pub enum Character {
    ArrowLeft,
    ArrowTop,
    ArrowRight,
    ArrowBottom,

    Enter,
    Backspace,

    Normal(u8),
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            Self::ArrowLeft => "ArrowLeft".to_string(),
            Self::ArrowTop => "ArrowTop".to_string(),
            Self::ArrowRight => "ArrowRight".to_string(),
            Self::ArrowBottom => "ArrowBottom".to_string(),
            Self::Enter => "Enter".to_string(),
            Self::Backspace => "Backspace".to_string(),
            Self::Normal(ch) => format!("Character: {}", ch),
        };

        write!(f, "{}", str)
    }
}

pub fn get_arrow_character_from_byte(byte: u8) -> Option<Character> {
    match byte {
        65 => Option::Some(Character::ArrowTop),
        66 => Option::Some(Character::ArrowBottom),
        67 => Option::Some(Character::ArrowRight),
        68 => Option::Some(Character::ArrowLeft),

        _ => Option::None,
    }
}

pub fn get_special_character_from_byte(byte: u8) -> Option<Character> {
    match byte {
        13 => Option::Some(Character::Enter),
        127 => Option::Some(Character::Backspace),

        _ => Option::None,
    }
}

pub fn read_character_from_stdin() -> Option<Character> {
    let byte = read_byte_from_stdin();
    let mut character: Option<Character>;

    character = get_special_character_from_byte(byte);

    // byte read is a escape code
    if byte == 27 {
        // character '[' (91), which comes from "ESC[..."
        // ignore it
        read_byte_from_stdin();

        // read it again to get the actual character byte
        character = get_arrow_character_from_byte(read_byte_from_stdin());
    } else if character.is_none() {
        character = Option::Some(Character::Normal(byte));
    }

    return character;
}

pub fn read_byte_from_stdin() -> u8 {
    let mut buffer = [0; 1];

    stdin().read(&mut buffer).expect("Couldn't read from stdin");

    return buffer[0];
}
