use std::env;
use std::fmt::{self, Display};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct Roots {
    repository: PathBuf,
    deployment_repository: PathBuf,
    home: PathBuf,
}

impl Roots {
    pub fn new(repository: impl Into<PathBuf>, home: impl Into<PathBuf>) -> Self {
        let repository = repository.into();
        Self {
            deployment_repository: repository.clone(),
            repository,
            home: home.into(),
        }
    }

    pub fn from_environment() -> Result<Self, RootsError> {
        let repository = env::current_dir()
            .map_err(|_| RootsError::new("repository: current directory is unavailable"))?;
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| RootsError::new("HOME: environment variable is required"))?;
        if home.as_os_str().is_empty() {
            return Err(RootsError::new(
                "HOME: environment variable cannot be empty",
            ));
        }
        if !home.is_absolute() {
            return Err(RootsError::new(
                "HOME: environment variable must be an absolute path",
            ));
        }
        let deployment_repository = deployment_repository(&repository, &home)?;
        Ok(Self {
            repository,
            deployment_repository,
            home,
        })
    }

    pub fn repository(&self) -> &Path {
        &self.repository
    }

    pub fn deployment_repository(&self) -> &Path {
        &self.deployment_repository
    }

    pub fn home(&self) -> &Path {
        &self.home
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RootsError(String);

impl RootsError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for RootsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RootsError {}

fn deployment_repository(repository: &Path, home: &Path) -> Result<PathBuf, RootsError> {
    let manifest = home.join(".arnes.yaml");
    let metadata = match fs::symlink_metadata(&manifest) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(repository.to_owned()),
        Err(_) => {
            return Err(RootsError::new(
                "repository: deployed .arnes.yaml could not be inspected",
            ));
        }
    };
    if !metadata.file_type().is_symlink() {
        return Ok(repository.to_owned());
    }
    let manifest = fs::canonicalize(&manifest).map_err(|_| {
        RootsError::new("repository: deployed .arnes.yaml symlink could not be resolved")
    })?;
    manifest
        .file_name()
        .filter(|name| *name == ".arnes.yaml")
        .and_then(|_| manifest.parent())
        .filter(|directory| directory.file_name().is_some_and(|name| name == "home"))
        .and_then(Path::parent)
        .map(Path::to_owned)
        .ok_or_else(|| {
            RootsError::new("repository: deployed .arnes.yaml must resolve from home/.arnes.yaml")
        })
}
