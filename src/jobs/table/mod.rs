//! Mutable table of background / suspended pipeline jobs.

mod disown;
mod resolve;
mod wait;

use self::resolve::{job_marker, resolve};
use self::wait::{poll_job, wait_foreground_job};

use std::io::{self, Write};
use std::process::Child;

/// How the user referred to a job (`fg`, `bg`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSpec {
    /// Most recent job (`fg` / `bg` with no args).
    Current,
    /// Explicit job id (`%1` or `1`).
    Id(usize),
}

/// Running vs suspended (Ctrl-Z) job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
}

/// Why a job-table operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobTableError {
    NoJobs,
    NotFound,
    Ambiguous,
}

impl JobTableError {
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::NoJobs => "No current job.",
            Self::NotFound => "No such job.",
            Self::Ambiguous => "Ambiguous job specifier.",
        }
    }
}

/// One background / suspended pipeline.
pub(super) struct Job {
    pub(super) id: usize,
    pub(super) command: String,
    pub(super) children: Vec<Child>,
    pub(super) pgid: i32,
    pub(super) state: JobState,
}

/// Session job table (not cloned into subshells).
#[derive(Debug, Default)]
pub struct JobTable {
    next_id: usize,
    jobs: Vec<Job>,
}

impl JobTable {
    /// Register a new job; returns its id and process-group id for the banner.
    pub fn add(&mut self, command: String, children: Vec<Child>, state: JobState) -> (usize, i32) {
        self.next_id += 1;
        let id = self.next_id;
        let pgid = children.first().map(|c| c.id() as i32).unwrap_or(0);
        self.jobs.push(Job {
            id,
            command,
            children,
            pgid,
            state,
        });
        (id, pgid)
    }

    /// Print jobs to `stdout` (Running / Suspended). With `long`, include PIDs.
    pub fn print_jobs(&mut self, stdout: &mut impl Write, long: bool) -> io::Result<()> {
        let _ = self.take_notifications();
        let last = self.jobs.len().saturating_sub(1);
        for (index, job) in self.jobs.iter().enumerate() {
            let marker = job_marker(index, last);
            let state = match job.state {
                JobState::Running => "Running",
                JobState::Stopped => "Suspended",
            };
            if long {
                write_job_long(stdout, job, marker, state)?;
            } else {
                writeln!(
                    stdout,
                    "[{}] {} {state:<16}      {}",
                    job.id, marker, job.command
                )?;
            }
        }
        Ok(())
    }
}

fn write_job_long(stdout: &mut impl Write, job: &Job, marker: char, state: &str) -> io::Result<()> {
    let mut pids = job.children.iter().map(|c| c.id());
    let Some(first) = pids.next() else {
        writeln!(
            stdout,
            "[{}] {} {state:<16}      {}",
            job.id, marker, job.command
        )?;
        return Ok(());
    };
    write!(
        stdout,
        "[{}] {} {first:>5} {state:<16}      {}",
        job.id, marker, job.command
    )?;
    writeln!(stdout)?;
    for pid in pids {
        writeln!(stdout, "       {pid:>5}")?;
    }
    Ok(())
}

impl JobTable {
    /// Bring a job to the foreground; may leave it Suspended again on Ctrl-Z.
    pub fn foreground(
        &mut self,
        spec: JobSpec,
        stderr: &mut impl Write,
    ) -> io::Result<Result<u8, JobTableError>> {
        let index = match resolve(&self.jobs, spec) {
            Ok(i) => i,
            Err(err) => return Ok(Err(err)),
        };
        let mut job = self.jobs.remove(index);
        writeln!(stderr, "{}", job.command)?;
        match wait_foreground_job(&mut job)? {
            wait::FgOutcome::Done(status) => Ok(Ok(status)),
            wait::FgOutcome::Stopped => {
                let status = crate::jobs::sigtstp_status();
                job.state = JobState::Stopped;
                writeln!(
                    stderr,
                    "[{}]+  Suspended                 {}",
                    job.id, job.command
                )?;
                self.jobs.push(job);
                Ok(Ok(status))
            }
        }
    }

    /// Continue a suspended job in the background.
    pub fn background(
        &mut self,
        spec: JobSpec,
        stderr: &mut impl Write,
    ) -> io::Result<Result<(), JobTableError>> {
        let index = match resolve(&self.jobs, spec) {
            Ok(i) => i,
            Err(err) => return Ok(Err(err)),
        };
        let job = &mut self.jobs[index];
        crate::jobs::continue_background(job.pgid)?;
        job.state = JobState::Running;
        writeln!(stderr, "[{}]+ {} &", job.id, job.command)?;
        Ok(Ok(()))
    }

    /// Reap exited jobs; return notices for the REPL.
    pub fn take_notifications(&mut self) -> Vec<(usize, String, u8)> {
        let mut notices = Vec::new();
        let mut still = Vec::new();
        for mut job in self.jobs.drain(..) {
            match poll_job(&mut job) {
                Ok(Some(status)) => notices.push((job.id, job.command, status)),
                _ => still.push(job),
            }
        }
        self.jobs = still;
        notices
    }
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job")
            .field("id", &self.id)
            .field("command", &self.command)
            .field("pgid", &self.pgid)
            .field("state", &self.state)
            .field("children", &self.children.len())
            .finish()
    }
}
