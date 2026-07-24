//! Opening screen (zsh-newuser-install style).

use super::key;

use std::io::{self, BufRead, Write};
use std::path::Path;

pub(super) enum Choice {
    Quit,
    Minimal,
    Recommended,
    MainMenu,
}

pub(super) fn run(
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    dotdir: &Path,
    zdmsg: &str,
) -> io::Result<Choice> {
    clear(stdout)?;
    writeln!(
        stdout,
        "This is the NEXUS configuration function for new users,
nexus-newuser-install.
You are seeing this message because you have no NEXUS startup files
(the files .nexusenv, .nexusrc, .nexuslogin in the directory
{zdmsg}).  This function can help you with a few settings that should
make your use of the shell easier.

You can:

(q)  Quit and do nothing.  The function will be run again next time."
    )?;
    if !dotdir.join(".nexusrc").is_file() {
        writeln!(
            stdout,
            "
(0)  Exit, creating the file {zdmsg}/.nexusrc containing just a comment.
     That will prevent this function being run again."
        )?;
    }
    writeln!(
        stdout,
        "
(1)  Continue to the main menu.
"
    )?;
    if recommended_available() {
        writeln!(
            stdout,
            "(2)  Populate your {zdmsg}/.nexusrc with the configuration recommended
     by the system administrator and exit (you will need to edit
     the file by hand, if so desired).
"
        )?;
    }
    match key::read_key(stdin, stdout)? {
        'q' | 'Q' => Ok(Choice::Quit),
        '0' => Ok(Choice::Minimal),
        '1' => Ok(Choice::MainMenu),
        '2' if recommended_available() => Ok(Choice::Recommended),
        _ => {
            writeln!(
                stdout,
                "Aborting.
The function will be run again next time.  To prevent this, execute:
  touch {zdmsg}/.nexusrc"
            )?;
            Ok(Choice::Quit)
        }
    }
}

fn recommended_available() -> bool {
    super::write::recommended_source().is_some()
}

fn clear(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "\x1b[H\x1b[2J")?;
    stdout.flush()
}
