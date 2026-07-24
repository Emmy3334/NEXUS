//! Parse `typeset` flags.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Scalar { export: bool },
    Array,
    Assoc,
}

pub(super) fn parse(argv: &[String]) -> Result<(Mode, &[String]), &'static str> {
    let mut rest = argv.get(1..).unwrap_or(&[]);
    let (mut export, mut array, mut assoc) = (false, false, false);
    while let Some(flag) = rest.first().map(String::as_str) {
        match flag {
            "-x" | "--export" => export = true,
            "-a" => array = true,
            "-A" => assoc = true,
            other if other.starts_with('-') => return Err("unknown option"),
            _ => break,
        }
        rest = &rest[1..];
    }
    Ok((resolve_mode(export, array, assoc)?, rest))
}

fn resolve_mode(export: bool, array: bool, assoc: bool) -> Result<Mode, &'static str> {
    match (export, array, assoc) {
        (_, true, true) => Err("combined -a -A unsupported"),
        (true, true, _) => Err("combined -a -x unsupported"),
        (true, _, true) => Err("combined -A -x unsupported"),
        (_, true, _) => Ok(Mode::Array),
        (_, _, true) => Ok(Mode::Assoc),
        (export, _, _) => Ok(Mode::Scalar { export }),
    }
}
