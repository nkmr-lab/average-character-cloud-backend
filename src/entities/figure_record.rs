use super::character;
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct FigureRecord {
    pub id: Ulid,
    pub user_id: String,
    pub character: character::Character,
    pub figure: super::figure::Figure,
    pub created_at: DateTime<Utc>,
}
