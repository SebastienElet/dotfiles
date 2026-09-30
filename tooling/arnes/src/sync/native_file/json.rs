use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::value::{RawValue, to_raw_value};
use std::collections::BTreeMap;

pub type Object = BTreeMap<String, Box<RawValue>>;

pub fn parse(bytes: &[u8]) -> Result<Object, &'static str> {
    crate::mcp::json::parse(bytes).map_err(|_| "JSON is malformed or has duplicate keys")?;
    serde_json::from_slice(bytes).map_err(|_| "JSON must be an object")
}

pub fn get<T: DeserializeOwned>(object: &Object, key: &str) -> Result<Option<T>, &'static str> {
    object
        .get(key)
        .map(|value| {
            serde_json::from_str(value.get()).map_err(|_| "managed value has an unsupported type")
        })
        .transpose()
}

pub fn child(object: &Object, key: &str) -> Result<Object, &'static str> {
    get(object, key).map(Option::unwrap_or_default)
}

pub fn set<T: DeserializeOwned + Serialize + PartialEq>(
    object: &mut Object,
    key: &str,
    expected: &T,
) -> Result<bool, &'static str> {
    if get::<T>(object, key)?.as_ref() == Some(expected) {
        return Ok(false);
    }
    put(object, key, expected)?;
    Ok(true)
}

pub fn put<T: Serialize>(object: &mut Object, key: &str, value: &T) -> Result<(), &'static str> {
    object.insert(
        key.to_owned(),
        to_raw_value(value).map_err(|_| "managed value could not be rendered")?,
    );
    Ok(())
}
