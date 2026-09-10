use crate::measure::MeasureError;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MAX_BYTES: u64 = 1_048_576;
const WINDOW_BYTES: u32 = 65_536;

pub struct Manifest {
    pub contents: Option<String>,
    pub marker: Option<String>,
}

pub fn read(path: &Path) -> Result<Manifest, MeasureError> {
    let Ok(mut file) = File::open(path) else {
        return Ok(Manifest {
            contents: None,
            marker: None,
        });
    };
    let size = file.metadata()?.len();
    if size > MAX_BYTES {
        return Ok(Manifest {
            contents: None,
            marker: Some(window_marker(&mut file, size)?),
        });
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(Manifest {
        contents: String::from_utf8(bytes).ok(),
        marker: None,
    })
}

fn window_marker(file: &mut File, size: u64) -> Result<String, MeasureError> {
    let mut hasher = Sha256::new();
    hasher.update(size.to_le_bytes());
    std::io::copy(&mut file.take(u64::from(WINDOW_BYTES)), &mut hasher)?;
    file.seek(SeekFrom::End(-(i64::from(WINDOW_BYTES))))?;
    std::io::copy(&mut file.take(u64::from(WINDOW_BYTES)), &mut hasher)?;
    Ok(format!("oversized:{size}:{:x}", hasher.finalize()))
}
