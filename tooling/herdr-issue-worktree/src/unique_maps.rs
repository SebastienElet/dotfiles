use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

type Tokens = BTreeMap<String, String>;

#[derive(Deserialize)]
#[serde(transparent)]
struct UniqueTokens(#[serde(deserialize_with = "unique_map")] Tokens);

pub fn bindings<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Tokens>, D::Error> {
    Ok(unique_map::<D, UniqueTokens>(deserializer)?
        .into_iter()
        .map(|(key, UniqueTokens(tokens))| (key, tokens))
        .collect())
}

pub fn tokens<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Tokens, D::Error> {
    unique_map(deserializer)
}

fn unique_map<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    deserializer.deserialize_map(UniqueMapVisitor(PhantomData))
}

struct UniqueMapVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for UniqueMapVisitor<T> {
    type Value = BTreeMap<String, T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map with unique string keys")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry()? {
            if entries.insert(key, value).is_some() {
                return Err(A::Error::custom("Duplicate persisted or request map key"));
            }
        }
        Ok(entries)
    }
}
