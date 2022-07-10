use sqlx::PgPool;

use super::{
    character_config_query::{
        CharacterConfigByCharacterLoader, CharacterConfigByIdLoader, CharacterConfigsLoader,
    },
    dataloader_with_params::DataloaderWithParams,
    figure_record_query::FigureRecordByIdLoader,
};

pub struct Loaders {
    pub character_config_by_character_loader:
        DataloaderWithParams<CharacterConfigByCharacterLoader>,
    pub character_config_by_id_loader: DataloaderWithParams<CharacterConfigByIdLoader>,
    pub character_configs_loader: DataloaderWithParams<CharacterConfigsLoader>,
    pub figure_record_by_id_loader: DataloaderWithParams<FigureRecordByIdLoader>,
}

impl Loaders {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            character_config_by_character_loader: DataloaderWithParams::new(
                CharacterConfigByCharacterLoader { pool: pool.clone() },
            ),
            character_config_by_id_loader: DataloaderWithParams::new(CharacterConfigByIdLoader {
                pool: pool.clone(),
            }),
            character_configs_loader: DataloaderWithParams::new(CharacterConfigsLoader {
                pool: pool.clone(),
            }),
            figure_record_by_id_loader: DataloaderWithParams::new(FigureRecordByIdLoader {
                pool: pool.clone(),
            }),
        }
    }
}
