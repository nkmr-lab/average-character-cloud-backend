use sqlx::PgPool;

use crate::queries::{
    CharacterConfigByCharacterLoader, CharacterConfigSeedByCharacterLoader,
    CharacterConfigSeedsLoader, CharacterConfigsLoader, FigureRecordByIdLoader,
    FigureRecordsByCharacterLoader,
};
use crate::{adapters, DataloaderWithParams};

pub struct Loaders {
    pub character_config_by_character_loader: DataloaderWithParams<
        CharacterConfigByCharacterLoader<adapters::CharacterConfigsRepositoryImpl<PgPool>>,
    >,
    pub character_configs_loader: DataloaderWithParams<
        CharacterConfigsLoader<adapters::CharacterConfigsRepositoryImpl<PgPool>>,
    >,
    pub figure_record_by_id_loader:
        DataloaderWithParams<FigureRecordByIdLoader<adapters::FigureRecordsRepositoryImpl<PgPool>>>,
    pub figure_records_by_character_loader: DataloaderWithParams<
        FigureRecordsByCharacterLoader<adapters::FigureRecordsRepositoryImpl<PgPool>>,
    >,
    pub character_config_seed_by_character_loader:
        DataloaderWithParams<CharacterConfigSeedByCharacterLoader>,
    pub character_config_seeds_loader: DataloaderWithParams<CharacterConfigSeedsLoader>,
}

impl Loaders {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            character_config_by_character_loader: DataloaderWithParams::new(
                CharacterConfigByCharacterLoader {
                    character_configs_repository: adapters::CharacterConfigsRepositoryImpl::new(
                        pool.clone(),
                    ),
                },
            ),
            character_configs_loader: DataloaderWithParams::new(CharacterConfigsLoader {
                character_configs_repository: adapters::CharacterConfigsRepositoryImpl::new(
                    pool.clone(),
                ),
            }),
            figure_record_by_id_loader: DataloaderWithParams::new(FigureRecordByIdLoader {
                figure_records_repository: adapters::FigureRecordsRepositoryImpl::new(pool.clone()),
            }),
            figure_records_by_character_loader: DataloaderWithParams::new(
                FigureRecordsByCharacterLoader {
                    figure_records_repository: adapters::FigureRecordsRepositoryImpl::new(
                        pool.clone(),
                    ),
                },
            ),
            character_config_seed_by_character_loader: DataloaderWithParams::new(
                CharacterConfigSeedByCharacterLoader { pool: pool.clone() },
            ),
            character_config_seeds_loader: DataloaderWithParams::new(CharacterConfigSeedsLoader {
                pool: pool.clone(),
            }),
        }
    }
}
