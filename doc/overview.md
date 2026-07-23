# Overview

## What this repo is

**NEXUS** implements a Unix-first interactive shell and script runner in Rust. Domain logic lives in the **`nexus` library** (`src/lib.rs`). The **`nexus` binary** (`src/bin/nexus.rs`) is a thin driver:

- No script argument → interactive (or piped) REPL
- Script path as first argument → `repl::run_script`
- I/O / program failure → exit **84**

Product stages (from project PDFs / standing rules):

1. **Minishell** — prompt, lex/parse/exec, builtins, pipes, redirects, env
2. **42sh-class** — jobs, history/`!`, aliases, line edit, control structures, `&&`/`||`, bonuses
3. **Cloud / self-healing** — resolver seam + Wasm cache + Docker backend + native `@kube` (kube-rs) + K8s Pod heal

## Package facts

| Field | Value |
|-------|--------|
| Crate / binary name | `nexus` |
| Edition | 2021 |
| `rust-version` | 1.89 |
| Tests | Single integration crate (`autotests = false`) |
| Main dependency | `nix` 0.29 (`fs`, `process`, `signal`, `term`) |

## Feature map (implemented)

| Area | Status |
|------|--------|
| Lex / parse / exec lists & pipelines | Yes |
| `;` `&` `&&` `\|\|` `\|` | Yes |
| Redirects `<` `>` `>>` `<<` (heredoc) | Yes |
| Subshells `( … )` | Yes |
| `$` / `$?` / backticks / globs / aliases | Yes |
| History store + `!` designators | Yes |
| Jobs `&` / `jobs` / `fg` / `bg` (Unix) | Yes |
| Line edition + `bindkey` (Unix TTY) | Yes |
| Context Tab complete (git branches, `python`/`ruby` filters, `@docker`/`@kube` logs) | Yes |
| Git-aware prompt (`$> [branch*] `) | Yes |
| Startup RC (`~/.nexusrc` / `NEXUSRC`) on interactive TTY | Yes |
| Wasm sandbox (`sandbox`) + module cache heal | Yes |
| Kubernetes API (`@kube` nodes/pods/logs/get/describe via kube-rs) | Yes |
| `foreach` / `while` / `if` | Yes |
| Scripting / `source` / `.` | Yes |
| Specials `precmd` / `cwdcmd` / `ignoreeof` | Yes |
| Bonuses `which`/`where`, `repeat`, `pushd`/`popd`/`dirs`, bracketed paste | Yes |
| Self-heal resolver seam (`CommandResolver` / empty chain → classic 127) | Yes |
| Docker heal backend (`bollard`, alpine image when daemon up) | Yes |
| Cloud / Wasm / K8s heal backends | Wasm cache yes; `@kube` API yes; K8s Pod heal yes |

## Top-level layout

```text
NEXUS/
├── Cargo.toml
├── src/
│   ├── lib.rs              # library surface
│   ├── bin/nexus.rs        # thin main
│   ├── lex/ parse/ expand/ glob/
│   ├── exec/ env/ builtins/ repl/
│   ├── alias.rs history/ jobs/ keybind/
│   ├── foreach/ while_loop/ if_block/
│   ├── pathfind/ specials/
│   └── …
├── tests/integration/      # ALL tests hang off main.rs
└── doc/                    # this documentation
```

Tests must not live under `src/` (`#[cfg(test)]` / `#[test]` are forbidden by project rule).
