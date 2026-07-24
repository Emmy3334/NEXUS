//! Parse `typeset` flags.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Scalar { export: bool },
    Array,
}

pub(super) fn parse(argv: &[String]) -> Result<(Mode, &[String]), &'static str> {
    let mut rest = argv.get(1..).unwrap_or(&[]);
    let mut export = false;
    let mut array = false;
    while let Some(flag) = rest.first().map(String::as_str) {
        match flag {
            "-x" | "--export" => export = true,
            "-a" => array = true,
            other if other.starts_with('-') => return Err("unknown option"),
            _ => break,
        }
        rest = &rest[1..];
    }
    if array && export {
        return Err("combined -a -x unsupported");
    }
    Ok((
        if array {
            Mode::Array
        } else {
            Mode::Scalar { export }
        },
        rest,
    ))
}
