# Changelog

All notable changes to NEXUS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
for release tags (crate version remains `0.1.0` until the first tagged release).

## [Unreleased]

### Changed

- Bumped Wasmtime / `wasmtime-wasi` from 24.x to **36.0.7+** (RUSTSEC-2026-0086…0096:
  sandbox-escape and related issues on 24.x). Sandbox runner imports
  `wasmtime_wasi::p2::pipe::MemoryOutputPipe`.
- Heal integration tests that exercise alpine catch-all (`apk` / `cat` / `printenv`) set
  `heal_catch_all=1` so they match the default mapped-only Docker/Kube policy.

### Added

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
