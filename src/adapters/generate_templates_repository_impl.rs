use std::str::FromStr;

use anyhow::{anyhow, Context};
use sqlx::{Acquire, Postgres};
use ulid::Ulid;

use crate::{entities, ports};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct GenerateTemplateModel {
    id: String,
    user_id: String,
    background_image_file_id: String,
    font_color: i32,
    writing_mode: i32,
    margin_block_start: i32,
    margin_inline_start: i32,
    line_spacing: i32,
    letter_spacing: i32,
    font_size: i32,
    font_weight: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    disabled: bool,
    version: i32,
}

impl GenerateTemplateModel {
    pub fn into_entity(self) -> anyhow::Result<entities::GenerateTemplate> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let background_image_file_id = entities::FileId::from(
            Ulid::from_str(&self.background_image_file_id).context("ulid decode error")?,
        );
        let font_color = entities::Color::try_from(self.font_color)?;
        let writing_mode = entities::WritingMode::try_from(self.writing_mode)?;
        let margin_block_start = entities::Margin::try_from(self.margin_block_start)?;
        let margin_inline_start = entities::Margin::try_from(self.margin_inline_start)?;
        let line_spacing = entities::Spacing::try_from(self.line_spacing)?;
        let letter_spacing = entities::Spacing::try_from(self.letter_spacing)?;
        let font_size = entities::FontSize::try_from(self.font_size)?;
        let font_weight = entities::FontWeight::try_from(self.font_weight)?;

        Ok(entities::GenerateTemplate {
            id: entities::GenerateTemplateId::from(id),
            user_id: entities::UserId::from(self.user_id),
            background_image_file_id,
            font_color,
            writing_mode,
            margin_block_start,
            margin_inline_start,
            line_spacing,
            letter_spacing,
            font_size,
            font_weight,
            created_at: self.created_at,
            updated_at: self.updated_at,
            disabled: self.disabled,
            version: entities::Version::try_from(self.version)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GenerateTemplatesRepositoryImpl<A> {
    db: A,
}

impl<A> GenerateTemplatesRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}
impl<A> ports::GenerateTemplatesRepository for GenerateTemplatesRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn create(
        &mut self,
        mut generate_template: entities::GenerateTemplate,
    ) -> Result<entities::GenerateTemplate, Self::Error> {
        let mut trx = self.db.begin().await?;
        generate_template.version = generate_template.version.next();

        sqlx::query!(
            r#"
                INSERT INTO generate_templates (
                    id,
                    user_id,
                    background_image_file_id,
                    font_color,
                    writing_mode,
                    margin_block_start,
                    margin_inline_start,
                    line_spacing,
                    letter_spacing,
                    font_size,
                    font_weight,
                    created_at,
                    updated_at,
                    disabled,
                    version
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            Ulid::from(generate_template.id).to_string(),
            String::from(generate_template.user_id.clone()),
            Ulid::from(generate_template.background_image_file_id).to_string(),
            i32::from(generate_template.font_color),
            i32::from(generate_template.writing_mode),
            i32::from(generate_template.margin_block_start),
            i32::from(generate_template.margin_inline_start),
            i32::from(generate_template.line_spacing),
            i32::from(generate_template.letter_spacing),
            i32::from(generate_template.font_size),
            i32::from(generate_template.font_weight),
            generate_template.created_at,
            generate_template.updated_at,
            generate_template.disabled,
            i32::from(generate_template.version),
        )
        .execute(&mut *trx)
        .await
        .context("insert generate_template")?;

        trx.commit().await?;
        Ok(generate_template)
    }

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        ids: &[entities::GenerateTemplateId],
    ) -> Result<Vec<entities::GenerateTemplate>, Self::Error> {
        let mut conn = self.db.acquire().await?;
        let ids = ids
            .iter()
            .map(|&id| Ulid::from(id).to_string())
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            GenerateTemplateModel,
            r#"
                SELECT
                    id,
                    user_id,
                    background_image_file_id,
                    font_color,
                    writing_mode,
                    margin_block_start,
                    margin_inline_start,
                    line_spacing,
                    letter_spacing,
                    font_size,
                    font_weight,
                    created_at,
                    updated_at,
                    disabled,
                    version
                FROM
                    generate_templates
                WHERE
                    id = Any($1)
                    AND user_id = $2
            "#,
            ids.as_slice(),
            String::from(user_id.clone()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch generate_templates")?;

        let generate_templates = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert GenerateTemplate")?;

        Ok(generate_templates)
    }

    async fn update(
        &mut self,
        now: DateTime<Utc>,
        mut generate_template: entities::GenerateTemplate,
    ) -> Result<entities::GenerateTemplate, Self::Error> {
        let mut trx = self.db.begin().await?;
        let prev_version = generate_template.version;
        generate_template.version = generate_template.version.next();
        generate_template.updated_at = now;

        let result = sqlx::query!(
            r#"
            UPDATE generate_templates
                SET
                    background_image_file_id = $1,
                    font_color = $2,
                    writing_mode = $3,
                    margin_block_start = $4,
                    margin_inline_start = $5,
                    line_spacing = $6,
                    letter_spacing = $7,
                    font_size = $8,
                    font_weight = $9,
                    updated_at = $10,
                    disabled = $11,
                    version = $12
                WHERE
                    user_id = $13
                    AND id = $14
                    AND version = $15
            "#,
            Ulid::from(generate_template.background_image_file_id).to_string(),
            i32::from(generate_template.font_color),
            i32::from(generate_template.writing_mode),
            i32::from(generate_template.margin_block_start),
            i32::from(generate_template.margin_inline_start),
            i32::from(generate_template.line_spacing),
            i32::from(generate_template.letter_spacing),
            i32::from(generate_template.font_size),
            i32::from(generate_template.font_weight),
            generate_template.updated_at,
            generate_template.disabled,
            i32::from(generate_template.version),
            String::from(generate_template.user_id.clone()),
            Ulid::from(generate_template.id.clone()).to_string(),
            i32::from(prev_version),
        )
        .execute(&mut *trx)
        .await
        .context("update generate_template")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("conflict"));
        }

        trx.commit().await?;
        Ok(generate_template)
    }

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<entities::GenerateTemplateId>,
        before_id: Option<entities::GenerateTemplateId>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::GenerateTemplate>, Self::Error> {
        let mut conn = self.db.acquire().await?;

        let after_id = after_id.map(|id| Ulid::from(id).to_string());
        let before_id = before_id.map(|id| Ulid::from(id).to_string());
        let models = sqlx::query_as!(
            GenerateTemplateModel,
            r#"
                SELECT
                    id,
                    user_id,
                    background_image_file_id,
                    font_color,
                    writing_mode,
                    margin_block_start,
                    margin_inline_start,
                    line_spacing,
                    letter_spacing,
                    font_size,
                    font_weight,
                    created_at,
                    updated_at,
                    disabled,
                    version
                FROM
                    generate_templates
                WHERE
                    user_id = $1
                    AND ($2::VARCHAR(64) IS NULL OR id > $2)
                    AND ($3::VARCHAR(64) IS NULL OR id < $3)
                ORDER BY
                    CASE WHEN $4 = 0 THEN id END ASC,
                    CASE WHEN $4 = 1 THEN id END DESC
                LIMIT $5
            "#,
            String::from(user_id.clone()),
            after_id.as_deref(),
            before_id.as_deref(),
            i32::from(limit.kind() == entities::LimitKind::Last),
            i64::from(limit.value()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch generate_templates")?;

        let generate_templates = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert GenerateTemplate")?;

        Ok(generate_templates)
    }
}
