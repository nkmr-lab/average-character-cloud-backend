use chrono::{DateTime, Utc};
use ulid::Ulid;

pub struct CharacterConfig {
    pub id: Ulid,
    pub user_id: String,
    pub character: char,
    pub stroke_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
