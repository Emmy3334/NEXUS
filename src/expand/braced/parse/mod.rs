//! Parse `${…}` bodies into plain names or parameter operators.

mod index;
mod name;
mod slice;

use index::index_form;
use name::param_name_end;
use slice::slice_form;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Form<'a> {
    Plain(&'a str),
    Length(&'a str),
    Default {
        name: &'a str,
        word: &'a str,
    },
    Alternate {
        name: &'a str,
        word: &'a str,
    },
    StripPrefix {
        name: &'a str,
        pat: &'a str,
        longest: bool,
    },
    StripSuffix {
        name: &'a str,
        pat: &'a str,
        longest: bool,
    },
    Index {
        name: &'a str,
        index: &'a str,
    },
    Slice {
        name: &'a str,
        offset: &'a str,
        length: Option<&'a str>,
    },
    Keys(&'a str),
    Values(&'a str),
}

pub(super) fn form(body: &str) -> Form<'_> {
    if let Some(form) = flag_form(body) {
        return form;
    }
    if let Some(name) = length_name(body) {
        return Form::Length(name);
    }
    let end = param_name_end(body);
    if end == 0 {
        return Form::Plain(body);
    }
    let name = &body[..end];
    let rest = &body[end..];
    if rest.is_empty() {
        return Form::Plain(name);
    }
    if let Some(form) = index_form(name, rest) {
        return form;
    }
    if let Some(form) = slice_form(name, rest) {
        return form;
    }
    operator(name, rest).unwrap_or(Form::Plain(body))
}

/// `${(k)name}` / `${(v)name}` associative-array key/value flags.
fn flag_form(body: &str) -> Option<Form<'_>> {
    let rest = body.strip_prefix('(')?;
    let close = rest.find(')')?;
    let flag = &rest[..close];
    let name = &rest[close + 1..];
    if name.is_empty() || param_name_end(name) != name.len() {
        return None;
    }
    match flag {
        "k" => Some(Form::Keys(name)),
        "v" => Some(Form::Values(name)),
        _ => None,
    }
}

fn length_name(body: &str) -> Option<&str> {
    let rest = body.strip_prefix('#')?;
    if rest.is_empty() {
        return None;
    }
    if param_name_end(rest) == rest.len() {
        return Some(rest);
    }
    None
}

fn operator<'a>(name: &'a str, rest: &'a str) -> Option<Form<'a>> {
    if let Some(word) = rest.strip_prefix(":-") {
        return Some(Form::Default { name, word });
    }
    if let Some(word) = rest.strip_prefix(":+") {
        return Some(Form::Alternate { name, word });
    }
    if let Some(pat) = rest.strip_prefix("%%") {
        return Some(Form::StripSuffix {
            name,
            pat,
            longest: true,
        });
    }
    if let Some(pat) = rest.strip_prefix('%') {
        return Some(Form::StripSuffix {
            name,
            pat,
            longest: false,
        });
    }
    if let Some(pat) = rest.strip_prefix("##") {
        return Some(Form::StripPrefix {
            name,
            pat,
            longest: true,
        });
    }
    if let Some(pat) = rest.strip_prefix('#') {
        return Some(Form::StripPrefix {
            name,
            pat,
            longest: false,
        });
    }
    None
}
