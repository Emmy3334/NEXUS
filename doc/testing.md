# Testing

## Layout

All tests live under **`tests/integration/`**, reached from a single crate root:

```text
tests/integration/main.rs
├── advanced/
├── builtins/          # alias, bindkey, cd, dirstack, env, exit, history*, jobs, …
├── exec/              # external, list, pipe, redirect, subshell, jobs/
├── expand.rs
├── foreach.rs
├── glob.rs
├── if_block.rs
├── jobs_signals.rs
├── lex.rs
├── line_edit.rs
├── line_edit_paste.rs
├── parse.rs
├── repl.rs
├── scripting.rs
├── shell_env.rs
├── specials.rs
└── while_loop.rs
```

`Cargo.toml` sets `autotests = false` and declares:

```toml
[[test]]
name = "integration"
path = "tests/integration/main.rs"
```

**Do not** add `#[test]` inside `src/`. That keeps rust-analyzer on one module tree and matches project structure rules.

## Gates (before finishing a slice)

From repo root:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test -- --test-threads=1
```

`--test-threads=1` avoids flaky cwd races between tests that call `cd` / dir-stack builtins.

CI (`.github/workflows/ci.yml`) also runs a **release** build and **`cargo audit`**. Locally:

```bash
cargo build --release --locked --bin nexus
cargo audit   # requires: cargo install cargo-audit
```

Container build notes and hardened `docker run` flags: [security.md](security.md).

## What tests cover (by area)

| Area | Typical modules |
|------|-----------------|
| Lexer / parser | `lex`, `parse` |
| Expand / glob | `expand`, `glob` |
| Exec | `exec/*` |
| Builtins | `builtins/*` |
| Jobs / signals | `exec/jobs`, `jobs_signals`, `builtins/jobs` |
| REPL / line edit | `repl`, `line_edit`, `line_edit_paste` |
| Control | `foreach`, `while_loop`, `if_block` |
| Scripting / env / specials | `scripting`, `shell_env`, `specials` |
| Host hardening | `path_jail`, `trusted_bin`, `rlimit` |

Platform-sensitive tests should stay **Unix-first** and document assumptions when they skip or gate on `cfg(unix)`.
