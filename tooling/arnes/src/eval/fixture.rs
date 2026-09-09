use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Component, Path, PathBuf},
};

pub struct Fixture {
    pub root: PathBuf,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub env: BTreeMap<String, String>,
    pub observations: PathBuf,
    _directory: tempfile::TempDir,
}

impl Fixture {
    pub fn prepare(
        files: &BTreeMap<String, String>,
        instructions: &str,
        skill: &str,
        executable: &Path,
    ) -> Result<Self, String> {
        let directory = tempfile::Builder::new()
            .prefix("harness-eval-")
            .tempdir()
            .map_err(|error| error.to_string())?;
        let root = directory.path().to_path_buf();
        let workspace = root.join("workspace");
        let home = root.join("home");
        fs::create_dir_all(&workspace).map_err(|error| error.to_string())?;
        fs::create_dir_all(home.join(".codex")).map_err(|error| error.to_string())?;
        let mut installed = files.clone();
        installed.insert("AGENTS.md".into(), instructions.into());
        installed.insert(".agents/skills/code-search/SKILL.md".into(), skill.into());
        install_files(&workspace, &installed)?;
        let bin = install_shims(&workspace, executable)?;
        let observations = workspace.join(".observations.jsonl");
        write_new(&observations, "", 0o600)?;
        let env = fixture_environment(&home, &workspace, &observations, &bin, executable)?;
        Ok(Self {
            root,
            home,
            workspace,
            env,
            observations,
            _directory: directory,
        })
    }
}

fn install_files(workspace: &Path, files: &BTreeMap<String, String>) -> Result<(), String> {
    for (path, content) in files {
        let path = Path::new(path);
        if path.as_os_str().is_empty()
            || !path
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err(format!(
                "Fixture path escapes workspace: {}",
                path.display()
            ));
        }
        let destination = workspace.join(path);
        fs::create_dir_all(destination.parent().ok_or("Missing fixture parent")?)
            .map_err(|error| error.to_string())?;
        write_new(&destination, content, 0o644)?;
    }
    Ok(())
}

fn install_shims(workspace: &Path, executable: &Path) -> Result<PathBuf, String> {
    let bin = workspace.join(".eval-bin");
    fs::create_dir(&bin).map_err(|error| error.to_string())?;
    let executable = path_text(executable)?;
    let executable = format!("'{}'", executable.replace('\'', "'\\''"));
    for tool in ["cat", "rg", "fd", "colgrep-search"] {
        write_new(
            &bin.join(tool),
            &format!("#!/usr/bin/env bash\nexec {executable} eval shim {tool} -- \"$@\"\n"),
            0o755,
        )?;
    }
    Ok(bin)
}

fn fixture_environment(
    home: &Path,
    workspace: &Path,
    observations: &Path,
    bin: &Path,
    executable: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let executable_parent = executable
        .parent()
        .ok_or("Executable requires an absolute path")?;
    Ok(BTreeMap::from([
        ("HOME".into(), path_text(home)?),
        ("CODEX_HOME".into(), path_text(&home.join(".codex"))?),
        ("XDG_CONFIG_HOME".into(), path_text(&home.join(".config"))?),
        (
            "PATH".into(),
            format!(
                "{}:{}:/usr/bin:/bin",
                path_text(bin)?,
                path_text(executable_parent)?
            ),
        ),
        ("HARNESS_EVAL_WORKSPACE".into(), path_text(workspace)?),
        ("HARNESS_EVAL_OBSERVATIONS".into(), path_text(observations)?),
        ("LANG".into(), "C.UTF-8".into()),
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
    ]))
}

fn path_text(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("Path is not UTF-8: {}", path.display()))
}

fn write_new(path: &Path, text: &str, mode: u32) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
