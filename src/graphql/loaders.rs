use sqlx::PgPool;

use super::{
    character_config_query::CharacterConfigLoader, dataloader_with_params::DataloaderWithParams,
};

pub struct Loaders {
    pub character_config_loader: DataloaderWithParams<CharacterConfigLoader>,
}

impl Loaders {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            character_config_loader: DataloaderWithParams::new(CharacterConfigLoader {
                pool: pool.clone(),
            }),
        }
    }
}
