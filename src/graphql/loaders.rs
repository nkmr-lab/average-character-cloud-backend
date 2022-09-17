use sqlx::PgPool;

use crate::queries::{
    CharacterConfigByCharacterLoader, CharacterConfigByIdLoader, CharacterConfigsLoader,
    FigureRecordByIdLoader, FigureRecordsByCharacterLoader,
};
use crate::DataloaderWithParams;

pub struct Loaders {
    pub character_config_by_character_loader:
        DataloaderWithParams<CharacterConfigByCharacterLoader>,
    pub character_config_by_id_loader: DataloaderWithParams<CharacterConfigByIdLoader>,
    pub character_configs_loader: DataloaderWithParams<CharacterConfigsLoader>,
    pub figure_record_by_id_loader: DataloaderWithParams<FigureRecordByIdLoader>,
    pub figure_records_by_character_loader: DataloaderWithParams<FigureRecordsByCharacterLoader>,
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
            figure_records_by_character_loader: DataloaderWithParams::new(
                FigureRecordsByCharacterLoader { pool: pool.clone() },
            ),
        }
    }
}
