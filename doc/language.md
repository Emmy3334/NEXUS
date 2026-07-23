# Language

Grammar and expansions as implemented in `src/parse`, `src/lex`, `src/expand`, `src/glob`, and the control modules.

## Grammar

From `src/parse/mod.rs`:

```text
line     → list
list     → pipeline ( (';' | '&' | '&&' | '||') pipeline )* [ ';' | '&' ]
pipeline → command ( '|' command )*
command  → simple | '(' list ')' redirect*
simple   → ( WORD | redirect )+   # at least one WORD
redirect → ( '>' | '<' | '>>' | '<<' ) WORD
```

Notes:

- Empty commands around `|` / `&&` / `||` are parse errors (`NullCommand`)
- Trailing `;` or `&` is allowed
- Heredoc **bodies** are not on the same physical line; the REPL collects them after parse

## Operators

| Token | Meaning |
|-------|---------|
| `;` | Sequence pipelines |
| `&` | Run preceding pipeline in background; also list separator |
| `&&` | Run next pipeline only if previous status is `0` |
| `\|\|` | Run next pipeline only if previous status is non-zero |
| `\|` | Pipe stdout of left stage to stdin of right |
| `( … )` | Subshell: clone env (jobs cleared), run list |

## Redirects

| Op | Kind | Behavior |
|----|------|----------|
| `<` | `Read` | stdin from file |
| `>` | `Write` | stdout truncate/create |
| `>>` | `Append` | stdout append |
| `<<` | `Heredoc` | stdin from body until delimiter line (`path` is the delimiter) |

Redirects may attach to simple commands or to `( … )` groups. A redirect on a stage overrides the pipe on that fd.

## Word expansions

Handled mainly in `src/expand/`:

| Form | Behavior |
|------|----------|
| `$name` / `${name}` | Local first, then exported |
| `${name:-word}` | If unset or empty → expand `word`; else value |
| `${name:+word}` | If unset or empty → empty; else expand `word` |
| `${#name}` / `${#}` | Character length of value / argc |
| `${name#pat}` / `##` / `%` / `%%` | Strip shortest/longest matching prefix (`#`/`##`) or suffix (`%`/`%%`); `pat` uses `*` / `?` |
| `$?` / `$status` | Last command status |
| `$n` / `$#` / `$*` | Positional / count / all (scripting argv) |
| `` `cmd` `` | Capture stdout of nested command |
| escapes / quotes | Preserved through lex; decoded at expand |

Unmatched globs stay literal (`src/glob/`).

## Glob

Patterns `*`, `?`, and `[…]` expand to matching pathnames. No match → original word.

## Aliases

`src/alias.rs` rewrites `argv[0]` after expand/glob. Skipped for `alias` / `unalias`. Cycle detection prevents infinite expansion.

## History designators (`!`)

`src/history/expand/` supports tcsh-style events (including `!!`, `!n`, `!-n`, `!str`, `!?str?`, `!#`) plus word selection and modifiers. Expansion runs **before** lex on the physical line.

## Control structures

These are **not** ordinary parse productions on one line. The REPL detects headers, collects bodies, then runs them.

### `foreach`

```text
foreach name ( wordlist )
  …body…
end
```

Module: `src/foreach/` + `src/repl/foreach_run.rs`. Nested `foreach`/`while` deepen collection until matching `end`.

### `while`

```text
while ( expression )
  …body…
end
```

Module: `src/while_loop/` (expression evaluator under `expr/`) + `src/repl/while_run.rs`.

### `if`

```text
if ( expression ) then
  …then body…
else if ( expression ) then
  …
else
  …
endif
```

Module: `src/if_block/` + `src/repl/if_collect.rs` / `if_run.rs`.

Expressions for `while` / `if` are evaluated by the while-loop expression engine (comparisons, file tests, etc. as implemented under `while_loop/expr/`).
