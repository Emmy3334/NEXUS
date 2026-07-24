//! Parse completion dump file text.

use super::{CompRegistry, DUMP_VERSION};

use std::collections::BTreeMap;
use std::io;

pub fn parse_dump(text: &str) -> io::Result<Option<CompRegistry>> {
    let mut lines = text.lines();
    let header = lines.next().ok_or_else(|| invalid_dump("missing header"))?;
    let (count, version) = parse_header(header)?;
    if version != DUMP_VERSION {
        return Ok(None);
    }
    let mut words = BTreeMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (cmd, rest) = line
            .split_once('\t')
            .ok_or_else(|| invalid_dump("missing tab"))?;
        let list: Vec<String> = rest.split_whitespace().map(str::to_owned).collect();
        words.insert(cmd.to_owned(), list);
    }
    if words.len() != count {
        return Ok(None);
    }
    Ok(Some(CompRegistry { words }))
}

fn parse_header(header: &str) -> io::Result<(usize, u32)> {
    let mut count = None;
    let mut version = None;
    for part in header.split('\t') {
        if let Some(n) = part.strip_prefix("#files:") {
            count = Some(n.parse::<usize>().map_err(|_| invalid_dump("bad count"))?);
        } else if let Some(v) = part.strip_prefix("version:") {
            version = Some(v.parse::<u32>().map_err(|_| invalid_dump("bad version"))?);
        }
    }
    match (count, version) {
        (Some(c), Some(v)) => Ok((c, v)),
        _ => Err(invalid_dump("bad header")),
    }
}

fn invalid_dump(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}
