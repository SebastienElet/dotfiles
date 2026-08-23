use std::env;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct Roots {
    repository: PathBuf,
    home: PathBuf,
}

impl Roots {
    pub fn new(repository: impl Into<PathBuf>, home: impl Into<PathBuf>) -> Self {
        Self {
            repository: repository.into(),
            home: home.into(),
        }
    }

    pub fn from_environment() -> Result<Self, RootsError> {
        let repository = env::current_dir()
            .map_err(|_| RootsError("repository: current directory is unavailable"))?;
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(RootsError("HOME: environment variable is required"))?;
        if home.as_os_str().is_empty() {
            return Err(RootsError("HOME: environment variable cannot be empty"));
        }
        if !home.is_absolute() {
            return Err(RootsError(
                "HOME: environment variable must be an absolute path",
            ));
        }
        Ok(Self::new(repository, home))
    }

    pub fn repository(&self) -> &Path {
        &self.repository
    }

    pub fn home(&self) -> &Path {
        &self.home
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RootsError(&'static str);

impl Display for RootsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for RootsError {}
