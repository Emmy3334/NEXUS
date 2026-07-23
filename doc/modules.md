# Modules (`src/`)

Library surface is declared in `src/lib.rs`. Below: responsibility of each public (or major) module and where to look for implementation detail.

## Language processor

| Module | Path | Responsibility |
|--------|------|----------------|
| **lex** | `src/lex/` | Tokenize input into words and operators; quote / backtick awareness (`scan.rs`, `quote.rs`) |
| **parse** | `src/parse/` | Build `CommandList` / `Pipeline` / redirects; parser under `parser/` |
| **expand** | `src/expand/` | Brace `{a,b}`, `$` / backticks / field splitting → `ExpandedWord` |
| **functions** | `src/functions/` | Define/call shell functions (`name() {…}`); `return` |
| **glob** | `src/glob/` | Pathname expansion (`* ? […] **`); walk + globstar + bracket |
| **alias** | `src/alias.rs` | Post-expand `argv[0]` rewrite |

## Runtime

| Module | Path | Responsibility |
|--------|------|----------------|
| **exec** | `src/exec/` | `execute_list`, pipes, redirects/heredoc, process spawn, background, subshell, repeat, capture |
| **env** | `src/env/` | `ShellEnvironment`: vars, locals, aliases, functions, argv, history, keys, dirstack, healers, jobs |
| **builtins** | `src/builtins/` | Builtin implementations + dispatch |
| **repl** | `src/repl/` | Main loop, script runner, control collect/run, line edition |
| **pathfind** | `src/pathfind/` | PATH search for `which` / `where` / (related resolution) |
| **heal** | `src/heal/` | Self-heal seam + backends + status + opt-in command→image map |
| **sandbox** | `src/sandbox/` | Wasmtime WASI runner + on-disk module cache (`install`/`list`/`rm`) |
| **kube** | `src/kube/` | Native Kubernetes client (nodes, pods, logs, exec) via kube-rs |

### Exec internals (important subtrees)

| Subtree | Role |
|---------|------|
| `exec/list.rs` | Walk pipelines; `&&` / `\|\|` |
| `exec/command.rs` | One simple command: expand → alias → builtin/external |
| `exec/pipe/` | Multi-stage pipelines |
| `exec/redirect/` | File redirects + `heredoc/` |
| `exec/background/` | Background spawn / shell job registration |
| `exec/subshell/` | `( … )` execution + stdio apply |
| `exec/process.rs` | Shared child helpers, env inheritance |
| `exec/repeat.rs` | Honor `BuiltinResult::Repeat` |

## Interactive / tcsh features

| Module | Path | Responsibility |
|--------|------|----------------|
| **history** | `src/history/` | Store + `!` expand (`store/`, `expand/`); histfile path helper |
| **jobs** | `src/jobs/` | Job table; Unix pgrp/tty vs stub |
| **keybind** | `src/keybind/` | Emacs/vi maps, parse/display for `bindkey` |
| **specials** | `src/specials/` | `precmd`, `cwdcmd`, `ignoreeof` |
| **foreach** | `src/foreach/` | Header parse / `end` helpers |
| **case_block** | `src/case_block/` | `case` / `esac` header + pattern match |
| **while_loop** | `src/while_loop/` | Header + expression evaluator |
| **if_block** | `src/if_block/` | `if` / `else if` / `else` / `endif` headers |

## Binary

| Path | Role |
|------|------|
| `src/bin/nexus.rs` | `main`: script vs REPL; exit 84 on I/O failure |

## Design constraints (structure)

Standing project rules force **small files**:

- No function longer than **35 lines**
- No source file with more than **7 functions** (including methods)

When a file would exceed either limit, split into a folder + `mod.rs` (as already done for `exec/pipe`, `builtins/bindkey`, `repl/line_edit/tty`, etc.).
