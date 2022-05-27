use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Record {
    pub id: Uuid,
    pub character: char,
    pub figure: super::figure::Figure,
    pub created_at: DateTime<Utc>,
}
