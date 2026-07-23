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

## Session history

Interactive TTY sessions:

1. Load `~/.nexusrc` (RC / `source` lines are **not** recorded in history)
2. Soft-load `~/.nexus_history` (or the `histfile` local if set in RC / env)
3. Run the REPL (typed lines are recorded)
4. Soft-save the histfile on exit / EOF

Missing histfile on load is quiet. Manual `history -S` / `-L` / `-M` still work.

## Line edition

| Platform | Behavior |
|----------|----------|
| Unix TTY | Raw terminal editor under `src/repl/line_edit/tty/` — navigation, **word ops / kill-ring / yank**, history recall, **Ctrl-R reverse-i-search**, completion, paste queue, bindkey actions |
| Else | Plain line read (`plain.rs`) |

Emacs defaults (also available via `bindkey`):

| Keys | Action |
|------|--------|
| `M-b` / `M-f` | Backward / forward word |
| `C-w` / `M-d` / `M-BS` | Kill word backward / forward |
| `C-k` / `C-u` | Kill to end of line / kill whole line |
| `C-y` | Yank from kill-ring |
| `M-t` | Transpose adjacent words |
| `C-r` | Reverse incremental history search |

While reverse-i-search is active, the prompt shows `(reverse-i-search)\`query': ` (or `(failed r-search)…`). Type to refine the query; `C-r` again selects an older match; Backspace edits the query; Enter accepts; Esc / `C-c` aborts and restores the draft line. Other bound actions leave isearch with the current match.

Vi command map also binds `b` / `w` for word motion. Words are whitespace-separated.

Public pieces re-exported from `repl`: `Action`, `HistoryISearch`, `HistoryRecall`, `KeyBindings`, `ReplInput`, `complete`, `format_columns`, `list_display_lines`, `list_display_lines_width`, and on Unix `take_complete_line`.

### Context-aware Tab completion

`src/repl/line_edit/complete/` inspects words before the token:

| Context | Suggestions |
|---------|-------------|
| `git … checkout\|switch\|branch …` (flags allowed) | Local branches under `.git/refs/heads/` |
| `python` / `python3` | Cwd files ending in `.py` (dirs still listed) |
| `ruby` | Cwd files ending in `.rb` |
| `@docker ` (no subcommand yet) | `ps` / `logs` / `help` |
| `@docker … logs …` | Running container names via bollard |
| `@kube …` after `@kube ` | Subcommands (`nodes`, `pods`, `logs`, `exec`, `get`, `describe`, …) |
| `@kube … -n ` | Namespace names via kube-rs |
| `@kube … logs …` / `exec …` / `describe pod …` | Pod names via kube-rs (soft-fail if no cluster) |

Otherwise falls back to builtins + `PATH` + filesystem matches.

Ambiguous matches: insert the shared prefix when it grows, then list remaining choices in
`$COLUMNS`-aware columns (soft-capped at 100, with an “… and N more” trailer). Unique
matches replace the token with no listing. No compsys / `zstyle` menu-select yet.

### Git-aware primary prompt

Interactive primary prompts come from `repl::format_primary()` (`src/repl/prompt/`):

- Outside a git work tree: `$> `
- On a branch: `$> [main] `
- With uncommitted changes (`git status --porcelain -uno`): `$> [main*] `

Branch is read from `.git/HEAD` (no subprocess). Dirty state uses `git status --porcelain -uno` when available, with a short TTL / metadata cache so successive prompts avoid redundant spawns.

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
| `jobs` / `fg` / `bg` / `disown` | Wait / continue / drop-from-table via tty ownership | Stub |
| SIGTSTP → status | `128 + SIGTSTP` | N/A |

Unix pieces live under `src/jobs/unix/` (child setup, `wait_fg`, tty/`tcsetpgrp`). Stubs under `src/jobs/stub/`.

`job_control_active()` reports whether interactive job control is enabled for this process.

Background registration is wired from `src/exec/background/`. Cloning `ShellEnvironment` **drops** the job table so subshells do not inherit live children.

## Startup RC (`~/.nexusrc`)

On an **interactive TTY** session (`repl::run_with_env`), before the main loop Nexus loads a startup file like `source`:

| Resolution | Behavior |
|------------|----------|
| `NEXUSRC` set to a path | Use that file |
| `NEXUSRC` empty | Skip RC entirely |
| unset | `$home` / `$HOME` + `/.nexusrc` |

Missing file = quiet no-op. `exit` in the RC ends the shell before the REPL. Otherwise the RC’s last status seeds the REPL `$?` (`RcLoad::Continue`). Piped / Cursor-based “interactive” tests do **not** load RC (`stdin.is_terminal()` is false).

Helpers: `repl::load_startup_rc`, `repl::source_rc`, `repl::RcLoad`.

## Observability

Interactive sessions stay quiet unless `NEXUS_LOG` or `RUST_LOG` is set; then stderr gets a `tracing-subscriber` env-filter log. Useful filters: `nexus=info`, `nexus::heal=debug`.

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
