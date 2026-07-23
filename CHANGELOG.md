# Changelog

All notable changes to NEXUS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
for release tags (crate version remains `0.1.0` until the first tagged release).

## [Unreleased]

### Added

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

### Fixed

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
