use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

pub struct Fixture {
    _root: TempDir,
    repository: PathBuf,
    home: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SnapshotEntry {
    Directory(u32),
    File(Vec<u8>, u32),
    Symlink(PathBuf),
}

impl Fixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let repository = root.path().join("repository");
        let home = root.path().join("home");
        fs::create_dir(&repository).unwrap();
        fs::create_dir(&home).unwrap();
        Self {
            _root: root,
            repository,
            home,
        }
    }

    pub fn command<I, S>(&self, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(args)
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
    }

    pub fn write_home(&self, path: impl AsRef<Path>, contents: &str) {
        write(&fixture_path(&self.home, path), contents);
    }

    pub fn write_repository(&self, path: impl AsRef<Path>, contents: &str) {
        write(&fixture_path(&self.repository, path), contents);
    }

    pub fn repository(&self) -> &Path {
        &self.repository
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn snapshot(&self) -> BTreeMap<PathBuf, SnapshotEntry> {
        let mut files = BTreeMap::new();
        collect_files(
            &self.repository,
            &self.repository,
            Path::new("repository"),
            &mut files,
        );
        collect_files(&self.home, &self.home, Path::new("home"), &mut files);
        files
    }
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn fixture_path(root: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    assert!(
        !path.as_os_str().is_empty()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
        "fixture path must stay within its root"
    );
    let mut destination = root.to_owned();
    for component in path.components() {
        destination.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&destination) {
            assert!(
                !metadata.file_type().is_symlink(),
                "fixture path cannot traverse a symlink"
            );
        }
    }
    destination
}

fn collect_files(
    root: &Path,
    directory: &Path,
    prefix: &Path,
    files: &mut BTreeMap<PathBuf, SnapshotEntry>,
) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative = prefix.join(path.strip_prefix(root).unwrap());
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            files.insert(
                relative,
                SnapshotEntry::Directory(entry.metadata().unwrap().permissions().mode()),
            );
            collect_files(root, &path, prefix, files);
        } else if file_type.is_symlink() {
            files.insert(
                relative,
                SnapshotEntry::Symlink(fs::read_link(path).unwrap()),
            );
        } else {
            files.insert(
                relative,
                SnapshotEntry::File(
                    fs::read(path).unwrap(),
                    entry.metadata().unwrap().permissions().mode(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Fixture;
    use std::fs;
    use std::os::unix::fs::symlink;

    #[test]
    #[should_panic(expected = "fixture path must stay within its root")]
    fn fixture_rejects_absolute_write_paths() {
        Fixture::new().write_home("/outside", "contents");
    }

    #[test]
    #[should_panic(expected = "fixture path must stay within its root")]
    fn fixture_rejects_parent_write_paths() {
        Fixture::new().write_repository("../outside", "contents");
    }

    #[test]
    fn fixture_rejects_writes_through_symlinks() {
        let fixture = Fixture::new();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), fixture.home.join("link")).unwrap();

        let result = std::panic::catch_unwind(|| fixture.write_home("link/outside", "contents"));

        assert!(result.is_err());
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
    }
}
