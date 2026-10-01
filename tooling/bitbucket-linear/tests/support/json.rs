use serde_json::Value;
use std::error::Error;

pub fn at<'a>(value: &'a Value, pointer: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .pointer(pointer)
        .ok_or_else(|| format!("fixture field missing: {pointer}").into())
}

pub fn put(replacement: Value, value: &mut Value, pointer: &str) -> Result<(), Box<dyn Error>> {
    let (parent, key) = pointer.rsplit_once('/').ok_or("fixture pointer")?;
    let object = value
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .ok_or("fixture parent object")?;
    object.insert(key.to_owned(), replacement);
    Ok(())
}

pub fn at_mut<'a>(value: &'a mut Value, pointer: &str) -> Result<&'a mut Value, Box<dyn Error>> {
    value
        .pointer_mut(pointer)
        .ok_or_else(|| format!("fixture field missing: {pointer}").into())
}
