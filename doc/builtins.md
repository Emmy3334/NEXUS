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
| `history` | `history/` | Print / save / load / merge / clear; TTY auto-persists `~/.nexus_history` |
| `jobs` | `jobs.rs` | List background jobs; `-l` includes process IDs |
| `fg` | `jobs.rs` | Bring job to foreground and wait |
| `bg` | `jobs.rs` | Continue job in background |
| `disown` | `jobs.rs` | Remove job from table without killing it |
| `source` / `.` | `source.rs` | `BuiltinResult::Source(path)` — run file in current shell |
| `@` | `at.rs` | tcsh-style `@ i++` / `@ i = n` on locals |
| `bindkey` | `bindkey/` | List / set / clear editor bindings (`-e` `-v` `-c` `-s` `-a`, …) |

## Bonus builtins

| Builtin | Module | Behavior |
|---------|--------|----------|
| `which` | `which.rs` | First match: builtin or PATH (`pathfind`) |
| `where` | `which.rs` | All matches |
| `repeat` | `repeat.rs` | `Repeat { count, argv }` — run command N times |
| `sandbox` | `sandbox.rs` | Run a Wasm module (path / cache name) via Wasmtime; else host command in a temp HOME with filtered env |
| `@kube` | `kube/` | Native K8s: `nodes`, `pods`, `logs`, `get`, `describe` |
| `@docker` | `docker/` | Native Docker Engine: `ps`, `logs` |
| `pushd` | `dirstack/` | Push directory and `cd`; `-l`/`-n`/`-v`/`-p` print flags; `+n` rotates |
| `popd` | `dirstack/` | Pop and `cd`; same print flags; `+n` drops entry `n` |
| `dirs` | `dirstack/` | Print stack (`-l`/`-n`/`-v`/`-p`); `-c` clear; `-S`/`-L` [file] save/load |

Directory stack state is `ShellEnvironment::dir_stack` (`src/env/dirstack/`).
Self-heal backends hang off `ShellEnvironment::healers` (`src/heal/`); empty chain keeps classic `127`. Default order: **Wasm cache**, then **Kubernetes Pod**, then **Docker** (override with `heal_order` / `NEXUS_HEAL_ORDER`). Image: `heal_image` / `NEXUS_HEAL_IMAGE` (default `alpine:3.20`). Quiet success banners **and** not-found tips/suggestions: `heal_quiet` / `NEXUS_HEAL_QUIET=1`. When heal declines, stderr may include `nexus: did you mean: …` (PATH/cwd/history neighbors) and a one-line heal tip; exit status stays `127`.

Docker and Kube heal forward the **exported** shell environment (same map as external children). Finite stdin bytes from heredocs / buffered pipes are fed into Docker heal; Kube declines when stdin is present so Docker can handle it. Locals are not forwarded.

### `sandbox` / Wasm cache

| Piece | Role |
|-------|------|
| `sandbox <name\|path.wasm> [args…]` | Run a WASI Preview1 module via Wasmtime; if not a module, run the host command in a temp `HOME`/`TMPDIR` with only `PATH`/`TERM`/`LANG`/… (no heal backends) |
| `NEXUS_WASM_CACHE` | Override cache directory (default `~/.nexus/wasm`) |
| `~/.nexus/wasm/<name>.wasm` | Module looked up by `sandbox <name>` and by the Wasm heal backend |

Install helpers for tests/tools: `sandbox::install_from_wat` / `install_from_wat_into`.

### `@kube` (Kubernetes)

Talks to the cluster via **kube-rs** (default kubeconfig / namespace):

| Command | Behavior |
|---------|----------|
| `@kube nodes` | List nodes (`NAME STATUS ROLES AGE VERSION`) |
| `@kube pods` | List pods (`NAME READY STATUS RESTARTS AGE`) |
| `@kube pods -n NS` | List pods in namespace `NS` |
| `@kube pods -A` | Same with `NAMESPACE` column (all namespaces) |
| `@kube logs [-n NS] <pod>` | Last 100 log lines |
| `@kube get pods\|nodes …` | Alias to `pods` / `nodes` (flags apply to pods) |
| `@kube describe pod <name> [-n NS]` | Multi-line summary (name, ns, phase, node, restarts, images, conditions) |
| no cluster / bad kubeconfig | Status `1` + stderr (soft for Tab complete → empty) |

Tab: after `@kube ` → subcommands; after `-n` → namespaces; after `logs` / `describe pod` → pod names (when the API is reachable).

Missing external commands may also be healed via an ephemeral alpine Pod when the cluster is reachable (default heal chain: Wasm → Kube → Docker). The Pod bind-mounts the host cwd via `hostPath` when the node can see that path (same idea as Docker heal). Success prints `nexus: healed via kube (pod …)` / `docker (…)` / `wasm (…)` unless quiet.

### `@docker` (Docker Engine)

Uses bollard against the local Docker socket (same stack as Docker heal / Tab).

| Form | Behavior |
|------|----------|
| `@docker ps` | Running containers (Docker CLI–style columns) |
| `@docker logs <name\|id>` | Container logs to stdout |
| daemon down / API error | Status `1` + stderr |

Tab: after `@docker ` → `ps`/`logs`/`help`; after `@docker logs` → running container names (soft-empty if daemon down).

## Recognition list

`is_builtin` matches exactly:

`cd`, `setenv`, `unsetenv`, `env`, `exit`, `set`, `unset`, `alias`, `unalias`, `history`, `jobs`, `fg`, `bg`, `disown`, `source`, `.`, `@`, `bindkey`, `which`, `where`, `repeat`, `sandbox`, `@kube`, `@docker`, `pushd`, `popd`, `dirs`, `return`.

Anything else is treated as an **external** (PATH lookup / relative path), subject to spawn errors (`127` when not found).

## External vs builtin in pipelines

Pipeline stages may mix builtins and externals. Builtin stages in a pipe are handled under `src/exec/pipe/builtin.rs`; externals under `pipe/external.rs`. Subshell stages use `pipe/subshell_stage/`.

## Notes for implementers

- Builtins take `&mut ShellEnvironment` and writers for stdout/stderr
- Validate arguments; do not panic on expected bad input
- Prefer `Result` / status codes over unwind
- Keep each builtin file within project size limits (≤35 lines/fn, ≤7 fns/file) — split into submodules when needed (`bindkey/`, `history/`, `dirstack/`)
