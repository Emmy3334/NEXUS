//! Comparison helpers and truthiness for while expressions.

pub(super) fn eval_cmp(left: &str, op: &str, right: &str) -> Result<bool, String> {
    match op {
        "==" => Ok(left == right),
        "!=" => Ok(left != right),
        "<" | ">" | "<=" | ">=" => cmp_num(left, right, op),
        _ => Err("while: Expression Syntax.".into()),
    }
}

pub(super) fn truthy(value: &str) -> bool {
    if let Ok(n) = value.parse::<i64>() {
        return n != 0;
    }
    !value.is_empty()
}

fn cmp_num(left: &str, right: &str, op: &str) -> Result<bool, String> {
    let Ok(l) = left.parse::<i64>() else {
        return Err("while: Expression Syntax.".into());
    };
    let Ok(r) = right.parse::<i64>() else {
        return Err("while: Expression Syntax.".into());
    };
    Ok(match op {
        "<" => l < r,
        ">" => l > r,
        "<=" => l <= r,
        ">=" => l >= r,
        _ => false,
    })
}
