use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Bitbucket,
    Github,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub stale_days: u64,
    pub next_count: usize,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Track {
    pub name: String,
    pub teams: Vec<String>,
    pub repos: Vec<RepoConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RepoConfig {
    pub provider: Provider,
    pub path: String,
    #[serde(default = "requires_linear_by_default")]
    pub requires_linear: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RepoKey {
    pub provider: Provider,
    pub path: String,
}

fn requires_linear_by_default() -> bool {
    true
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let home = std::env::var_os("HOME");
        Self::load_from_home(home.as_deref())
    }

    fn load_from_home(home: Option<&OsStr>) -> Result<Self, Box<dyn Error>> {
        let home = home.ok_or("HOME environment variable is not set")?;
        let home = Path::new(home);
        if home.as_os_str().is_empty() {
            return Err("HOME must not be empty".into());
        }
        if !home.is_absolute() {
            return Err("HOME must be an absolute path".into());
        }
        let path = home.join(".config/daily-routine/config.toml");
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read configuration {}: {error}", path.display()))?;
        Self::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Self, Box<dyn Error>> {
        let config: Self = toml::from_str(source)?;

        if config.tracks.is_empty() {
            return Err("configuration must contain at least one track".into());
        }

        for track in &config.tracks {
            if track.name.trim().is_empty() {
                return Err("track name must not be empty".into());
            }
            if track.teams.iter().any(|team| team.trim().is_empty()) {
                return Err(format!("team key must not be empty in track {}", track.name).into());
            }
            if track.teams.iter().any(|team| team != team.trim()) {
                return Err(format!(
                    "team key must not contain surrounding whitespace in track {}",
                    track.name
                )
                .into());
            }
            for repo in &track.repos {
                let Some((owner, name)) = repo.path.split_once('/') else {
                    return Err(format!(
                        "repository path must be owner/name with exactly one slash: {}",
                        repo.path
                    )
                    .into());
                };
                if owner.trim().is_empty()
                    || name.trim().is_empty()
                    || owner != owner.trim()
                    || name != name.trim()
                    || name.contains('/')
                {
                    return Err(format!(
                        "repository path must be owner/name with exactly one slash: {}",
                        repo.path
                    )
                    .into());
                }
            }
        }

        Ok(config)
    }

    pub fn unique_repos(&self) -> Vec<&RepoConfig> {
        let mut seen = HashSet::new();
        let mut repos = Vec::new();

        for track in &self.tracks {
            for repo in &track.repos {
                let key = RepoKey {
                    provider: repo.provider,
                    path: repo.path.clone(),
                };
                if seen.insert(key) {
                    repos.push(repo);
                }
            }
        }

        repos
    }

    pub fn team_keys(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut keys = Vec::new();

        for team in self.tracks.iter().flat_map(|track| &track.teams) {
            let key = team.to_uppercase();
            if seen.insert(key.clone()) {
                keys.push(key);
            }
        }

        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TemporaryHome(PathBuf);

    impl TemporaryHome {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "daily-routine-config-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn config_path(&self) -> PathBuf {
            self.0.join(".config/daily-routine/config.toml")
        }

        fn write_config(&self, source: &str) {
            let path = self.config_path();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TemporaryHome {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn parses_example_and_preserves_declared_scope() {
        let config = Config::parse(include_str!("../config.example.toml")).unwrap();

        assert_eq!(config.stale_days, 7);
        assert_eq!(config.next_count, 3);
        assert_eq!(config.tracks.len(), 3);
        assert_eq!(config.tracks[0].name, "Application");
        assert_eq!(config.tracks[1].name, "Platform");
        assert_eq!(config.tracks[2].name, "Standalone");
        assert!(config.tracks[0].repos[0].requires_linear);
        assert!(!config.tracks[2].repos[0].requires_linear);
        assert!(config.tracks[2].teams.is_empty());

        let repos = config.unique_repos();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].provider, Provider::Bitbucket);
        assert_eq!(repos[0].path, "ExampleOrg/shared-app");
        assert_eq!(repos[1].provider, Provider::Github);
        assert_eq!(repos[1].path, "ExampleOrg/standalone");

        assert_eq!(config.team_keys(), ["APP", "API", "PLT", "OPS"]);
        let mut mixed_case_config = config.clone();
        mixed_case_config.tracks[1].teams.insert(0, "app".into());
        assert_eq!(mixed_case_config.team_keys(), ["APP", "API", "PLT", "OPS"]);
    }

    #[test]
    fn rejects_repo_path_without_owner_and_name() {
        let source = r#"
            stale_days = 7
            next_count = 3

            [[tracks]]
            name = "Application"
            teams = ["APP"]

            [[tracks.repos]]
            provider = "bitbucket"
            path = "shared-app"
        "#;

        let error = Config::parse(source).unwrap_err();

        assert!(error.to_string().contains("owner/name"));
    }

    #[test]
    fn rejects_empty_tracks() {
        let source = "stale_days = 7\nnext_count = 3\ntracks = []\n";

        let error = Config::parse(source).unwrap_err();

        assert!(error.to_string().contains("at least one track"));
    }

    #[test]
    fn rejects_empty_track_names() {
        let source = r#"
            stale_days = 7
            next_count = 3

            [[tracks]]
            name = "  "
            teams = []
            repos = []
        "#;

        let error = Config::parse(source).unwrap_err();

        assert!(error.to_string().contains("track name"));
    }

    #[test]
    fn rejects_empty_team_keys() {
        let source = r#"
            stale_days = 7
            next_count = 3

            [[tracks]]
            name = "Application"
            teams = ["APP", "  "]
            repos = []
        "#;

        let error = Config::parse(source).unwrap_err();

        assert!(error.to_string().contains("team key"));
    }

    #[test]
    fn rejects_team_keys_with_surrounding_whitespace() {
        let source = r#"
            stale_days = 7
            next_count = 3

            [[tracks]]
            name = "Platform"
            teams = [" OPS "]
            repos = []
        "#;

        let error = Config::parse(source).unwrap_err();

        assert!(error.to_string().contains("surrounding whitespace"));
    }

    #[test]
    fn rejects_malformed_repo_paths() {
        for path in [
            "/shared-app",
            "ExampleOrg/",
            "org/team/repo",
            " owner/repo ",
        ] {
            let source = format!(
                r#"
                    stale_days = 7
                    next_count = 3

                    [[tracks]]
                    name = "Application"
                    teams = []

                    [[tracks.repos]]
                    provider = "bitbucket"
                    path = "{path}"
                "#
            );

            let error = Config::parse(&source).unwrap_err();

            assert!(error.to_string().contains("owner/name"));
        }
    }

    #[test]
    fn load_rejects_missing_home() {
        let error = Config::load_from_home(None).unwrap_err();

        assert!(error.to_string().contains("HOME"));
    }

    #[test]
    fn load_rejects_empty_home() {
        let error = Config::load_from_home(Some(OsStr::new(""))).unwrap_err();

        assert_eq!(error.to_string(), "HOME must not be empty");
    }

    #[test]
    fn load_rejects_relative_home() {
        let error = Config::load_from_home(Some(OsStr::new("relative/home"))).unwrap_err();

        assert_eq!(error.to_string(), "HOME must be an absolute path");
    }

    #[test]
    fn load_reports_missing_config_file() {
        let home = TemporaryHome::new();

        let error = Config::load_from_home(Some(home.path().as_os_str())).unwrap_err();

        assert!(error.to_string().contains("failed to read configuration"));
    }

    #[test]
    fn load_reports_unreadable_config_file() {
        let home = TemporaryHome::new();
        fs::create_dir_all(home.config_path()).unwrap();

        let error = Config::load_from_home(Some(home.path().as_os_str())).unwrap_err();

        assert!(error.to_string().contains("failed to read configuration"));
    }

    #[test]
    fn load_reports_invalid_toml() {
        let home = TemporaryHome::new();
        home.write_config("not valid TOML");

        let error = Config::load_from_home(Some(home.path().as_os_str())).unwrap_err();

        assert!(error.to_string().contains("TOML parse error"));
    }

    #[test]
    fn load_reports_invalid_configuration() {
        let home = TemporaryHome::new();
        home.write_config("stale_days = 7\nnext_count = 3\ntracks = []\n");

        let error = Config::load_from_home(Some(home.path().as_os_str())).unwrap_err();

        assert!(error.to_string().contains("at least one track"));
    }

    #[test]
    fn load_reads_valid_configuration() {
        let home = TemporaryHome::new();
        home.write_config(include_str!("../config.example.toml"));

        let config = Config::load_from_home(Some(home.path().as_os_str())).unwrap();

        assert_eq!(config.tracks.len(), 3);
    }
}
