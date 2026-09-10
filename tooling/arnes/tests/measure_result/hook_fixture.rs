use crate::measure_support::Harness;
use serde_json::Value;
use std::io::Write;
use std::process::{Output, Stdio};

impl Harness {
    pub(super) fn hook(
        &self,
        agent: &str,
        payload: &Value,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = self
            .command()
            .args(["measure", "hook", "--agent", agent])
            .stdin(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("required test value is missing")?
            .write_all(payload.to_string().as_bytes())?;
        Ok(child.wait_with_output()?)
    }
}
