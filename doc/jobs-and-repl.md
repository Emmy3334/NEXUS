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

1. Load layered startup (`.nexusenv` → `.nexusrc` → `.nexuslogin` when login) on interactive TTY
2. Soft-load `~/.nexus_history` (or the `histfile` local if set in RC / env)
3. Run the REPL (typed lines are recorded)
4. Soft-save the histfile on exit / EOF

History length is capped by the `histsize` local (default **10000**); oldest events
are dropped first. `histsize=0` keeps nothing.

Missing histfile on load is quiet. Manual `history -S` / `-L` / `-M` still work.

## Line edition

| Platform | Behavior |
|----------|----------|
| Unix TTY | Raw terminal editor under `src/repl/line_edit/tty/` — navigation (**BOL/EOL**, char / word motion), **kill-ring / yank**, history recall, **Ctrl-R reverse-i-search**, completion, paste queue, bindkey actions |
| Else | Plain line read (`plain.rs`) |

Emacs defaults (also available via `bindkey`):

| Keys | Action |
|------|--------|
| `C-a` / `C-e` | Beginning / end of line |
| `C-b` / `C-f` | Backward / forward char (same as ← / →) |
| `C-l` | Clear screen and redraw the current line |
| `Home` / `End` | Beginning / end of line (`\e[H` / `\e[F`, also `\eOH` / `\eOF`) |
| `M-b` / `M-f` | Backward / forward word |
| `C-w` / `M-d` / `M-BS` | Kill word backward / forward |
| `C-k` / `C-u` | Kill to end of line / kill whole line |
| `C-y` | Yank from kill-ring |
| `M-t` | Transpose adjacent words |
| `C-r` | Reverse incremental history search |

While reverse-i-search is active, the prompt shows `(reverse-i-search)\`query': ` (or `(failed r-search)…`). Type to refine the query; `C-r` again selects an older match; Backspace edits the query; Enter accepts; Esc / `C-c` aborts and restores the draft line. Other bound actions leave isearch with the current match.

Vi command map also binds `0` / `^` / `$` for BOL/EOL and `b` / `w` for word motion. Words are whitespace-separated.

Public pieces re-exported from `repl`: `Action`, `HistoryISearch`, `HistoryRecall`, `KeyBindings`,
`ReplInput`, `complete`, `complete_or_cycle`, `CompleteCycle`, `format_columns`,
`list_display_lines`, `list_display_lines_width`, and on Unix `take_complete_line`.

### Context-aware Tab completion

`src/repl/line_edit/complete/` inspects words before the token:

| Context | Suggestions |
|---------|-------------|
| `helm <verb>` / `systemctl <verb>` / `aws <svc>` / `gcloud <group>` when token starts with `-` | Curated host verb flags |
| `aws s3 ` (non-flag token) | Common S3 actions (`ls`, `cp`, `sync`, `mb`, `rb`, `rm`, `presign`) |
| `gcloud compute ` (non-flag token) | Compute subcommands (`instances`, `ssh`, `disks`, `zones`, …) |
| `git <verb>` (curated) | Common flags when token starts with `-`; branch names for merge/rebase/checkout/… |
| `kubectl get` / `describe` | Common resource kinds (`pods`, `deployments`, …) |
| `git … checkout\|switch\|branch …` (flags allowed) | Local branches under `.git/refs/heads/` |
| `python` / `python3` | Cwd files ending in `.py` (dirs still listed) |
| `ruby` | Cwd files ending in `.rb` |
| `@docker ` (no subcommand yet) | `ps` / `logs` / `help` |
| `@docker ps` / `logs` when token starts with `-` | Curated builtin flags (`--follow`, `--all`, …) |
| `@docker … logs …` (non-flag token) | Running container names via bollard |
| `@kube …` after `@kube ` | Subcommands (`nodes`, `pods`, `logs`, `exec`, `get`, `describe`, …) |
| `@kube <verb>` when token starts with `-` | Builtin flags (`-n` / `--follow` / `-A`, …) |
| `@kube … -n ` | Namespace names via kube-rs |
| `@kube … logs …` / `exec …` / `describe pod …` | Pod names via kube-rs (soft-fail if no cluster) |
| `heal ` / `doctor ` | `help` / `status` |

Otherwise falls back to **all** builtins (`builtins::NAMES`, including `@docker` / `@kube` / `sandbox` / dirstack / …) + `PATH` + filesystem matches.

**PATH command cache (beat-zsh PR5):** `pathfind` remembers resolved externals and caches
per-directory executable names for Tab; both invalidate when `PATH` changes. `hash` lists the
command hash; `hash -r` clears it (like zsh). External spawn reuses the same cache via
`resolve_first`.

**Beat-zsh PR1 engine:** collectors use case-insensitive prefix matching (substring when the
prefix length is ≥2). When nothing matches exactly, the engine re-collects with an empty prefix
and keeps Levenshtein distance ≤1 candidates. Matches carry a `Tag` (`Resources`, `Flags`,
`Commands`, `Files`, …) for ranked sort and `-- Category --` headers in the arrow menu.

**Beat-zsh PR3 cloud depth:** after curated host services/groups, Tab completes nested actions
(`aws s3 ls|cp|sync|…`, `gcloud compute instances|ssh|disks|…`). Static description tables
(`annotate/cloud`, `annotate/nexus`) fill `Match.description`; the arrow menu shows
`value  — description` when present. Live `@docker logs` container names and `@kube` pod /
namespace names get `Tag::Resources` and a score bump so they rank above file paths on ties.
Case-insensitive prefix matching applies to `heal`, `@docker`, and `@kube` static lists.

After a curated host verb, tokens starting with `-` complete common flags (same idea as `git`):
`kubectl get|describe|logs|apply|delete|exec`, `docker ps|logs|run|exec|rm|images|pull|build`,
`helm install|upgrade|…`, `systemctl start|status|…` (leading `--user` / `--system` skipped when
finding the verb), `aws s3|ec2|…`, `gcloud compute|run|…`. `kubectl get` / `describe` still complete resource kinds when the kind token
is not a flag.

Tokens starting with `$` / `${` complete against shell locals ∪ exported names (`$HOME`, `${HOME}`).

Ambiguous matches: insert the shared prefix when it grows, then list remaining choices
(vertical menu with reverse-video highlight). **Up/Down** move the highlight; **Enter**
inserts the highlighted match (does not submit the line); **Esc** / **Ctrl-C** cancel the
menu. Further **Tabs** still cycle-apply matches. Full compsys / `zstyle` menu-select
remains out of scope.

### Primary prompt

Interactive primary prompts come from `repl::format_primary()` / `format_primary_with()` (`src/repl/prompt/`):

**Classic** (default when `NEXUS_PROMPT_STYLE` is unset):

- Outside a git work tree: `$> `
- On a branch: `$> [main] `
- With uncommitted changes (`git status --porcelain -uno`): `$> [main*] `

**Powerlevel10k-inspired** (`NEXUS_PROMPT_STYLE=powerlevel10k`, typically via Oh My Nexus `omn_theme = powerlevel10k`):

- Left segments from `NEXUS_PROMPT_ELEMENTS` (default: `os_icon user dir vcs prompt_char`)
- Nerd Font icons when `NEXUS_PROMPT_ICONS=nerdfont` (use `ascii` otherwise)
- Directory shortened with unique prefixes under `$home` (`~/Doc/G/NEXUS`)
- `❯` colored by previous command status

Branch is read from `.git/HEAD` (no subprocess). Dirty state uses `git status --porcelain -uno` when available, with a short TTL / metadata cache so successive prompts avoid redundant spawns. TTY redraw measures **display columns** (ANSI stripped) so colored / wide-glyph prompts keep the cursor aligned.

**Bracketed paste**: multi-line paste keeps one PS1 on the first row (continuation
rows are bare, like zsh). Up/Down move between those rows only — history recall is
disabled while the buffer contains a newline. Enter runs each physical line via
`input_queue`.

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

## Layered startup files

Files live under **`$NEXUS_DOTDIR`** when set, else **`$HOME`**:

| File | When loaded |
|------|-------------|
| `.nexusenv` | Every invocation (interactive + scripts); keep stdout quiet |
| `.nexusrc` | Interactive TTY only |
| `.nexuslogin` | Interactive TTY + login shell |
| `.nexuslogout` | On exit of login interactive shells (quiet) |

Sample templates: `StartupFiles/` at repo root (copy to your dotdir and edit).

### Gates

| Variable | Effect |
|----------|--------|
| `NEXUS_NORCS=1` | Skip **all** startup files |
| `NEXUSRC` set to a path | Override **`.nexusrc` only** (empty disables RC) |
| `NEXUSRC` unset | `$dotdir/.nexusrc` |

Scripts load **`.nexusenv` only** (not rc/login), unless `NEXUS_NORCS=1`.

Interactive TTY order: **env → rc → login (if login) → histfile → compdump boot**.

### Login detection

- argv0 starts with `-` (e.g. `-nexus`)
- `NEXUS_LOGIN=1`
- `nexus -l` (flag stripped before script / REPL)

Missing file = quiet no-op. `exit` in env/rc/login ends the shell before the REPL. Otherwise the last loaded file’s status seeds `$?` (`RcLoad::Continue`). Piped / Cursor-based “interactive” tests do **not** load startup files (`stdin.is_terminal()` is false).

Helpers: `repl::load_startup_chain`, `repl::load_startup_env`, `repl::load_startup_rc`, `repl::load_logout`, `repl::source_rc`, `repl::RcLoad`.

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
