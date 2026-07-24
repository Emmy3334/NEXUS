//! Binary `[[` tests (string and integer comparisons).

pub(super) fn eval(left: &str, op: &str, right: &str) -> Result<bool, &'static str> {
    match op {
        "=" | "==" => Ok(left == right),
        "!=" => Ok(left != right),
        "-eq" => Ok(parse_int(left)? == parse_int(right)?),
        "-ne" => Ok(parse_int(left)? != parse_int(right)?),
        "-lt" => Ok(parse_int(left)? < parse_int(right)?),
        "-le" => Ok(parse_int(left)? <= parse_int(right)?),
        "-gt" => Ok(parse_int(left)? > parse_int(right)?),
        "-ge" => Ok(parse_int(left)? >= parse_int(right)?),
        "<" => Ok(left < right),
        ">" => Ok(left > right),
        _ => Err("unknown binary operator"),
    }
}

fn parse_int(value: &str) -> Result<i64, &'static str> {
    value.parse().map_err(|_| "integer expression expected")
}
