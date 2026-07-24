# Plugins — Oh My Nexus

NEXUS loads interactive config from `~/.nexusrc` (see `src/repl/rc/`). Plugin management is provided by a **separate** companion repo modeled on Oh My Zsh:

**[oh-my-nexus](https://github.com/Emmy3334/oh-my-nexus)** (local clone: `../oh-my-nexus` next to this tree)

## Wire-up

```nexus
setenv NEXUS_OMN "$HOME/.oh-my-nexus"
set omn_theme = git
set omn_plugins = "git directories aliases"
source $NEXUS_OMN/oh-my-nexus.nexus
```

Install:

```sh
sh /path/to/oh-my-nexus/tools/install.sh
# or: git clone … ~/.oh-my-nexus && cp templates/nexusrc.nexus-template ~/.nexusrc
```

## How it maps from Oh My Zsh

| Oh My Zsh | Oh My Nexus / NEXUS |
|-----------|---------------------|
| `~/.zshrc` | `~/.nexusrc` |
| `$ZSH` | `$NEXUS_OMN` |
| `$ZSH_CUSTOM` | `$NEXUS_OMN_CUSTOM` |
| `plugins=(git)` | `set omn_plugins = "git …"` |
| `ZSH_THEME=…` | `set omn_theme = …` |
| `source $ZSH/oh-my-zsh.sh` | `source $NEXUS_OMN/oh-my-nexus.nexus` |
| `*.plugin.zsh` | `*.plugin.nexus` |
| `*.zsh-theme` | `*.nexus-theme` |

## Shell adaptations that matter for plugins

- **`#` comments** — lex skips `#` to end of line when `#` starts a word (needed for OMZ-style scripts).
- **`source` inside `foreach`** — works (body runner applies nested sources).
- **File tests in `if`** — use `{ test -f path }` (not bare `-f`).
- **Do not `exit` inside sourced framework files** — that ends the whole session; the loader prints and skips instead.
- **Brace `{ cmd }` re-tokenizes** — avoid `test -n "$multi word"`; keep flags single-token.

## Startup chain reminder

1. `.nexusenv` — all invocations  
2. `.nexusrc` — interactive TTY (source Oh My Nexus here)  
3. `.nexuslogin` / `.nexuslogout` — login shells  
