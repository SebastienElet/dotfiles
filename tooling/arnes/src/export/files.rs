use super::ExportError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn validate_export_directory(output: &Path) -> Result<(), ExportError> {
    let metadata = fs::symlink_metadata(output).map_err(|error| {
        ExportError::new(format!(
            "export directory does not exist or is unreadable: {error}"
        ))
    })?;
    if !metadata.file_type().is_dir() {
        return Err(ExportError::new("export path is not an owned directory"));
    }
    let parent = output
        .parent()
        .ok_or_else(|| ExportError::new("export directory has no parent"))?;
    let expected = fs::canonicalize(parent)
        .map_err(|error| ExportError::new(format!("export parent is unreadable: {error}")))?
        .join(
            output
                .file_name()
                .ok_or_else(|| ExportError::new("export directory has no name"))?,
        );
    let resolved = fs::canonicalize(output)
        .map_err(|error| ExportError::new(format!("export directory is unreadable: {error}")))?;
    if resolved != expected {
        return Err(ExportError::new("export directory escapes its parent"));
    }
    Ok(())
}

pub(super) fn check_snapshot(
    output: &Path,
    expected: &BTreeMap<String, String>,
) -> Result<(), ExportError> {
    let entries = fs::read_dir(output).map_err(|error| {
        ExportError::new(format!(
            "export directory does not exist or is unreadable: {error}"
        ))
    })?;
    let mut actual_names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            ExportError::new(format!("export directory could not be inspected: {error}"))
        })?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| ExportError::new("export directory contains a non-UTF-8 artifact name"))?;
        if !entry
            .file_type()
            .map_err(|error| ExportError::new(format!("{name} could not be inspected: {error}")))?
            .is_file()
        {
            return Err(ExportError::new(format!(
                "unexpected non-file export artifact: {name}"
            )));
        }
        actual_names.insert(name);
    }
    let expected_names = expected.keys().cloned().collect::<BTreeSet<_>>();
    if let Some(name) = actual_names.difference(&expected_names).next() {
        return Err(ExportError::new(format!(
            "unexpected export artifact: {name}"
        )));
    }
    if let Some(name) = expected_names.difference(&actual_names).next() {
        return Err(ExportError::new(format!(
            "export artifact {name} is missing"
        )));
    }
    for (name, expected_contents) in expected {
        let actual = fs::read(output.join(name)).map_err(|error| {
            ExportError::new(format!("export artifact {name} could not be read: {error}"))
        })?;
        if actual != expected_contents.as_bytes() {
            return Err(ExportError::new(format!("export artifact {name} is stale")));
        }
    }
    Ok(())
}

pub(super) fn publish_snapshot(
    output: &Path,
    snapshot: &BTreeMap<String, String>,
) -> Result<(), ExportError> {
    if output.exists() {
        validate_export_directory(output)?;
    } else if fs::symlink_metadata(output).is_ok() {
        return Err(ExportError::new("export path is not an owned directory"));
    }
    let parent = output
        .parent()
        .ok_or_else(|| ExportError::new("export directory has no parent"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ExportError::new("system clock is before the Unix epoch"))?
        .as_nanos();
    let temporary = parent.join(format!(
        ".harness-export.tmp-{}-{nonce}",
        std::process::id()
    ));
    let backup = parent.join(format!(
        ".harness-export.backup-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&temporary).map_err(|error| {
        ExportError::new(format!("temporary export could not be created: {error}"))
    })?;
    let result = write_snapshot(&temporary, snapshot).and_then(|()| {
        let had_output = output.exists();
        if had_output {
            fs::rename(output, &backup).map_err(|error| {
                ExportError::new(format!("existing export could not be preserved: {error}"))
            })?;
        }
        match fs::rename(&temporary, output) {
            Ok(()) => {
                if had_output {
                    fs::remove_dir_all(&backup).map_err(|error| {
                        ExportError::new(format!("obsolete export could not be removed: {error}"))
                    })?;
                }
                Ok(())
            }
            Err(error) => {
                if had_output {
                    let _ = fs::rename(&backup, output);
                }
                Err(ExportError::new(format!(
                    "new export could not be published: {error}"
                )))
            }
        }
    });
    if temporary.exists() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn write_snapshot(
    directory: &Path,
    snapshot: &BTreeMap<String, String>,
) -> Result<(), ExportError> {
    for (name, contents) in snapshot {
        fs::write(directory.join(name), contents).map_err(|error| {
            ExportError::new(format!(
                "export artifact {name} could not be written: {error}"
            ))
        })?;
    }
    Ok(())
}
