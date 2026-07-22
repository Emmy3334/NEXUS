//! Wait / poll helpers for a single job.

use super::{Job, JobState};
use crate::exec::exit_status_code;
use crate::jobs::{continue_background, wait_pipeline, FgWait};

use std::io;

pub(super) enum FgOutcome {
    Done(u8),
    Stopped,
}

pub(super) fn wait_foreground_job(job: &mut Job) -> io::Result<FgOutcome> {
    if job.children.is_empty() {
        return Ok(FgOutcome::Done(0));
    }
    if job.state == JobState::Stopped {
        continue_background(job.pgid)?;
    }
    match wait_pipeline(&mut job.children, job.pgid)? {
        FgWait::Done(status) => {
            for child in &mut job.children {
                let _ = child.try_wait();
            }
            job.children.clear();
            Ok(FgOutcome::Done(status))
        }
        FgWait::Stopped => Ok(FgOutcome::Stopped),
    }
}

pub(super) fn poll_job(job: &mut Job) -> io::Result<Option<u8>> {
    if job.state == JobState::Stopped {
        return Ok(None);
    }
    let count = job.children.len();
    let mut last = 0u8;
    for (index, child) in job.children.iter_mut().enumerate() {
        match child.try_wait()? {
            Some(status) => {
                if index + 1 == count {
                    last = exit_status_code(status);
                }
            }
            None => return Ok(None),
        }
    }
    Ok(Some(last))
}
