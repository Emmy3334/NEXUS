# Conventions

Standing rules for this repository (also mirrored under `.cursor/rules/`).

## Clean code limits

- Meaningful names; single-responsibility functions
- **≤ 35 lines** per function
- **≤ 7 functions** per source file (including methods) — split into modules when exceeded
- Errors via `Result` / `Option`; no panic on expected input
- Prefer borrowing; comments explain **why**
- No unused `pub` APIs; no dead parameters “for later”

## Shell conventions

- Pipeline: **lex → parse → exec** (with expand/glob/alias between parse and run)
- Prompt `$> ` only on TTY stdin
- User-facing errors on **stderr**
- Program I/O failure exit **84**
- External not found → **127** + `{cmd}: Command not found.`
- Propagate child exit status; EOF returns last status (subject to `ignoreeof`)
- Children inherit only the **owned** shell environ map

## Git workflow

- Feature / fix / chore branches from **`develop`**
- PR base is always **`develop`** (merge commit, not squash)
- Do not push straight to `main` / `develop`
- Conventional Commits; branch names like `feature/kebab-case`
- Do not commit PDFs or `/target/`

## Rust review before finishing a slice

1. Load `.cursor/skills/rust-code-review/SKILL.md`
2. Self-check against [RCRG](https://github.com/ZhangHanDong/rust-code-review-guidelines)
3. Gates: `fmt`, `clippy -D warnings`, `cargo test`
4. Paste an explicit review (or “No Critical/Major findings” + gates) before commit/PR on a feature branch

## Optimization

Readability first. Measure before clever opts. On hot paths (REPL read / lex / parse / exec): reuse buffers; prefer contiguous data; avoid `Box<dyn Trait>` in inner loops.

## Documentation in this folder

When you add a major feature:

1. Update the feature map in [overview.md](overview.md)
2. Extend [language.md](language.md) / [builtins.md](builtins.md) as appropriate
3. Note new `src/` modules in [modules.md](modules.md)
4. Point new test modules from [testing.md](testing.md)
