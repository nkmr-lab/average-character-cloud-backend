use crate::entities;
use anyhow::{anyhow, Context};
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
        disabled: false,
        version: entities::Version::new(),
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

pub async fn update(
    conn: impl Acquire<'_, Database = Postgres>,
    mut figure_record: entities::FigureRecord,
    disabled: Option<bool>,
) -> anyhow::Result<entities::FigureRecord> {
    let mut trx = conn.begin().await?;
    let prev_version = figure_record.version;

    figure_record.version = figure_record.version.next();
    if let Some(disabled) = disabled {
        figure_record.disabled = disabled;
    }

    let result = sqlx::query!(
        r#"
            UPDATE figure_records
                SET
                    disabled = $1,
                    version = $2
                WHERE
                    user_id = $3
                    AND
                    id = $4
                    AND
                    version = $5
            "#,
        disabled,
        i32::from(figure_record.version),
        String::from(figure_record.user_id.clone()),
        Ulid::from(figure_record.id.clone()).to_string(),
        i32::from(prev_version),
    )
    .execute(&mut *trx)
    .await
    .context("update figure_record")?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("conflict"));
    }

    trx.commit().await?;
    Ok(figure_record)
}
