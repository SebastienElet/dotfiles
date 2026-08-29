#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffError {
    pub message: String,
    pub exit_code: u8,
}

impl HandoffError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    pub fn unexpected(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 3,
        }
    }
}
