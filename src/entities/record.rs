use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Record {
    id: Uuid,
    character: char,
    figure: super::figure::Figure,
    created_at: DateTime<Utc>,
}
