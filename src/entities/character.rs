use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Character {
    value: char,
}

impl From<char> for Character {
    fn from(value: char) -> Self {
        Self { value }
    }
}

impl From<Character> for char {
    fn from(value: Character) -> Self {
        value.value
    }
}

impl From<Character> for String {
    fn from(value: Character) -> Self {
        value.value.to_string()
    }
}

#[derive(Error, Debug, Clone)]
pub struct StringToCharacterError;

impl fmt::Display for StringToCharacterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Character must be single character")
    }
}

impl TryFrom<&str> for Character {
    type Error = StringToCharacterError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let &[character] = value.chars().collect::<Vec<_>>().as_slice() {
            Ok(Self { value: character })
        } else {
            Err(StringToCharacterError)
        }
    }
}
