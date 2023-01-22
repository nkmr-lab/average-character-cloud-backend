use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use sqlx::PgPool;
use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};

use super::UserType;
use crate::BatchFnWithParams;
use crate::ShareableError;

#[derive(Debug, Clone)]
pub struct FigureRecordModel {
    pub id: String,
    pub user_id: String,
    pub character: String,
    pub figure: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub stroke_count: i32,
    pub disabled: bool,
    pub version: i32,
}

impl FigureRecordModel {
    pub fn into_entity(self) -> anyhow::Result<entities::FigureRecord> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let character = entities::Character::try_from(self.character.as_str())?;

        let figure = entities::Figure::from_json_ast(self.figure)
            .ok_or_else(|| anyhow!("figure must be valid json"))?;

        ensure!(
            self.stroke_count == i32::from(figure.stroke_count()),
            "stroke_count invalid"
        );

        Ok(entities::FigureRecord {
            id: entities::FigureRecordId::from(id),
            user_id: entities::UserId::from(self.user_id),
            character,
            figure,
            created_at: self.created_at,
            disabled: self.disabled,
            version: entities::Version::try_from(self.version)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct FigureRecordByIdLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FigureRecordByIdLoaderParams {
    pub user_id: entities::UserId,
}

#[async_trait]
impl BatchFnWithParams for FigureRecordByIdLoader {
    type K = entities::FigureRecordId;
    type V = Result<Option<entities::FigureRecord>, ShareableError>;
    type P = FigureRecordByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let ids = keys
            .iter()
            .map(|&id| Ulid::from(id).to_string())
            .collect::<Vec<_>>();

        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
                FigureRecordModel,
                r#"
                SELECT
                    r.id,
                    r.user_id,
                    r.character,
                    r.figure,
                    r.created_at,
                    r.stroke_count,
                    r.disabled,
                    r.version
                FROM
                    figure_records AS r
                    LEFT OUTER JOIN user_configs ON r.user_id = user_configs.user_id
                WHERE
                    r.id = Any($1)
                    AND (r.user_id = $2 OR user_configs.allow_sharing_figure_records)
            "#,
                ids.as_slice(),
                String::from(params.user_id.clone()),
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch figure_records")?;

            let figure_records = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert FigureRecord")?;

            let figure_record_map = figure_records
                .into_iter()
                .map(|figure_record| (figure_record.id, figure_record))
                .collect::<HashMap<_, _>>();

            Ok(figure_record_map)
        })()
        .await
        .map_err(ShareableError);

        keys.iter()
            .map(|key| {
                (
                    *key,
                    result
                        .as_ref()
                        .map(|figure_record_map| figure_record_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct FigureRecordsByCharacterLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FigureRecordsByCharacterLoaderParams {
    pub user_id: entities::UserId,
    pub ids: Option<Vec<entities::FigureRecordId>>,
    pub after_id: Option<entities::FigureRecordId>,
    pub before_id: Option<entities::FigureRecordId>,
    pub limit: entities::Limit,
    pub user_type: Option<UserType>,
}

#[async_trait]
impl BatchFnWithParams for FigureRecordsByCharacterLoader {
    type K = entities::Character;
    type V = Result<(Vec<entities::FigureRecord>, bool), ShareableError>;
    type P = FigureRecordsByCharacterLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let ids = params.ids.as_ref().map(|ids| {
            ids.iter()
                .map(|&id| Ulid::from(id).to_string())
                .collect::<Vec<_>>()
        });

        let characters = keys
            .iter()
            .map(|c| String::from(c.clone()))
            .collect::<Vec<_>>();

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
                        stroke_count,
                        disabled,
                        version
                    FROM (
                        SELECT
                            r.id,
                            r.user_id,
                            r.character,
                            r.figure,
                            r.created_at,
                            r.stroke_count,
                            rank() OVER (
                                PARTITION BY r.character
                                ORDER BY
                                    CASE WHEN $6 = 0 THEN r.id END DESC,
                                    CASE WHEN $6 = 1 THEN r.id END ASC
                            ) AS rank,
                            r.disabled,
                            r.version
                        FROM
                            figure_records AS r
                            JOIN character_configs ON r.character = character_configs.character AND r.user_id = character_configs.user_id
                            LEFT OUTER JOIN user_configs ON r.user_id = user_configs.user_id
                        WHERE
                            (r.user_id = $1 OR user_configs.allow_sharing_figure_records)
                            AND
                            r.character = Any($2)
                            AND
                            ($3::VARCHAR(64)[] IS NULL OR r.id = Any($3))
                            AND
                            ($4::VARCHAR(64) IS NULL OR r.id < $4)
                            AND
                            ($5::VARCHAR(64) IS NULL OR r.id > $5)
                            AND
                            r.stroke_count = character_configs.stroke_count
                            AND
                            (NOT $8 OR r.user_id = $1)
                            AND
                            (NOT $9 OR r.user_id <> $1)
                            AND
                            NOT r.disabled
                    ) as r
                    WHERE
                        rank <= $7
                    ORDER BY
                        id DESC
                "#,
                String::from(params.user_id.clone()),
                characters.as_slice(),
                ids.as_ref().map(|ids| ids.as_slice()),
                params.after_id.map(|id| Ulid::from(id).to_string()),
                params.before_id.map(|id| Ulid::from(id).to_string()),
                i32::from(params.limit.kind() == entities::LimitKind::Last),
                i64::from(params.limit.value()) + 1,
                params.user_type == Some(UserType::Myself),
                params.user_type == Some(UserType::Other),
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch figure_records")?;

            let figure_records = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert FigureRecord")?;

            let figure_records_map = figure_records
                .into_iter()
                .fold(HashMap::new(), |mut map, figure_record| {
                    map.entry(figure_record.character.clone())
                        .or_insert_with(Vec::new)
                        .push(figure_record);
                    map
                });


            Ok(figure_records_map)
        })()
        .await
        .map_err(ShareableError);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    result.as_ref().map_err(|e| e.clone()).and_then(
                        |figure_records_map| -> Result<_, ShareableError> {
                            let mut figure_records =
                                figure_records_map.get(key).cloned().unwrap_or_default();
                            let has_extra = figure_records.len()
                                > usize::try_from(params.limit.value()).context("into usize")?;
                            figure_records.truncate(
                                usize::try_from(params.limit.value()).context("into usize")?,
                            );
                            Ok((figure_records, has_extra))
                        },
                    ),
                )
            })
            .collect()
    }
}
