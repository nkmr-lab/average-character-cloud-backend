use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use sqlx::PgPool;
use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};

use super::dataloader_with_params::{BatchFnWithParams, ShareableError};

#[derive(Debug, Clone)]
pub struct FigureRecordModel {
    pub id: String,
    pub user_id: String,
    pub character: String,
    pub figure: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub stroke_count: i32,
}

impl FigureRecordModel {
    pub fn into_entity(self) -> anyhow::Result<entities::figure_record::FigureRecord> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let character = entities::character::Character::try_from(self.character.as_str())?;

        let figure = entities::figure::Figure::from_json_ast(self.figure)
            .ok_or_else(|| anyhow!("figure must be valid json"))?;

        ensure!(
            self.stroke_count as usize == figure.strokes.len(),
            "stroke_count invalid"
        );

        Ok(entities::figure_record::FigureRecord {
            id,
            user_id: self.user_id,
            character,
            figure,
            created_at: self.created_at,
        })
    }
}

#[derive(Clone, Debug)]
pub struct FigureRecordByIdLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FigureRecordByIdLoaderParams {
    pub user_id: String,
}

#[async_trait]
impl BatchFnWithParams for FigureRecordByIdLoader {
    type K = Ulid;
    type V = Result<Option<entities::figure_record::FigureRecord>, ShareableError>;
    type P = FigureRecordByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let ids = keys.iter().map(|id| id.to_string()).collect::<Vec<_>>();

        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
                FigureRecordModel,
                r#"
                SELECT
                    id,
                    user_id,
                    character,
                    figure,
                    created_at,
                    stroke_count
                FROM
                    figure_records
                WHERE
                    id = Any($1)
                    AND user_id = $2
            "#,
                ids.as_slice(),
                &params.user_id,
            )
            .fetch_optional(&self.pool)
            .await
            .context("fetch figure_records")?;

            let figure_records = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert FigureRecord")?;

            let figure_record_map = figure_records
                .into_iter()
                .map(|figure_record| (figure_record.id.clone(), figure_record))
                .collect::<HashMap<_, _>>();

            Ok(figure_record_map)
        })()
        .await
        .map_err(ShareableError);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    result
                        .as_ref()
                        .map(|figure_record_map| figure_record_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}
