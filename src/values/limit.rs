#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LimitKind {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Limit {
    pub kind: LimitKind,
    pub value: i32,
}
