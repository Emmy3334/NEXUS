# Jobs, REPL, scripting, and specials

## REPL loop

Entry points:

- `repl::run_with_env` — interactive / stdin-driven session
- `repl::run_script` — file + argv (`$0`, `$1`, …)

Typical step (`src/repl/`):

1. Optional `precmd`
2. Read a complete logical line (editor or plain; may continue for incomplete quotes / control / heredoc)
3. History `!` expansion
4. Detect `foreach` / `while` / `if` → collect body → run specialized runner
5. Otherwise lex → parse → heredoc collect → `execute_list`
6. Handle `CommandResult::Exit` / `Source` / status
7. On EOF: respect `ignoreeof`; else return last status

Buffers (`line`, `expanded`, `tokens`, `argv`) are reused across steps to avoid alloc churn on the hot path.

## Line edition

| Platform | Behavior |
|----------|----------|
| Unix TTY | Raw terminal editor under `src/repl/line_edit/tty/` — navigation, history recall, completion, paste queue, bindkey actions |
| Else | Plain line read (`plain.rs`) |

Public pieces re-exported from `repl`: `Action`, `HistoryRecall`, `KeyBindings`, `ReplInput`, `complete`, and on Unix `take_complete_line`.

### Context-aware Tab completion

`src/repl/line_edit/complete/` inspects words before the token:

| Context | Suggestions |
|---------|-------------|
| `git … checkout\|switch\|branch …` (flags allowed) | Local branches under `.git/refs/heads/` |
| `python` / `python3` | Cwd files ending in `.py` (dirs still listed) |
| `ruby` | Cwd files ending in `.rb` |
| `@docker … logs …` (flags allowed) | Running container names via bollard |

Otherwise falls back to builtins + `PATH` + filesystem matches.

**Bracketed paste**: TTY path queues paste bytes (`input_queue` on `ReplIo`) so multi-line pastes do not scramble the editor.

**Completion**: path-oriented helpers under `line_edit/complete/`.

### `bindkey`

Builtin + `src/keybind/`:

- Styles: emacs / vi (primary and alternate maps)
- Binding kinds: actions, commands, literals
- Flags as implemented in `builtins/bindkey/` (`-e`, `-v`, `-c`, `-s`, `-a`, …)

## Job control

API surface: `src/jobs/mod.rs` + `JobTable` in `jobs/table/`.

| Feature | Unix | Non-Unix |
|---------|------|----------|
| `&` background jobs | Real pgrp / session helpers | Stub / limited |
| `jobs` / `fg` / `bg` | Wait / continue via tty ownership | Stub |
| SIGTSTP → status | `128 + SIGTSTP` | N/A |

Unix pieces live under `src/jobs/unix/` (child setup, `wait_fg`, tty/`tcsetpgrp`). Stubs under `src/jobs/stub/`.

`job_control_active()` reports whether interactive job control is enabled for this process.

Background registration is wired from `src/exec/background/`. Cloning `ShellEnvironment` **drops** the job table so subshells do not inherit live children.

## Scripting

- `nexus script.sh args…` → `run_script` with argv `[script, …args]`
- Missing file → stderr `{path}: No such file.` and status `1`
- `source` / `.` queue another file in the **same** environment (REPL handles `CommandResult::Source`)

Positional expansion uses `ShellEnvironment` argv (`$0`…, `$#`, `$*`).

## Specials

Module: `src/specials/`.

| Mechanism | Behavior |
|-----------|----------|
| `precmd` | Alias run before each interactive prompt |
| `cwdcmd` | Alias run after successful directory change |
| `ignoreeof` | Local that blocks exit on EOF until allowed |
| Seeded locals | `cwd`, `home`, `user`, `term` at `ShellEnvironment::capture` / `seed_specials` |

Hooks are alias-based: define `alias precmd '…'` / `alias cwdcmd '…'` as in tcsh.
