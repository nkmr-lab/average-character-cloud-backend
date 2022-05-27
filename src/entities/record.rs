use chrono::{DateTime, Utc};
use ulid::Ulid;

pub struct Record {
    pub id: Ulid,
    pub user_id: String,
    pub character: char,
    pub figure: super::figure::Figure,
    pub created_at: DateTime<Utc>,
}
