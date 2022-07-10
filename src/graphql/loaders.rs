use sqlx::PgPool;

use super::{
    character_config_query::{
        CharacterConfigByCharacterLoader, CharacterConfigByIdLoader, CharacterConfigsLoader,
    },
    dataloader_with_params::DataloaderWithParams,
};

pub struct Loaders {
    pub character_config_by_character_loader:
        DataloaderWithParams<CharacterConfigByCharacterLoader>,
    pub character_config_by_id_loader: DataloaderWithParams<CharacterConfigByIdLoader>,
    pub character_configs_loader: DataloaderWithParams<CharacterConfigsLoader>,
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
        }
    }
}
