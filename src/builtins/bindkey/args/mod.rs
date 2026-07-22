//! Parse `bindkey` argv into a structured invocation.

mod finish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum BindKind {
    #[default]
    Editor,
    Command,
    Literal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Invocation {
    Usage,
    ListCommands,
    ResetDefault,
    ResetEmacs,
    ResetVi,
    List {
        alternate: bool,
    },
    Show {
        keys: Vec<u8>,
        alternate: bool,
    },
    Remove {
        keys: Vec<u8>,
        alternate: bool,
    },
    Bind {
        keys: Vec<u8>,
        value: String,
        alternate: bool,
        kind: BindKind,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Flags {
    pub(super) alternate: bool,
    pub(super) bind_b: bool,
    pub(super) arrow: bool,
    pub(super) remove: bool,
    pub(super) kind: BindKind,
}

pub(super) fn parse_invocation(argv: &[String]) -> Result<Invocation, String> {
    if argv.len() == 1 {
        return Ok(Invocation::List { alternate: false });
    }
    if let Some(solo) = solo_mode(&argv[1]) {
        return if argv.len() == 2 {
            Ok(solo)
        } else {
            Err("Too many arguments.".into())
        };
    }
    let (flags, rest) = collect_flags(&argv[1..])?;
    finish::finish(flags, rest)
}

fn solo_mode(arg: &str) -> Option<Invocation> {
    Some(match arg {
        "-u" => Invocation::Usage,
        "-l" => Invocation::ListCommands,
        "-d" => Invocation::ResetDefault,
        "-e" => Invocation::ResetEmacs,
        "-v" => Invocation::ResetVi,
        _ => return None,
    })
}

fn collect_flags(args: &[String]) -> Result<(Flags, &[String]), String> {
    let mut flags = Flags {
        kind: BindKind::Editor,
        ..Flags::default()
    };
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            return Ok((flags, &args[i + 1..]));
        }
        if !arg.starts_with('-') || arg == "-" {
            return Ok((flags, &args[i..]));
        }
        apply_flag(&mut flags, arg)?;
        i += 1;
    }
    Ok((flags, &args[i..]))
}

fn apply_flag(flags: &mut Flags, arg: &str) -> Result<(), String> {
    match arg {
        "-a" => flags.alternate = true,
        "-b" => flags.bind_b = true,
        "-k" => flags.arrow = true,
        "-r" => flags.remove = true,
        "-c" => flags.kind = BindKind::Command,
        "-s" => flags.kind = BindKind::Literal,
        "-l" | "-d" | "-e" | "-v" | "-u" => {
            return Err(format!("Option {arg} must be used alone."));
        }
        _ => return Err(format!("Unknown option {arg}.")),
    }
    Ok(())
}
