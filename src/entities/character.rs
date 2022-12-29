use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
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
#[error("Character must be single character")]
pub struct CharacterTryFromError;

impl TryFrom<&str> for Character {
    type Error = CharacterTryFromError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let &[character] = value.chars().collect::<Vec<_>>().as_slice() {
            Ok(Self { value: character })
        } else {
            Err(CharacterTryFromError)
        }
    }
}
