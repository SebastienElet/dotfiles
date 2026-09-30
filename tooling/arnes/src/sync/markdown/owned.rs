use crate::hooks::io::ConfigFile;
use crate::sync::{SyncEntry, SyncState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

pub(in crate::sync) struct Intent {
    pub id: String,
    pub owner: String,
    pub root: PathBuf,
    pub destination: PathBuf,
    pub contents: String,
}

pub(in crate::sync) struct Prepared {
    intent: Intent,
    previous: Option<Vec<u8>>,
    previous_identity: Option<(u64, u64)>,
    receipt_path: PathBuf,
    previous_receipt: Option<Vec<u8>>,
    current: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u8,
    owner: String,
    root: PathBuf,
    destination: PathBuf,
    hash: String,
    device: u64,
    inode: u64,
}

pub(in crate::sync) fn prepare(intent: Intent) -> Result<Prepared, SyncEntry> {
    let refused = |message| SyncEntry::new(&intent.id, SyncState::Refused, message);
    validate_parent(&intent.root, &intent.destination).map_err(refused)?;
    let root = fs::canonicalize(&intent.root).map_err(|_| refused("scope root is unreadable"))?;
    let destination = root.join(&intent.destination);
    let (previous, previous_identity) = read_regular(&destination).map_err(refused)?;
    let name = intent
        .destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refused("destination name is invalid"))?;
    let receipt_path = intent
        .destination
        .with_file_name(format!(".{name}.arnes.json"));
    let current = previous.as_deref() == Some(intent.contents.as_bytes());
    if current {
        return Ok(Prepared {
            intent,
            previous,
            previous_identity,
            receipt_path,
            previous_receipt: None,
            current,
        });
    }
    let auxiliary = [
        lock_path(&intent.destination).map_err(refused)?,
        lock_path(&receipt_path).map_err(refused)?,
    ];
    for path in &auxiliary {
        validate_lock(&root.join(path)).map_err(refused)?;
    }
    let (previous_receipt, _) = read_regular(&root.join(&receipt_path)).map_err(refused)?;
    if let Some(bytes) = &previous {
        let receipt: Receipt = previous_receipt
            .as_deref()
            .and_then(|bytes| serde_json::from_slice(bytes).ok())
            .ok_or_else(|| refused("divergent file has no verified ownership receipt"))?;
        if receipt.version != 1
            || receipt.owner != intent.owner
            || receipt.root != root
            || receipt.destination != intent.destination
            || receipt.hash != hash(bytes)
            || Some((receipt.device, receipt.inode)) != previous_identity
        {
            return Err(refused("divergent file ownership is uncertain"));
        }
    } else if previous_receipt.is_some() {
        return Err(refused("receipt exists without its owned artifact"));
    }
    Ok(Prepared {
        intent,
        previous,
        previous_identity,
        receipt_path,
        previous_receipt,
        current,
    })
}

impl Prepared {
    pub(in crate::sync) fn expected(&self) -> &str {
        &self.intent.contents
    }

    pub(in crate::sync) fn id(&self) -> &str {
        &self.intent.id
    }

    pub(in crate::sync) fn publish(self) -> SyncEntry {
        if self.current {
            return SyncEntry::new(
                self.intent.id,
                SyncState::Current,
                "artifact is conforming; ownership unchanged",
            );
        }
        let state = match self.write() {
            Ok(()) => SyncState::Applied,
            Err(message) => return SyncEntry::new(self.intent.id, SyncState::Failed, message),
        };
        SyncEntry::new(
            self.intent.id,
            state,
            "managed artifact and ownership receipt published",
        )
    }

    fn write(&self) -> Result<(), &'static str> {
        validate_parent(&self.intent.root, &self.intent.destination)?;
        let file = ConfigFile::open_path(&self.intent.root, &self.intent.destination)
            .map_err(|_| "artifact could not be safely opened")?;
        if file.content() != self.previous.as_deref() || file.identity() != self.previous_identity {
            return Err("artifact changed after preflight");
        }
        let published_identity = file
            .replace_with_identity(self.intent.contents.as_bytes())
            .map_err(|_| "artifact publication failed")?;
        let destination = self.intent.root.join(&self.intent.destination);
        let (contents, identity) = read_regular(&destination)?;
        if contents.as_deref() != Some(self.intent.contents.as_bytes())
            || identity != Some(published_identity)
        {
            return Err("published artifact could not be verified");
        }
        let (device, inode) = identity.ok_or("published artifact is missing")?;
        let receipt = Receipt {
            version: 1,
            owner: self.intent.owner.clone(),
            root: fs::canonicalize(&self.intent.root)
                .map_err(|_| "scope root could not be verified")?,
            destination: self.intent.destination.clone(),
            hash: hash(self.intent.contents.as_bytes()),
            device,
            inode,
        };
        let bytes = serde_json::to_vec(&receipt).map_err(|_| "receipt could not be rendered")?;
        let file = ConfigFile::open_path(&self.intent.root, &self.receipt_path)
            .map_err(|_| "artifact published but ownership receipt could not be opened")?;
        if file.content() != self.previous_receipt.as_deref() {
            return Err("artifact published but ownership receipt changed");
        }
        file.replace(&bytes)
            .map_err(|_| "artifact published but ownership receipt publication failed")
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

type ObservedFile = (Option<Vec<u8>>, Option<(u64, u64)>);

fn lock_path(path: &Path) -> Result<PathBuf, &'static str> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("publication filename is invalid")?;
    Ok(path.with_file_name(format!(".{name}.lock")))
}

fn validate_lock(path: &Path) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.nlink() == 1 => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        _ => Err("publication lock is unreadable or has an unsupported type"),
    }
}

fn read_regular(path: &Path) -> Result<ObservedFile, &'static str> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok((None, None)),
        Err(_) => return Err("artifact or receipt could not be inspected"),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.nlink() != 1 {
        return Err("artifact or receipt must be a single-link regular file");
    }
    let bytes = fs::read(path).map_err(|_| "artifact or receipt could not be read")?;
    Ok((Some(bytes), Some((metadata.dev(), metadata.ino()))))
}

pub(in crate::sync) fn validate_parent(root: &Path, relative: &Path) -> Result<(), &'static str> {
    let metadata = fs::symlink_metadata(root).map_err(|_| "scope root could not be inspected")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("scope root must be a directory, not a symbolic link");
    }
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("destination must remain within its scope root");
    }
    let parent = relative.parent().ok_or("destination has no parent")?;
    let mut path = root.to_owned();
    for part in parent.components() {
        path.push(part.as_os_str());
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Ok(_) => return Err("destination parent is not a regular directory"),
            Err(_) => return Err("destination parent could not be inspected"),
        }
    }
    Ok(())
}
