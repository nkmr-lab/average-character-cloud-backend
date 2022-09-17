mod character_config_query;
mod figure_record_query;
pub use character_config_query::{
    CharacterConfigByCharacterLoader, CharacterConfigByCharacterLoaderParams,
    CharacterConfigByIdLoader, CharacterConfigByIdLoaderParams, CharacterConfigsLoader,
    CharacterConfigsLoaderParams,
};
pub use figure_record_query::{
    FigureRecordByIdLoader, FigureRecordByIdLoaderParams, FigureRecordsByCharacterLoader,
    FigureRecordsByCharacterLoaderParams,
};
