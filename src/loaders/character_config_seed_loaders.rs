use crate::entities;
use crate::ports;
use crate::BatchFnWithParams;
use crate::ShareableError;
use anyhow::Context;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CharacterConfigSeedByCharacterLoader<A> {
    pub character_config_seeds_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigSeedByCharacterLoaderParams {}

impl<A> BatchFnWithParams for CharacterConfigSeedByCharacterLoader<A>
where
    A: ports::CharacterConfigSeedsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::Character;
    type V = Result<Vec<entities::CharacterConfigSeed>, ShareableError>;
    type P = CharacterConfigSeedByCharacterLoaderParams;

    async fn load_with_params(
        &mut self,
        _params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let character_config_seed_map = self
            .character_config_seeds_repository
            .get_by_characters(keys)
            .await
            .map(|character_config_seeds| {
                character_config_seeds.into_iter().fold(
                    HashMap::new(),
                    |mut acc, character_config_seed| {
                        acc.entry(character_config_seed.character.clone())
                            .or_insert_with(Vec::new)
                            .push(character_config_seed);
                        acc
                    },
                )
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    character_config_seed_map
                        .as_ref()
                        .map(|character_config_seed_map| {
                            character_config_seed_map
                                .get(key)
                                .cloned()
                                .unwrap_or_default()
                        })
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigSeedsLoader<A> {
    pub character_config_seeds_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigSeedsLoaderParams {
    pub user_id: entities::UserId,
    pub after_id: Option<(entities::Character, entities::StrokeCount)>,
    pub before_id: Option<(entities::Character, entities::StrokeCount)>,
    pub limit: entities::Limit,
    pub include_exist_character_config: bool,
}

impl<A> BatchFnWithParams for CharacterConfigSeedsLoader<A>
where
    A: ports::CharacterConfigSeedsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = ();
    type V = Result<ports::PaginationResult<entities::CharacterConfigSeed>, ShareableError>;
    type P = CharacterConfigSeedsLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result = self
            .character_config_seeds_repository
            .query(
                params.user_id.clone(),
                params.after_id.clone(),
                params.before_id.clone(),
                params.limit.increment_unchecked(),
                params.include_exist_character_config,
            )
            .await
            .and_then(|mut character_config_seeds| {
                let has_next = character_config_seeds.len()
                    > usize::try_from(params.limit.value()).context("into usize")?;
                character_config_seeds
                    .truncate(usize::try_from(params.limit.value()).context("into usize")?);
                if params.limit.kind() == entities::LimitKind::Last {
                    character_config_seeds.reverse();
                }
                Ok(ports::PaginationResult {
                    values: character_config_seeds,
                    has_next,
                })
            })
            .map_err(ShareableError::from);
        vec![((), result)].into_iter().collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigSeedByIdLoader<A> {
    pub character_config_seeds_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigSeedByIdLoaderParams {}

impl<A> BatchFnWithParams for CharacterConfigSeedByIdLoader<A>
where
    A: ports::CharacterConfigSeedsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = (entities::Character, entities::StrokeCount);
    type V = Result<Option<entities::CharacterConfigSeed>, ShareableError>;
    type P = CharacterConfigSeedByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        _params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let character_config_seed_map = self
            .character_config_seeds_repository
            .get_by_ids(keys)
            .await
            .map(|character_config_seeds| {
                character_config_seeds
                    .into_iter()
                    .map(|character_config_seed| {
                        (
                            (
                                character_config_seed.character.clone(),
                                character_config_seed.stroke_count,
                            ),
                            character_config_seed,
                        )
                    })
                    .collect::<HashMap<_, _>>()
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    character_config_seed_map
                        .as_ref()
                        .map(|character_config_seed_map| {
                            character_config_seed_map.get(key).cloned()
                        })
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}
