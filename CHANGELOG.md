# Changelog

All notable changes to NEXUS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
for release tags (crate version remains `0.1.0` until the first tagged release).

## [Unreleased]

### Added

- Beat-zsh PR6: shell arrays (`typeset -a`, `${name[i]}`, `${name[@]}`, `${#name}` element count),
  bash-style `${name:offset}` / `${name:offset:length}` slices (negative offset: `${name: -N}`),
  and zsh-style glob qualifiers `*(.)` / `*(/)` / `*(*)` on active patterns.
- Layered startup files under `$NEXUS_DOTDIR` or `$HOME`: `.nexusenv` (all invocations),
  `.nexusrc` (interactive TTY; `NEXUSRC` override unchanged), `.nexuslogin` / `.nexuslogout`
  (login shells). `NEXUS_NORCS=1` skips all; `nexus -l` and argv0 `-name` mark login shells.
  Sample templates in `StartupFiles/`.
- `CwdGuard` RAII restore for command substitution cwd (unwind-safe).

### Changed

- Bumped Wasmtime / `wasmtime-wasi` from 24.x to **36.0.7+** (RUSTSEC-2026-0086…0096:
  sandbox-escape and related issues on 24.x). Sandbox runner imports
  `wasmtime_wasi::p2::pipe::MemoryOutputPipe`.
- Heal integration tests that exercise alpine catch-all (`apk` / `cat` / `printenv`) set
  `heal_catch_all=1` so they match the default mapped-only Docker/Kube policy.

### Fixed

- `${name:-word}` stays default substitution even when `word` is numeric; negative
  substring offsets use bash spacing `${name: -N}` (disambiguates from `:-`).
- Word expansion no longer clones the full shell env (including history) on every
  argv word; `` `…` `` / `$(…)` isolation uses `clone_for_capture` (empty history).
  Capture clones are skipped when quotes / `$((…))` make subst impossible.
- Lines without `!` / leading `^` skip history expansion (hot-path fast path).
- `` `…` `` / `$(…)` restore process cwd after `cd` (same as `(…)` subshells).
- Function bodies replay through the same control path as loop/`if`/`case` bodies, so
  nested `foreach` / `while` / `if` / `case` (and `return` from inside them) work.

### Added

- Beat-zsh PR5 command hash: `pathfind` caches resolved PATH lookups (invalidates on
  PATH change); `hash` / `hash -r` builtins; directory listing cache for Tab PATH
  complete; `scripts/bench_hotpath.sh` for spawn vs builtin timing.
- Beat-zsh PR3 cloud completion depth: nested `aws s3` / `gcloud compute` action Tab complete,
  static match descriptions in the arrow menu (`value  — description`), live container/pod
  `Tag::Resources` ranking, and case-insensitive `@docker` / `@kube` / `heal` collectors.
- Beat-zsh PR2 completion registry: `compdef` / `compinit` / `compdump` builtins,
  `CompRegistry` on `ShellEnvironment`, dump load at TTY boot, and first-verb Tab
  complete for user-registered commands (curated providers keep priority).
- Beat-zsh PR1 completion engine: ranked `Match` model with `Tag` grouping, case-insensitive
  prefix/substring matchers, Levenshtein-≤1 approximate fallback, and tagged arrow-menu headers.
- Core builtins `echo` (`-n`), `true`, `false`, and `:` (in-process; no `/bin` spawn).
- `histsize` local caps retained history (default **10000**; `0` clears).
- `typeset` builtin: function-local like `local`; `-x` / `--export` also exports
  (restored with the local frame inside functions; works like `setenv` outside).
- Host `aws` / `gcloud` curated service/group **flag** Tab complete (e.g. `aws s3 --rec` →
  `--recursive`; `gcloud compute --proj` → `--project`).
- Heal UX: `heal status` alias; Tab `status`/`help`; tips for quiet and catch_all.
- `@docker ps -q` / `--quiet` prints short container IDs only (no table header).
- Host `helm` / `systemctl` curated verb **flag** Tab complete (e.g. `helm install --names` →
  `--namespace`; `systemctl status --us` → `--user`).
- `@docker ps -a` / `--all` lists exited containers; `@docker logs -f` / `--follow`
  streams until the container stops (bollard `follow`).
- `@docker` / `@kube` curated verb **flag** Tab complete (e.g. `@docker logs --fol` →
  `--follow`; `@kube pods -` → `-n` / `-A`); host `docker`/`kubectl` flags unchanged.
- Opt-in **trusted-bin** allowlist: `trusted_bin` / `NEXUS_TRUSTED_BIN` =
  colon-separated absolute files/dirs; unmatched resolved externals → stderr +
  status 126 (no heal). Default off.
- Ambiguous Tab **arrow menu**: Up/Down highlight a match, Enter inserts it, Esc cancels;
  Tab still cycles. Soft-capped list (100).
- Host `docker` / `kubectl` Tab flags after curated verbs (e.g. `docker logs --fol` →
  `--follow`; `kubectl get --names` → `--namespace`); `kubectl get`/`describe` kinds unchanged.
- Arithmetic bitwise assignments `&=` `|=` `^=` `<<=` `>>=` and comma `,` in
  `$((…))` / `((…))` (value is the rightmost expression).
- Host spawn hardening: absolute-only **PATH jail** (default on;
  `path_jail=0` / `NEXUS_PATH_JAIL=0` to disable) and opt-in child **setrlimit**
  (`rlimit=1` / `NEXUS_RLIMIT=1`; CPU/NOFILE everywhere; AS/NPROC on Linux —
  macOS cannot apply finite `RLIMIT_AS`).
- Arithmetic command `((expr))`: same engine as `$((…))` (including `++`/`--` and
  assignments); status 0 if result ≠ 0, else 1; does not execute the number as a command.
- Arithmetic prefix/postfix `++` / `--` on bare names in `$((…))` (same writeback as
  assignments: local, else exported, else new local).
- Richer Tab complete: curated `git <verb>` flags (token starting with `-`) and wider
  branch verbs (`merge` / `rebase` / …); host `kubectl get`/`describe` resource kinds.
- Tab completion registry for first-verb subcommands of many common CLIs (VCS, language
  toolchains, containers, Kubernetes, cloud/IaC, package managers, systemctl/tmux, build
  tools, databases/backup). Static lists under `complete/subcmds/`; unregistered commands
  still use PATH/file complete. `git checko` → `checkout`; after `checkout`/`switch`/
  `branch`, branch names still complete.
- Ambiguous Tab completion: after the column listing, further Tabs cycle through matches
  (wrap) until the token is edited (`complete_or_cycle`).
- Arithmetic assignments in `$((…))`: `=` / `+=` `-=` `*=` `/=` `%=` (right-assoc;
  writes local, or exported if already exported with no local).
- Command substitution `$(cmd)` (lex keeps the span as one word; unquoted field-split
  like backticks; double-quoted keeps blanks; reuses capture / trailing-newline strip).
- Line-edit `C-l` clear-screen (`clear-screen`) and Tab completion for `$name` /
  `${name}` against locals ∪ exported vars.
- Tab completion uses the shared `builtins::NAMES` list (cloud + dirstack +
  `bindkey` / `which` / …), so prefix complete stays in sync with `is_builtin`.
- Line-edit BOL/EOL and emacs char motion: `C-a` / `C-e`, `C-b` / `C-f`, Home/End
  (`beginning-of-line` / `end-of-line`); vi command map `0` / `^` / `$`.
- Extended arithmetic expansion: bare names, nested `$((…))`, `**`, shifts, bitwise /
  compare / `&&` `||` / `!` `~`, and ternary `?:`.
- Arithmetic expansion `$((expr))`: integers, `+ - * / %`, unary `+/-`, `(…)`, and
  `$name` / `$?` / `$n` inside the expression (unset → 0). Unquoted form stays one
  lex word; div-by-zero / bad expr → expand error.
- Heal env pass policy: Docker/Kube forward **no** exports by default
  (`heal_env` / `NEXUS_HEAL_ENV` = `none`); opt in with `*`/`all` or a comma
  allowlist. `PATH` / loader-path keys stay scrubbed. Shown on `heal` / `doctor`.
- Security hardening: release profile (LTO / strip / `panic = "abort"`), multi-stage
  distroless `cc-debian12:nonroot` `Dockerfile`, GitHub Actions CI (`fmt` / `clippy` /
  test / release build / `cargo audit`), and [doc/security.md](doc/security.md).
- Expanded `@kube`: `-n` / `-A` scoping, `get pods|nodes`, `describe pod`, richer Tab complete
  (subcommands, namespaces, pods).
- Heal UX: default chain **Wasm → Kube → Docker**; overrides via `heal_order` /
  `NEXUS_HEAL_ORDER`, `heal_image` / `NEXUS_HEAL_IMAGE`, `heal_quiet` /
  `NEXUS_HEAL_QUIET`; success banners `nexus: healed via …`.
- Optional tracing (`tracing` + `tracing-subscriber`) when `NEXUS_LOG` or `RUST_LOG` is set;
  heal and `@kube` emit debug/info events.
- Heal-aware not-found hints: `nexus: did you mean: …` from PATH/cwd/history, plus a heal
  tip when backends declined; suppressed by `heal_quiet` / `NEXUS_HEAL_QUIET` (status stays 127).
- Line-edit word ops + kill-ring (emacs `M-b`/`M-f`/`C-w`/`M-d`/`C-k`/`C-u`/`C-y`/`M-t`; vi `b`/`w`).
- Reverse incremental history search (`C-r` / `history-incremental-search-backward`).
- Parameter operators: `${var:-word}`, `${var:+word}`, `${#var}`, `${var#pat}` / `##` / `%` / `%%`.
- Brace expand `{a,b}` and recursive `**` globstar pathname matching.
- Shell functions (`name() { … }` / `function name { … }`) with positionals and `return`.
- `case` / `esac` with glob patterns, and `disown` to drop a job from the table.
- Ambiguous Tab completion lists matches in `$COLUMNS`-aware columns (soft-capped), after
  common-prefix insert.
- Native `@docker` builtin (`ps`, `logs`) via bollard, with Tab for subcommands and
  running container names.
- Heal fidelity: Docker/Kube forward exported shell env; Docker accepts finite stdin
  bytes (heredoc/pipe); Kube declines when stdin is present so Docker can feed it.
- `sandbox install` / `list` / `rm` for the local Wasm module cache (`.wasm` / `.wat`).
- `@kube logs -f` / `--follow` (stream until EOF or Ctrl-C) and thin non-TTY `@kube exec`
  (`[-n NS] <pod> -- <cmd>…`).
- `heal` / `doctor` status builtin: configured order/image/quiet, session attach count, and
  live wasm/kube/docker reachability probes.
- Heal command→image map (python/node/ruby/php family) for Docker/Kube by default; unmapped
  typos skip containers (fast). Opt-in alpine catch-all: `heal_catch_all` / `NEXUS_HEAL_CATCH_ALL`.
- Function-scoped `local name[=value]` (restores prior shell local on function exit / `return`).

### Fixed

- Drop global Linux `link-arg=-pie` rustflags (broke proc-macro dylib link in CI
  with `undefined reference to main`); rely on rustc’s default PIE for Linux bins.

- Background job tests wait longer for nested-`nexus` pipelines (Wasmtime-sized
  debug binary cold-start) and prefer `CARGO_BIN_EXE_nexus` when spawning the
  isolated shell child.

- Heal no longer forwards host `PATH` / loader-path vars into Docker/Kube containers, so
  mapped images keep their default binary search path.

- Shared process-wide cwd lock across integration tests that mutate `current_dir`
  (glob, git prompt, complete, dirstack, heal bind-mount) to reduce parallel flakes.
- Heal soft-decline on kube wait timeouts and invalid Docker bind mounts so the
  chain can fall through to classic `127` instead of hard-failing with status `1`.
- Shorter kube heal wait deadline (30s) for snappier fallthrough.

## [0.1.0] — Cloud MVP (develop through PR #45)

Cloud / self-healing shell slice on `develop` (merge commits into `develop`):

| PR | Summary |
|----|---------|
| [#37](https://github.com/Emmy3334/NEXUS/pull/37) | Self-heal resolver seam (`CommandResolver` / classic `127`) |
| [#38](https://github.com/Emmy3334/NEXUS/pull/38) | Docker heal backend (bollard / alpine) |
| [#39](https://github.com/Emmy3334/NEXUS/pull/39) | Context-aware Tab completion |
| [#40](https://github.com/Emmy3334/NEXUS/pull/40) | Git-aware primary prompt |
| [#41](https://github.com/Emmy3334/NEXUS/pull/41) | `~/.nexusrc` interactive startup |
| [#42](https://github.com/Emmy3334/NEXUS/pull/42) | Wasmtime sandbox + Wasm cache heal |
| [#43](https://github.com/Emmy3334/NEXUS/pull/43) | History auto-persist (`~/.nexus_history`) |
| [#44](https://github.com/Emmy3334/NEXUS/pull/44) | Native `@kube` nodes/pods/logs (kube-rs) |
| [#45](https://github.com/Emmy3334/NEXUS/pull/45) | Kubernetes Pod heal backend |

Earlier Minishell / 42sh work landed in prior PRs on `develop` / `main` (see git history).
