//! `$name` / `${name}` Tab completion against shell variables.

enum Form {
    Dollar { prefix: String },
    Braced { prefix: String },
}

/// Collect `$NAME` / `${NAME}` matches for a `$…` token into `out`.
pub(super) fn collect(prefix: &str, names: &[String], out: &mut Vec<String>) {
    let Some(form) = parse_form(prefix) else {
        return;
    };
    let needle = match &form {
        Form::Dollar { prefix } | Form::Braced { prefix } => prefix.as_str(),
    };
    for name in names {
        if name.starts_with(needle) {
            out.push(format_match(&form, name));
        }
    }
}

/// Whether `prefix` is a `$` / `${` variable token (not a path/builtin token).
#[must_use]
pub(super) fn is_var_token(prefix: &str) -> bool {
    parse_form(prefix).is_some()
}

fn parse_form(prefix: &str) -> Option<Form> {
    let rest = prefix.strip_prefix('$')?;
    if let Some(inner) = rest.strip_prefix('{') {
        return name_prefix_ok(inner).then(|| Form::Braced {
            prefix: inner.to_owned(),
        });
    }
    name_prefix_ok(rest).then(|| Form::Dollar {
        prefix: rest.to_owned(),
    })
}

fn format_match(form: &Form, name: &str) -> String {
    match form {
        Form::Dollar { .. } => format!("${name}"),
        Form::Braced { .. } => format!("${{{name}}}"),
    }
}

fn name_prefix_ok(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        None => true,
        Some(c) if is_name_start(c) => chars.all(is_name_continue),
        Some(_) => false,
    }
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
