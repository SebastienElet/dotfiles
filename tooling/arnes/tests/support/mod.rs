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
    /// # Errors
    /// Returns an error if the temporary fixture directories cannot be created.
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let repository = root.path().join("repository");
        let home = root.path().join("home");
        fs::create_dir(&repository)?;
        fs::create_dir(&home)?;
        Ok(Self {
            _root: root,
            repository,
            home,
        })
    }
    /// # Errors
    /// Returns an error if the fixture command cannot be executed.
    pub fn command<I, S>(&self, args: I) -> Result<Output, Box<dyn std::error::Error + Send + Sync>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command_from(&self.repository, args)
    }
    /// # Errors
    /// Returns an error if the fixture command cannot be executed.
    pub fn command_from<I, S>(
        &self,
        directory: &Path,
        args: I,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(args)
            .current_dir(directory)
            .env_clear()
            .env("HOME", &self.home)
            .env("PATH", self.home.join("bin"))
            .output()?)
    }
    /// # Errors
    /// Returns an error if the fixture file or its parent directory cannot be written.
    pub fn write_home(
        &self,
        path: impl AsRef<Path>,
        contents: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        write(&fixture_path(&self.home, path), contents)?;
        Ok(())
    }
    /// # Errors
    /// Returns an error if the fixture file or its parent directory cannot be written.
    pub fn write_repository(
        &self,
        path: impl AsRef<Path>,
        contents: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        write(&fixture_path(&self.repository, path), contents)?;
        Ok(())
    }
    #[must_use]
    pub fn repository(&self) -> &Path {
        &self.repository
    }
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }
    /// # Errors
    /// Returns an error if fixture files, links or metadata cannot be read.
    pub fn snapshot(
        &self,
    ) -> Result<BTreeMap<PathBuf, SnapshotEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let mut files = BTreeMap::new();
        collect_files(
            &self.repository,
            &self.repository,
            Path::new("repository"),
            &mut files,
        )?;
        collect_files(&self.home, &self.home, Path::new("home"), &mut files)?;
        Ok(files)
    }
}
fn write(path: &Path, contents: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
    fs::write(path, contents)?;
    Ok(())
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let relative = prefix.join(path.strip_prefix(root)?);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            files.insert(
                relative,
                SnapshotEntry::Directory(entry.metadata()?.permissions().mode()),
            );
            collect_files(root, &path, prefix, files)?;
        } else if file_type.is_symlink() {
            files.insert(relative, SnapshotEntry::Symlink(fs::read_link(path)?));
        } else {
            files.insert(
                relative,
                SnapshotEntry::File(fs::read(path)?, entry.metadata()?.permissions().mode()),
            );
        }
    };
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::Fixture;
    use std::fs;
    use std::os::unix::fs::symlink;
    #[test]
    fn fixture_rejects_absolute_write_paths() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
        let fixture = Fixture::new()?;
        let failure = std::panic::catch_unwind(|| fixture.write_home("/outside", "contents"));
        let payload = failure.err().ok_or("expected fixture path refusal")?;
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .ok_or("expected a textual fixture path refusal")?;
        assert!(message.contains("fixture path must stay within its root"));
        Ok(())
    }
    #[test]
    fn fixture_rejects_parent_write_paths() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
        let fixture = Fixture::new()?;
        let failure =
            std::panic::catch_unwind(|| fixture.write_repository("../outside", "contents"));
        let payload = failure.err().ok_or("expected fixture path refusal")?;
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .ok_or("expected a textual fixture path refusal")?;
        assert!(message.contains("fixture path must stay within its root"));
        Ok(())
    }
    #[test]
    fn fixture_rejects_writes_through_symlinks()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let fixture = Fixture::new()?;
        let outside = tempfile::tempdir()?;
        symlink(outside.path(), fixture.home.join("link"))?;
        let result = std::panic::catch_unwind(|| fixture.write_home("link/outside", "contents"));
        assert!(result.is_err());
        assert!(fs::read_dir(outside.path())?.next().is_none());
        Ok(())
    }
}
