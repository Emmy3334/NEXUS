//! Snapshot of values needed to render one primary prompt.

use super::style::{Element, IconMode, Style};
use crate::env::ShellEnvironment;

/// Owned inputs for [`super::format_primary_with`].
#[derive(Debug, Clone)]
pub struct PromptContext {
    pub last_status: u8,
    pub user: String,
    pub home: String,
    pub cwd: String,
    pub style: Style,
    pub icons: IconMode,
    pub elements: Vec<Element>,
}

impl PromptContext {
    /// Build from shell locals/exports; falls back to process env when unset.
    #[must_use]
    pub fn from_env(env: &ShellEnvironment, last_status: u8) -> Self {
        let style = Style::parse(env.lookup("NEXUS_PROMPT_STYLE"));
        let icons = IconMode::parse(env.lookup("NEXUS_PROMPT_ICONS"));
        let elements = parse_elements(env.lookup("NEXUS_PROMPT_ELEMENTS"));
        Self {
            last_status,
            user: lookup_or(env, "user", "USER", ""),
            home: lookup_or(env, "home", "HOME", ""),
            cwd: cwd_value(env),
            style,
            icons,
            elements,
        }
    }

    /// Classic prompt with process cwd (integration tests / no shell env).
    #[must_use]
    pub fn classic_default() -> Self {
        Self {
            last_status: 0,
            user: String::new(),
            home: String::new(),
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            style: Style::Classic,
            icons: IconMode::Ascii,
            elements: default_elements(),
        }
    }
}

fn lookup_or(env: &ShellEnvironment, local: &str, export: &str, fallback: &str) -> String {
    env.lookup(local)
        .or_else(|| env.lookup(export))
        .unwrap_or(fallback)
        .to_owned()
}

fn cwd_value(env: &ShellEnvironment) -> String {
    if let Some(cwd) = env.lookup("cwd").filter(|s| !s.is_empty()) {
        return cwd.to_owned();
    }
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn default_elements() -> Vec<Element> {
    vec![
        Element::OsIcon,
        Element::User,
        Element::Dir,
        Element::Vcs,
        Element::PromptChar,
    ]
}

fn parse_elements(raw: Option<&str>) -> Vec<Element> {
    let Some(raw) = raw.filter(|s| !s.trim().is_empty()) else {
        return default_elements();
    };
    let parsed: Vec<_> = raw.split_whitespace().filter_map(parse_one).collect();
    if parsed.is_empty() {
        default_elements()
    } else {
        parsed
    }
}

fn parse_one(name: &str) -> Option<Element> {
    Some(match name {
        "os_icon" => Element::OsIcon,
        "user" => Element::User,
        "dir" => Element::Dir,
        "vcs" => Element::Vcs,
        "prompt_char" => Element::PromptChar,
        _ => return None,
    })
}
