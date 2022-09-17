use crate::entities;
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use ulid::Ulid;

pub async fn create(
    pool: &Pool<Postgres>,
    user_id: String,
    now: DateTime<Utc>,
    character: entities::Character,
    figure: entities::Figure,
) -> anyhow::Result<entities::FigureRecord> {
    let mut trx = pool.begin().await?;
    let record = entities::FigureRecord {
        id: Ulid::from_datetime(now),
        user_id,
        character,
        figure,
        created_at: now,
    };

    sqlx::query!(
            r#"
                INSERT INTO figure_records (id, user_id, character, figure, created_at, stroke_count)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            record.id.to_string(),
            record.user_id,
            String::from(record.character.clone()),
            record.figure.to_json_ast(),
            record.created_at,
            record.figure.strokes.len() as i32,
        )
        .execute(&mut trx)
        .await
        .context("fetch figure_records")?;

    trx.commit().await?;
    Ok(record)
}