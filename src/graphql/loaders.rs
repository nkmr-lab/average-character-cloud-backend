use sqlx::PgPool;

use crate::loaders::{
    CharacterConfigByCharacterLoader, CharacterConfigByIdLoader, CharacterConfigLoader,
    CharacterConfigSeedByCharacterLoader, CharacterConfigSeedByIdLoader,
    CharacterConfigSeedsLoader, FigureRecordByIdLoader, FigureRecordsByCharacterConfigIdLoader,
    FileByIdLoader, GenerateTemplateByIdLoader, GenerateTemplatesLoader,
};
use crate::{adapters, DataloaderWithParams};

pub struct Loaders {
    pub character_config_by_character_loader: DataloaderWithParams<
        CharacterConfigByCharacterLoader<adapters::CharacterConfigsRepositoryImpl<PgPool>>,
    >,
    pub character_config_by_id_loader: DataloaderWithParams<
        CharacterConfigByIdLoader<adapters::CharacterConfigsRepositoryImpl<PgPool>>,
    >,
    pub character_config_loader: DataloaderWithParams<
        CharacterConfigLoader<adapters::CharacterConfigsRepositoryImpl<PgPool>>,
    >,
    pub figure_record_by_id_loader:
        DataloaderWithParams<FigureRecordByIdLoader<adapters::FigureRecordsRepositoryImpl<PgPool>>>,
    pub figure_records_by_character_config_id_loader: DataloaderWithParams<
        FigureRecordsByCharacterConfigIdLoader<adapters::FigureRecordsRepositoryImpl<PgPool>>,
    >,
    pub character_config_seed_by_character_loader: DataloaderWithParams<
        CharacterConfigSeedByCharacterLoader<adapters::CharacterConfigSeedsRepositoryImpl<PgPool>>,
    >,
    pub character_config_seed_by_id_loader: DataloaderWithParams<
        CharacterConfigSeedByIdLoader<adapters::CharacterConfigSeedsRepositoryImpl<PgPool>>,
    >,
    pub character_config_seeds_loader: DataloaderWithParams<
        CharacterConfigSeedsLoader<adapters::CharacterConfigSeedsRepositoryImpl<PgPool>>,
    >,
    pub file_by_id_loader:
        DataloaderWithParams<FileByIdLoader<adapters::FilesRepositoryImpl<PgPool>>>,
    pub generate_template_by_id_loader: DataloaderWithParams<
        GenerateTemplateByIdLoader<adapters::GenerateTemplatesRepositoryImpl<PgPool>>,
    >,
    pub generate_templates_loader: DataloaderWithParams<
        GenerateTemplatesLoader<adapters::GenerateTemplatesRepositoryImpl<PgPool>>,
    >,
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
            character_config_by_id_loader: DataloaderWithParams::new(CharacterConfigByIdLoader {
                character_configs_repository: adapters::CharacterConfigsRepositoryImpl::new(
                    pool.clone(),
                ),
            }),
            character_config_loader: DataloaderWithParams::new(CharacterConfigLoader {
                character_configs_repository: adapters::CharacterConfigsRepositoryImpl::new(
                    pool.clone(),
                ),
            }),
            figure_record_by_id_loader: DataloaderWithParams::new(FigureRecordByIdLoader {
                figure_records_repository: adapters::FigureRecordsRepositoryImpl::new(pool.clone()),
            }),
            figure_records_by_character_config_id_loader: DataloaderWithParams::new(
                FigureRecordsByCharacterConfigIdLoader {
                    figure_records_repository: adapters::FigureRecordsRepositoryImpl::new(
                        pool.clone(),
                    ),
                },
            ),
            character_config_seed_by_character_loader: DataloaderWithParams::new(
                CharacterConfigSeedByCharacterLoader {
                    character_config_seeds_repository:
                        adapters::CharacterConfigSeedsRepositoryImpl::new(pool.clone()),
                },
            ),
            character_config_seed_by_id_loader: DataloaderWithParams::new(
                CharacterConfigSeedByIdLoader {
                    character_config_seeds_repository:
                        adapters::CharacterConfigSeedsRepositoryImpl::new(pool.clone()),
                },
            ),
            character_config_seeds_loader: DataloaderWithParams::new(CharacterConfigSeedsLoader {
                character_config_seeds_repository:
                    adapters::CharacterConfigSeedsRepositoryImpl::new(pool.clone()),
            }),
            file_by_id_loader: DataloaderWithParams::new(FileByIdLoader {
                files_repository: adapters::FilesRepositoryImpl::new(pool.clone()),
            }),
            generate_template_by_id_loader: DataloaderWithParams::new(GenerateTemplateByIdLoader {
                generate_templates_repository: adapters::GenerateTemplatesRepositoryImpl::new(
                    pool.clone(),
                ),
            }),
            generate_templates_loader: DataloaderWithParams::new(GenerateTemplatesLoader {
                generate_templates_repository: adapters::GenerateTemplatesRepositoryImpl::new(
                    pool.clone(),
                ),
            }),
        }
    }
}
