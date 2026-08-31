use super::{ProcessOutput, ProcessRunner};
use budget::ProcessBudget;
use cleanup::SystemGroupController;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use supervisor::{SystemCommandSpawner, run_command};

mod budget;
mod cleanup;
mod readers;
mod supervisor;

pub struct DeadlineProcessRunner {
    budget: ProcessBudget,
}

impl DeadlineProcessRunner {
    pub fn new(deadline: Instant) -> Self {
        Self {
            budget: ProcessBudget::new(deadline),
        }
    }
}

impl ProcessRunner for DeadlineProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput> {
        let mut command = Command::new(program);
        command
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        if let Some(directory) = current_directory {
            command.current_dir(directory);
        }
        run_command(
            &mut command,
            self.budget,
            &SystemCommandSpawner,
            &SystemGroupController,
        )
    }

    fn remaining_time(&self) -> Option<Duration> {
        Some(self.budget.remaining_work_at_observation())
    }
}
