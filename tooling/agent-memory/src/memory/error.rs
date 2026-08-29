use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryErrorClass {
    Rejection,
    Conflict,
    Unavailable,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MemoryError {
    class: MemoryErrorClass,
    code: &'static str,
    field: &'static str,
}

impl MemoryError {
    pub(crate) const fn new(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Rejection,
            code,
            field,
        }
    }

    pub(crate) const fn conflict(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Conflict,
            code,
            field,
        }
    }

    pub(crate) const fn unavailable(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Unavailable,
            code,
            field,
        }
    }

    pub const fn class(&self) -> MemoryErrorClass {
        self.class
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
