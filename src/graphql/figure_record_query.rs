use std::str::FromStr;

use chrono::{DateTime, Utc};

use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};

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
