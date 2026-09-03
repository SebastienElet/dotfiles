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
    diagnostic: Option<Box<crate::Diagnostic>>,
}

impl MemoryError {
    pub(crate) const fn new(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Rejection,
            code,
            field,
            diagnostic: None,
        }
    }

    pub(crate) const fn conflict(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Conflict,
            code,
            field,
            diagnostic: None,
        }
    }

    pub(crate) const fn unavailable(code: &'static str, field: &'static str) -> Self {
        Self {
            class: MemoryErrorClass::Unavailable,
            code,
            field,
            diagnostic: None,
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

    pub fn diagnostic(&self) -> crate::Diagnostic {
        self.diagnostic
            .as_deref()
            .copied()
            .unwrap_or_else(|| crate::Diagnostic::for_code(self.code, self.field))
    }

    pub(crate) fn with_diagnostic(mut self, diagnostic: crate::Diagnostic) -> Self {
        self.diagnostic = Some(Box::new(diagnostic));
        self
    }

    pub(crate) fn with_message(self, message: &'static str) -> Self {
        let diagnostic = crate::Diagnostic {
            message,
            ..self.diagnostic()
        };
        self.with_diagnostic(diagnostic)
    }

    pub(crate) fn at_item(self, index: usize) -> Self {
        let diagnostic = crate::Diagnostic {
            item_index: Some(index),
            ..self.diagnostic()
        };
        self.with_diagnostic(diagnostic)
    }
}

impl Display for MemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.field)
    }
}

impl std::error::Error for MemoryError {}
