use crate::entities;
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};
use ulid::Ulid;

pub async fn create(
    trx: &mut Transaction<'_, Postgres>,
    user_id: String,
    now: DateTime<Utc>,
    character: entities::Character,
    figure: entities::Figure,
) -> anyhow::Result<entities::FigureRecord> {
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
            i32::try_from(record.figure.strokes.len())?,
        )
        .execute(&mut *trx)
        .await
        .context("fetch figure_records")?;

    Ok(record)
}
