use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

type Tokens = BTreeMap<String, String>;
type Bindings = BTreeMap<String, Tokens>;

#[derive(Debug)]
pub struct Conflict(&'static str);

impl fmt::Display for Conflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for Conflict {}

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Read {},
    Replace {
        pane_id: String,
        #[serde(deserialize_with = "crate::unique_maps::bindings")]
        expected: Bindings,
        #[serde(deserialize_with = "crate::unique_maps::tokens")]
        tokens: Tokens,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u32,
    #[serde(deserialize_with = "crate::unique_maps::bindings")]
    bindings: Bindings,
}

pub fn dispatch(directory: &Path, request: Request) -> Result<Bindings, Box<dyn Error>> {
    match request {
        Request::Read {} => Ok(read_state(directory)?.bindings),
        Request::Replace {
            pane_id,
            expected,
            tokens,
        } => {
            fs::create_dir_all(directory)?;
            let lock = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(directory.join("bindings.lock"))?;
            lock.try_lock().map_err(|error| -> Box<dyn Error> {
                match error {
                    fs::TryLockError::WouldBlock => {
                        Box::new(Conflict("Binding writer unavailable"))
                    }
                    fs::TryLockError::Error(error) => Box::new(error),
                }
            })?;
            let mut state = read_state(directory)?;
            if state.bindings != expected {
                return Err(Box::new(Conflict(
                    "Binding changed concurrently; reread before updating",
                )));
            }
            state.bindings.insert(pane_id, tokens);
            write_state(directory, &state)?;
            Ok(state.bindings)
        }
    }
}

fn read_state(directory: &Path) -> Result<State, Box<dyn Error>> {
    let path = directory.join("bindings.json");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err("Binding state must be a regular file; original path retained".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(State {
                version: 1,
                bindings: Bindings::new(),
            });
        }
        Err(error) => return Err(error.into()),
    }
    let file = File::open(path)?;
    let state: State = serde_json::from_reader(BufReader::new(file))?;
    if state.version != 1 {
        return Err("Unsupported binding state version; original state retained".into());
    }
    Ok(state)
}

fn write_state(directory: &Path, state: &State) -> Result<(), Box<dyn Error>> {
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer(temporary.as_file_mut(), state)?;
    temporary.as_file().sync_all()?;
    temporary.persist(directory.join("bindings.json"))?;
    File::open(directory)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_values_survive_another_read() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let tokens = Tokens::from([("title".into(), format!(" {}\n\u{7f} ", "é".repeat(150)))]);
        dispatch(
            directory.path(),
            Request::Replace {
                pane_id: "w2:p1".into(),
                expected: Bindings::new(),
                tokens: tokens.clone(),
            },
        )?;
        assert_eq!(
            dispatch(directory.path(), Request::Read {})?.get("w2:p1"),
            Some(&tokens)
        );
        Ok(())
    }
}
