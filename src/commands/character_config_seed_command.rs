use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn update_seeds(pool: &PgPool, now: DateTime<Utc>) -> anyhow::Result<()> {
    let mut trx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
            SELECT
                character,
                AVG(stroke_count)::INTEGER as avg_stroke_count
            FROM
                character_configs
                INNER JOIN user_configs ON character_configs.user_id = user_configs.user_id
            WHERE
                user_configs.allow_sharing_character_configs
            GROUP BY
                character
            "#,
    )
    .fetch_all(&mut trx)
    .await?;

    sqlx::query!("DELETE FROM character_config_seeds")
        .execute(&mut trx)
        .await?;

    for row in result {
        sqlx::query!(
            r#"
                INSERT INTO character_config_seeds (character, stroke_count, updated_at)
                VALUES ($1, $2, $3)
                "#,
            row.character,
            row.avg_stroke_count
                .ok_or_else(|| anyhow::anyhow!("avg_stroke_count is null"))?,
            now,
        )
        .execute(&mut trx)
        .await?;
    }

    trx.commit().await?;
    Ok(())
}
