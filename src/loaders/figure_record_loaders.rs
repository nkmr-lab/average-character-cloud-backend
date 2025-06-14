use std::collections::HashMap;

use async_trait::async_trait;

use crate::entities;
use crate::ports;
use anyhow::Context;

use crate::BatchFnWithParams;
use crate::ShareableError;

#[derive(Clone, Debug)]
pub struct FigureRecordByIdLoader<A> {
    pub figure_records_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FigureRecordByIdLoaderParams {
    pub user_id: entities::UserId,
}

#[async_trait]
impl<A> BatchFnWithParams for FigureRecordByIdLoader<A>
where
    A: ports::FigureRecordsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::FigureRecordId;
    type V = Result<Option<entities::FigureRecord>, ShareableError>;
    type P = FigureRecordByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let figure_record_map = self
            .figure_records_repository
            .get_by_ids(params.user_id.clone(), keys)
            .await
            .map(|figure_records| {
                figure_records
                    .into_iter()
                    .map(|figure_record| (figure_record.id, figure_record))
                    .collect::<HashMap<_, _>>()
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    *key,
                    figure_record_map
                        .as_ref()
                        .map(|figure_record_map| figure_record_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct FigureRecordsByCharacterLoader<A> {
    pub figure_records_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FigureRecordsByCharacterLoaderParams {
    pub user_id: entities::UserId,
    pub ids: Option<Vec<entities::FigureRecordId>>,
    pub after_id: Option<entities::FigureRecordId>,
    pub before_id: Option<entities::FigureRecordId>,
    pub limit: entities::Limit,
    pub user_type: Option<ports::UserType>,
}

#[async_trait]
impl<A> BatchFnWithParams for FigureRecordsByCharacterLoader<A>
where
    A: ports::FigureRecordsRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::Character;
    type V = Result<ports::PaginationResult<entities::FigureRecord>, ShareableError>;
    type P = FigureRecordsByCharacterLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let figure_record_map = self
            .figure_records_repository
            .get_by_characters(
                params.user_id.clone(),
                keys,
                params.ids.as_ref().map(|ids| ids.as_slice()),
                params.after_id,
                params.before_id,
                params.limit.increment_unchecked(),
                params.user_type,
            )
            .await
            .map(|figure_records| {
                figure_records
                    .into_iter()
                    .fold(HashMap::new(), |mut map, figure_record| {
                        map.entry(figure_record.character.clone())
                            .or_insert_with(Vec::new)
                            .push(figure_record);
                        map
                    })
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    figure_record_map.as_ref().map_err(|e| e.clone()).and_then(
                        |figure_records_map| -> Result<_, ShareableError> {
                            let mut figure_records =
                                figure_records_map.get(key).cloned().unwrap_or_default();
                            let has_next = figure_records.len()
                                > usize::try_from(params.limit.value()).context("into usize")?;
                            figure_records.truncate(
                                usize::try_from(params.limit.value()).context("into usize")?,
                            );
                            Ok(ports::PaginationResult {
                                values: figure_records,
                                has_next,
                            })
                        },
                    ),
                )
            })
            .collect()
    }
}
