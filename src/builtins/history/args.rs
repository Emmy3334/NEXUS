//! Parse `history` argv into a typed action.

use std::path::PathBuf;

/// Flags for the print form: `history [-hTr] [n]`.
#[derive(Debug, Clone, Default)]
pub(super) struct PrintOpts {
    pub(super) hide_numbers: bool,
    pub(super) timestamps: bool,
    pub(super) reverse: bool,
    pub(super) last_n: Option<usize>,
}

/// What `history` should do.
#[derive(Debug, Clone)]
pub(super) enum HistoryAction {
    Print(PrintOpts),
    Clear,
    Save(Option<PathBuf>),
    Load(Option<PathBuf>),
    Merge(Option<PathBuf>),
}

/// Parse argv after `history`.
pub(super) fn parse_argv(argv: &[String]) -> Result<HistoryAction, String> {
    let args = &argv[1..];
    if args.is_empty() {
        return Ok(HistoryAction::Print(PrintOpts::default()));
    }
    if let Some(action) = try_exclusive(args)? {
        return Ok(action);
    }
    parse_print(args)
}

fn try_exclusive(args: &[String]) -> Result<Option<HistoryAction>, String> {
    let first = args[0].as_str();
    match first {
        "-c" => {
            if args.len() > 1 {
                return Err("history: Too many arguments.".into());
            }
            Ok(Some(HistoryAction::Clear))
        }
        "-S" | "-L" | "-M" => {
            if args.len() > 2 {
                return Err("history: Too many arguments.".into());
            }
            let path = args.get(1).map(PathBuf::from);
            Ok(Some(match first {
                "-S" => HistoryAction::Save(path),
                "-L" => HistoryAction::Load(path),
                _ => HistoryAction::Merge(path),
            }))
        }
        _ => Ok(None),
    }
}

fn parse_print(args: &[String]) -> Result<HistoryAction, String> {
    let mut opts = PrintOpts::default();
    let mut saw_n = false;
    for arg in args {
        if saw_n {
            return Err("history: Too many arguments.".into());
        }
        if let Some(rest) = arg.strip_prefix('-') {
            if rest.is_empty() || rest.bytes().all(|b| b.is_ascii_digit()) {
                opts.last_n = Some(parse_count(arg)?);
                saw_n = true;
                continue;
            }
            apply_print_flags(rest, &mut opts)?;
        } else {
            opts.last_n = Some(parse_count(arg)?);
            saw_n = true;
        }
    }
    Ok(HistoryAction::Print(opts))
}

fn apply_print_flags(flags: &str, opts: &mut PrintOpts) -> Result<(), String> {
    for ch in flags.chars() {
        match ch {
            'h' => opts.hide_numbers = true,
            'T' => opts.timestamps = true,
            'r' => opts.reverse = true,
            _ => return Err(format!("history: Unknown option: -{ch}.")),
        }
    }
    Ok(())
}

fn parse_count(arg: &str) -> Result<usize, String> {
    arg.parse()
        .map_err(|_| "history: Badly formed number.".to_string())
}
