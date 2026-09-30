use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnvironmentValue {
    Reference(String),
    RedactedLiteral,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedRegistration {
    pub command: String,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, EnvironmentValue>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Default)]
pub struct ObservedConfiguration {
    pub registrations: BTreeMap<String, ObservedRegistration>,
}
