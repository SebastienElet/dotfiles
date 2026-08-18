use super::MeasureError;
use super::model::{HookAgent, InvalidRecord};
use super::store::Store;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;

const MAX_PAYLOAD_BYTES: usize = 1_048_576;

pub struct Payload {
    value: Value,
    size: usize,
    sha256: String,
}

impl Payload {
    pub fn read(store: &Store, agent: HookAgent) -> Result<Self, MeasureError> {
        let raw = read_stdin()?;
        if raw.oversized {
            return raw.reject(store, agent, "payload exceeds 1048576 bytes");
        }
        let value = match serde_json::from_slice(&raw.bytes) {
            Ok(value) => value,
            Err(error) => {
                return raw.reject(store, agent, &format!("invalid JSON: {error}"));
            }
        };
        Ok(Self {
            value,
            size: raw.size,
            sha256: raw.sha256,
        })
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn record_invalid(
        &self,
        store: &Store,
        agent: HookAgent,
        error: &str,
    ) -> Result<(), MeasureError> {
        append_invalid(store, agent, self.size, &self.sha256, error)
    }
}

struct RawPayload {
    bytes: Vec<u8>,
    size: usize,
    sha256: String,
    oversized: bool,
}

impl RawPayload {
    fn reject<T>(&self, store: &Store, agent: HookAgent, error: &str) -> Result<T, MeasureError> {
        append_invalid(store, agent, self.size, &self.sha256, error)?;
        Err(MeasureError::new(error))
    }
}

fn read_stdin() -> Result<RawPayload, MeasureError> {
    let mut input = std::io::stdin().lock();
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut size = 0;
    let mut hasher = Sha256::new();
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        size += read;
        hasher.update(&buffer[..read]);
        if bytes.len() < MAX_PAYLOAD_BYTES {
            let keep = (MAX_PAYLOAD_BYTES - bytes.len()).min(read);
            bytes.extend_from_slice(&buffer[..keep]);
        }
    }
    Ok(RawPayload {
        bytes,
        size,
        sha256: format!("{:x}", hasher.finalize()),
        oversized: size > MAX_PAYLOAD_BYTES,
    })
}

fn append_invalid(
    store: &Store,
    agent: HookAgent,
    size: usize,
    sha256: &str,
    error: &str,
) -> Result<(), MeasureError> {
    store.append_invalid(&InvalidRecord {
        timestamp_ms: super::hook::now_ms(),
        agent: agent.as_str(),
        size,
        sha256: sha256.to_owned(),
        error: error.to_owned(),
    })
}
