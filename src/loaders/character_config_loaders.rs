use std::collections::HashMap;

use crate::entities;
use anyhow::Context;

use crate::ports;
use crate::BatchFnWithParams;
use crate::ShareableError;

#[derive(Clone, Debug)]
pub struct CharacterConfigByCharacterLoader<A> {
    pub character_configs_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigByCharacterLoaderParams {
    pub user_id: entities::UserId,
}

impl<A> BatchFnWithParams for CharacterConfigByCharacterLoader<A>
where
    A: ports::CharacterConfigsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::Character;
    type V = Result<Vec<entities::CharacterConfig>, ShareableError>;
    type P = CharacterConfigByCharacterLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let character_config_map = self
            .character_configs_repository
            .get_by_characters(keys, params.user_id.clone())
            .await
            .map(|character_configs| {
                character_configs
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, character_config| {
                        acc.entry(character_config.character.clone())
                            .or_insert_with(Vec::new)
                            .push(character_config);
                        acc
                    })
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    character_config_map
                        .as_ref()
                        .map(|character_config_map| {
                            character_config_map.get(key).cloned().unwrap_or_default()
                        })
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigLoader<A> {
    pub character_configs_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigLoaderParams {
    pub user_id: entities::UserId,
    pub after_id: Option<(entities::Character, entities::StrokeCount)>,
    pub before_id: Option<(entities::Character, entities::StrokeCount)>,
    pub limit: entities::Limit,
}

impl<A> BatchFnWithParams for CharacterConfigLoader<A>
where
    A: ports::CharacterConfigsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = ();
    type V = Result<ports::PaginationResult<entities::CharacterConfig>, ShareableError>;
    type P = CharacterConfigLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result = self
            .character_configs_repository
            .query(
                params.user_id.clone(),
                params.after_id.clone(),
                params.before_id.clone(),
                params.limit.increment_unchecked(),
            )
            .await
            .and_then(|mut character_configs| {
                let has_next = character_configs.len()
                    > usize::try_from(params.limit.value()).context("into usize")?;

                character_configs
                    .truncate(usize::try_from(params.limit.value()).context("into usize")?);

                if params.limit.kind() == entities::LimitKind::Last {
                    character_configs.reverse();
                }
                Ok(ports::PaginationResult {
                    values: character_configs,
                    has_next,
                })
            })
            .map_err(ShareableError::from);

        vec![((), result)].into_iter().collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigByIdLoader<A> {
    pub character_configs_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigByIdLoaderParams {
    pub user_id: entities::UserId,
}

impl<A> BatchFnWithParams for CharacterConfigByIdLoader<A>
where
    A: ports::CharacterConfigsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = (entities::Character, entities::StrokeCount);
    type V = Result<entities::CharacterConfig, ShareableError>;
    type P = CharacterConfigByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let character_config_map = self
            .character_configs_repository
            .get_by_ids(params.user_id.clone(), keys)
            .await
            .map_err(ShareableError::from);

        character_config_map
            .map(|map| {
                map.into_iter()
                    .map(|(key, value)| (key, Ok(value)))
                    .collect()
            })
            .unwrap_or_else(|e| {
                keys.iter()
                    .map(|key| (key.clone(), Err(e.clone())))
                    .collect()
            })
    }
}
