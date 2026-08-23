use serde_json::Value;
use std::fmt::{self, Display};

#[derive(Clone, Copy)]
pub(super) enum ConfigFormat {
    Json,
    Toml,
}

impl Display for ConfigFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Json => "JSON",
            Self::Toml => "TOML",
        })
    }
}

pub(super) enum ParseError {
    Malformed,
    WrongRoot,
}

pub(super) fn parse(input: &str, format: ConfigFormat) -> Result<Value, ParseError> {
    match format {
        ConfigFormat::Json => match serde_json::from_str::<Value>(input) {
            Ok(value @ Value::Object(_)) => Ok(value),
            Ok(_) => Err(ParseError::WrongRoot),
            Err(_) => Err(ParseError::Malformed),
        },
        ConfigFormat::Toml => match toml::from_str::<toml::Value>(input) {
            Ok(value @ toml::Value::Table(_)) => {
                serde_json::to_value(value).map_err(|_| ParseError::Malformed)
            }
            Ok(_) => Err(ParseError::WrongRoot),
            Err(_) => Err(ParseError::Malformed),
        },
    }
}
