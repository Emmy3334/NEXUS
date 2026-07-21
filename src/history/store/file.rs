//! Histfile save / load / merge (`history -S` / `-L` / `-M`).

use super::History;

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl History {
    /// Replace `path` with the current list (`#<unix>` then line).
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut file = File::create(path)?;
        for entry in &self.entries {
            let secs = entry
                .time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            writeln!(file, "#{secs}")?;
            writeln!(file, "{}", entry.line)?;
        }
        Ok(())
    }

    /// Append events from a histfile (`history -L`).
    pub fn load_append(&mut self, path: &Path) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut pending_time: Option<SystemTime> = None;
        for line in reader.lines() {
            let line = line?;
            if let Some(secs) = parse_stamp(&line) {
                pending_time = Some(UNIX_EPOCH + Duration::from_secs(secs));
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let time = pending_time.take().unwrap_or_else(SystemTime::now);
            self.push_at(line, time);
        }
        Ok(())
    }

    /// Merge file events by timestamp (`history -M`).
    pub fn merge_from(&mut self, path: &Path) -> io::Result<()> {
        let mut other = History::default();
        other.load_append(path)?;
        self.entries.extend(other.entries);
        self.entries.sort_by_key(|e| e.time);
        Ok(())
    }
}

fn parse_stamp(line: &str) -> Option<u64> {
    let rest = line.strip_prefix('#')?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok()
}
