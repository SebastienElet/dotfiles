mod messages;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl Diagnostic {
    pub(crate) const fn new(message: &'static str) -> Self {
        Self {
            message,
            minimum: None,
            maximum: None,
            unit: None,
            item_index: None,
            line: None,
            column: None,
        }
    }

    pub(crate) fn for_code(code: &str, field: &str) -> Self {
        let diagnostic = Self::new(messages::message(code, field));
        if code == "input_too_large" {
            diagnostic.bounds(1, 1_048_576, "bytes")
        } else {
            diagnostic
        }
    }

    pub(crate) const fn bounds(
        mut self,
        minimum: usize,
        maximum: usize,
        unit: &'static str,
    ) -> Self {
        self.minimum = Some(minimum);
        self.maximum = Some(maximum);
        self.unit = Some(unit);
        self
    }
}
