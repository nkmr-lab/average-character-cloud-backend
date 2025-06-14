use super::common;
use crate::entities;
use chrono::{DateTime, Utc};

pub trait FigureRecordsRepository {
    type Error;

    async fn create(
        &mut self,
        user_id: entities::UserId,
        now: DateTime<Utc>,
        character: entities::Character,
        figure: entities::Figure,
    ) -> Result<entities::FigureRecord, Self::Error>;

    async fn update(
        &mut self,
        figure_record: entities::FigureRecord,
        disabled: Option<bool>,
    ) -> Result<entities::FigureRecord, Self::Error>;

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        ids: &[entities::FigureRecordId],
    ) -> Result<Vec<entities::FigureRecord>, Self::Error>;

    async fn get_by_characters(
        &mut self,
        user_id: entities::UserId,
        characters: &[entities::Character],
        ids: Option<&[entities::FigureRecordId]>,
        after_id: Option<entities::FigureRecordId>,
        before_id: Option<entities::FigureRecordId>,
        limit_per_character: entities::Limit,
        user_type: Option<common::UserType>,
    ) -> Result<Vec<entities::FigureRecord>, Self::Error>;
}
