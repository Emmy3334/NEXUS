# Builtins

Dispatch lives in `src/builtins/mod.rs` (`is_builtin`, `try_run`) and `src/builtins/dispatch/{core,bonus}.rs`. Each builtin is a focused module under `src/builtins/`.

## Result kinds

```rust
pub enum BuiltinResult {
    Status(u8),           // keep REPL; update last status
    Exit(u8),             // terminate shell
    Source(String),       // REPL reads file in current env
    Repeat { count, argv }, // exec loops the argv (avoids recursion cycles)
}
```

`CommandResult` mirrors `Status` / `Exit` / `Source`. `Repeat` is consumed inside exec (`src/exec/repeat.rs`).

## Core builtins

| Builtin | Module | Behavior |
|---------|--------|----------|
| `cd` | `cd.rs` | Change cwd; update `PWD` / `OLDPWD`; trigger `cwdcmd` / specials |
| `setenv` | `setenv.rs` | Set or print exported variables |
| `unsetenv` | `unsetenv.rs` | Remove exported variables |
| `env` | `env.rs` | Print sorted exported environ |
| `exit` | `exit.rs` | `BuiltinResult::Exit` (default = last status) |
| `set` | `set.rs` | List / assign shell **locals** |
| `unset` | `unset.rs` | Remove locals |
| `alias` | `alias.rs` | Define / list aliases |
| `unalias` | `unalias.rs` | Remove aliases |
| `history` | `history/` | Print / save / load / merge / clear history |
| `jobs` | `jobs.rs` | List background jobs; `-l` includes process IDs |
| `fg` | `jobs.rs` | Bring job to foreground and wait |
| `bg` | `jobs.rs` | Continue job in background |
| `source` / `.` | `source.rs` | `BuiltinResult::Source(path)` — run file in current shell |
| `@` | `at.rs` | tcsh-style `@ i++` / `@ i = n` on locals |
| `bindkey` | `bindkey/` | List / set / clear editor bindings (`-e` `-v` `-c` `-s` `-a`, …) |

## Bonus builtins

| Builtin | Module | Behavior |
|---------|--------|----------|
| `which` | `which.rs` | First match: builtin or PATH (`pathfind`) |
| `where` | `which.rs` | All matches |
| `repeat` | `repeat.rs` | `Repeat { count, argv }` — run command N times |
| `pushd` | `dirstack/` | Push directory and `cd`; `-l`/`-n`/`-v`/`-p` print flags; `+n` rotates |
| `popd` | `dirstack/` | Pop and `cd`; same print flags; `+n` drops entry `n` |
| `dirs` | `dirstack/` | Print stack (`-l`/`-n`/`-v`/`-p`); `-c` clear; `-S`/`-L` [file] save/load |

Directory stack state is `ShellEnvironment::dir_stack` (`src/env/dirstack.rs`).

## Recognition list

`is_builtin` matches exactly:

`cd`, `setenv`, `unsetenv`, `env`, `exit`, `set`, `unset`, `alias`, `unalias`, `history`, `jobs`, `fg`, `bg`, `source`, `.`, `@`, `bindkey`, `which`, `where`, `repeat`, `pushd`, `popd`, `dirs`.

Anything else is treated as an **external** (PATH lookup / relative path), subject to spawn errors (`127` when not found).

## External vs builtin in pipelines

Pipeline stages may mix builtins and externals. Builtin stages in a pipe are handled under `src/exec/pipe/builtin.rs`; externals under `pipe/external.rs`. Subshell stages use `pipe/subshell_stage/`.

## Notes for implementers

- Builtins take `&mut ShellEnvironment` and writers for stdout/stderr
- Validate arguments; do not panic on expected bad input
- Prefer `Result` / status codes over unwind
- Keep each builtin file within project size limits (≤35 lines/fn, ≤7 fns/file) — split into submodules when needed (`bindkey/`, `history/`, `dirstack/`)
