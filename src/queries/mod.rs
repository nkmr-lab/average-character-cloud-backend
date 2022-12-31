mod character_config_query;
mod character_config_seed_query;
mod figure_record_query;
pub use character_config_query::{
    CharacterConfigByCharacterLoader, CharacterConfigByCharacterLoaderParams,
    CharacterConfigsLoader, CharacterConfigsLoaderParams,
};
pub use character_config_seed_query::{
    CharacterConfigSeedByCharacterLoader, CharacterConfigSeedByCharacterLoaderParams,
    CharacterConfigSeedsLoader, CharacterConfigSeedsLoaderParams,
};
pub use figure_record_query::{
    FigureRecordByIdLoader, FigureRecordByIdLoaderParams, FigureRecordsByCharacterLoader,
    FigureRecordsByCharacterLoaderParams,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserType {
    Myself,
    Other,
}
