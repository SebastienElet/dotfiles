use std::fmt::{self, Display};

#[derive(Debug, Eq, PartialEq)]
pub struct MemoryError {
    code: &'static str,
    field: &'static str,
}

impl MemoryError {
    pub(crate) const fn new(code: &'static str, field: &'static str) -> Self {
        Self { code, field }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl Display for MemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.field)
    }
}

impl std::error::Error for MemoryError {}
