//! Remove a job from the table without signaling it.

use super::resolve::resolve;
use super::{JobSpec, JobTable, JobTableError};

impl JobTable {
    /// Drop a job from the table; does not kill or wait for the process.
    pub fn disown(&mut self, spec: JobSpec) -> Result<(), JobTableError> {
        let index = resolve(&self.jobs, spec)?;
        let _ = self.jobs.remove(index);
        Ok(())
    }
}
