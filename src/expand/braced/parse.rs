//! Parse `${…}` bodies into plain names or parameter operators.

use super::super::name::{is_name_continue, is_name_start};

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
}

pub(super) fn form(body: &str) -> Form<'_> {
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
    operator(name, rest).unwrap_or(Form::Plain(body))
}

fn length_name(body: &str) -> Option<&str> {
    let rest = body.strip_prefix('#')?;
    // `${#}` is argc (plain `#`), not “length of `#`”.
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

fn param_name_end(body: &str) -> usize {
    let mut chars = body.char_indices();
    let Some((_, first)) = chars.next() else {
        return 0;
    };
    if first == '*' || first == '#' {
        return first.len_utf8();
    }
    if first.is_ascii_digit() {
        return scan_while(body, first.len_utf8(), |c| c.is_ascii_digit());
    }
    if is_name_start(first) {
        return scan_while(body, first.len_utf8(), is_name_continue);
    }
    0
}

fn scan_while(body: &str, start: usize, ok: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    for (i, c) in body[start..].char_indices() {
        if !ok(c) {
            break;
        }
        end = start + i + c.len_utf8();
    }
    end
}
