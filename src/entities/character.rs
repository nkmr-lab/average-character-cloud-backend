use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Character(char);

impl From<char> for Character {
    fn from(value: char) -> Self {
        Self(value)
    }
}

impl From<Character> for char {
    fn from(value: Character) -> Self {
        value.0
    }
}

impl From<Character> for String {
    fn from(value: Character) -> Self {
        value.0.to_string()
    }
}

#[derive(Error, Debug, Clone)]
pub enum CharacterTryFromError {
    #[error("Character must be single character")]
    NotSingleCharacter,
}

impl TryFrom<&str> for Character {
    type Error = CharacterTryFromError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let &[character] = value.chars().collect::<Vec<_>>().as_slice() {
            Ok(Self(character))
        } else {
            Err(CharacterTryFromError::NotSingleCharacter)
        }
    }
}
