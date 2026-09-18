mod claims;
mod classify;
mod content;
mod gate;
mod json;
mod model;
mod process;
mod repository;
mod targets;

use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    error::Error,
    fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(
    version,
    about = "Advisory proof-surface classification and bound receipt validation"
)]
struct Arguments {
    #[command(subcommand)]
    command: Operation,
}
#[derive(Subcommand)]
enum Operation {
    Digest {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Classify {
        #[arg(long, num_args = 1.., conflicts_with_all = ["repository", "base", "head", "worktree"])]
        paths: Option<Vec<String>>,
        #[arg(long)]
        repository: Option<PathBuf>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        worktree: bool,
    },
    Epoch {
        #[arg(long)]
        repository: PathBuf,
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        head: String,
        #[arg(long)]
        include_worktree: bool,
        #[arg(long)]
        policy_root: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Gate {
        #[arg(long)]
        epoch: PathBuf,
        #[arg(long)]
        receipt: PathBuf,
        #[arg(long)]
        policy_root: PathBuf,
    },
}
fn read<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > 32 * 1024 * 1024 {
        return Err("JSON input must be a regular file under 32MiB".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("JSON input exceeds size limit".into());
    }
    let strict: json::StrictValue = serde_json::from_slice(&bytes)?;
    Ok(serde_json::from_value(strict.0)?)
}
fn run(arguments: Arguments) -> Result<Value> {
    match arguments.command {
        Operation::Digest { file, json } => {
            let digest = if json {
                content::bound(&read::<Value>(&file)?)?
            } else {
                content::digest(&fs::read(file)?)
            };
            Ok(serde_json::json!({"digest": digest}))
        }
        Operation::Classify {
            paths,
            repository,
            base,
            head,
            worktree,
        } => {
            let paths = if let Some(paths) = paths {
                paths
            } else {
                let repository =
                    repository.ok_or("--repository is required for Git classification")?;
                let base = base.ok_or("--base is required for Git classification")?;
                let head = match head {
                    Some(head) => head,
                    None if worktree => "HEAD".into(),
                    None => return Err("--head is required for committed classification".into()),
                };
                repository::paths(&repository, &base, &head, worktree)?
            };
            serde_json::to_value(classify::classify(&paths)?).map_err(Into::into)
        }
        Operation::Epoch {
            repository,
            base,
            head,
            include_worktree,
            policy_root,
            output,
        } => {
            let epoch =
                repository::epoch(&repository, &base, &head, include_worktree, &policy_root)?;
            let value = serde_json::to_value(epoch)?;
            if let Some(path) = output {
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)?;
                serde_json::to_writer_pretty(&mut file, &value)?;
                writeln!(file)?;
            }
            Ok(value)
        }
        Operation::Gate {
            epoch,
            receipt,
            policy_root,
        } => gate::gate(&read(&epoch)?, &read(&receipt)?, &policy_root),
    }
}
fn main() -> ExitCode {
    match run(Arguments::parse()) {
        Ok(value) => {
            let mut output = io::stdout().lock();
            if serde_json::to_writer_pretty(&mut output, &value).is_err()
                || writeln!(output).is_err()
            {
                return ExitCode::from(2);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            let value = serde_json::json!({"schema_version":model::VERSION,"decision":"BLOCK","reason":error.to_string()});
            let mut output = io::stderr().lock();
            let _ = serde_json::to_writer_pretty(&mut output, &value);
            let _ = writeln!(output);
            ExitCode::from(2)
        }
    }
}
