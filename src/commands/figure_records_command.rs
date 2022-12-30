use crate::entities;
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};
use ulid::Ulid;

pub async fn create(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: entities::UserId,
    now: DateTime<Utc>,
    character: entities::Character,
    figure: entities::Figure,
) -> anyhow::Result<entities::FigureRecord> {
    let mut trx = conn.begin().await?;
    let record = entities::FigureRecord {
        id: entities::FigureRecordId::from(Ulid::from_datetime(now)),
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
            Ulid::from(record.id).to_string(),
            String::from(record.user_id.clone()),
            String::from(record.character.clone()),
            record.figure.to_json_ast(),
            record.created_at,
            i32::from(record.figure.stroke_count()),
        )
        .execute(&mut *trx)
        .await
        .context("fetch figure_records")?;

    trx.commit().await?;
    Ok(record)
}
