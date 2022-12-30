use super::{character, UserId};
use chrono::{DateTime, Utc};
use derive_more::{From, Into};
use ulid::Ulid;

#[derive(Clone, Debug, Into, From, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FigureRecordId(Ulid);

#[derive(Clone, Debug)]
pub struct FigureRecord {
    pub id: FigureRecordId,
    pub user_id: UserId,
    pub character: character::Character,
    pub figure: super::figure::Figure,
    pub created_at: DateTime<Utc>,
}
