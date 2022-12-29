use derive_more::{From, Into};

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, From, Into)]
pub struct UserId(String);
