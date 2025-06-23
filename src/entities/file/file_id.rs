use derive_more::{From, Into};
use ulid::Ulid;

#[derive(Clone, Debug, Into, From, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileId(Ulid);
