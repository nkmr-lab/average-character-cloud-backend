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
    type V = Result<Option<entities::CharacterConfig>, ShareableError>;
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
                    .map(|character_config| (character_config.character.clone(), character_config))
                    .collect::<HashMap<_, _>>()
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    character_config_map
                        .as_ref()
                        .map(|character_config_map| character_config_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigsLoader<A> {
    pub character_configs_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigsLoaderParams {
    pub user_id: entities::UserId,
    pub after_character: Option<entities::Character>,
    pub before_character: Option<entities::Character>,
    pub limit: entities::Limit,
}


impl<A> BatchFnWithParams for CharacterConfigsLoader<A>
where
    A: ports::CharacterConfigsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = ();
    type V = Result<ports::PaginationResult<entities::CharacterConfig>, ShareableError>;
    type P = CharacterConfigsLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result = self
            .character_configs_repository
            .get(
                params.user_id.clone(),
                params.after_character.clone(),
                params.before_character.clone(),
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
