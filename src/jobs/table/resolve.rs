//! Resolve a [`super::JobSpec`] to a table index.

use super::{Job, JobSpec, JobTableError};

pub(super) fn resolve(jobs: &[Job], spec: JobSpec) -> Result<usize, JobTableError> {
    match spec {
        JobSpec::Current => {
            if jobs.is_empty() {
                Err(JobTableError::NoJobs)
            } else {
                Ok(jobs.len() - 1)
            }
        }
        JobSpec::Id(id) => resolve_id(jobs, id),
    }
}

fn resolve_id(jobs: &[Job], id: usize) -> Result<usize, JobTableError> {
    let mut found = None;
    for (index, job) in jobs.iter().enumerate() {
        if job.id == id {
            if found.is_some() {
                return Err(JobTableError::Ambiguous);
            }
            found = Some(index);
        }
    }
    found.ok_or(JobTableError::NotFound)
}

pub(super) const fn job_marker(index: usize, last: usize) -> char {
    if index == last {
        '+'
    } else if last > 0 && index + 1 == last {
        '-'
    } else {
        ' '
    }
}
