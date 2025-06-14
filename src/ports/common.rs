#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserType {
    Myself,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaginationResult<T> {
    pub values: Vec<T>,
    pub has_next: bool,
}
