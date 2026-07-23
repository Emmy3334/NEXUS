# NEXUS documentation

NEXUS is a Rust **tcsh-style shell** (`nexus`) built as a language processor: **lex → parse → expand/glob → exec**, with an interactive REPL on top. The long-term product name is *The Self-Healing Cloud Shell*; this tree documents what is implemented today.

| Document | Contents |
|----------|----------|
| [overview.md](overview.md) | Goals, package facts, binary vs library, feature map |
| [architecture.md](architecture.md) | End-to-end pipeline, data flow, key types |
| [language.md](language.md) | Grammar, operators, redirects, expansions, control structures |
| [builtins.md](builtins.md) | Every builtin: behavior, result kinds, dispatch |
| [modules.md](modules.md) | `src/` module map and responsibilities |
| [jobs-and-repl.md](jobs-and-repl.md) | Job control, line edition, scripting, specials |
| [testing.md](testing.md) | Integration test layout and how to run gates |
| [conventions.md](conventions.md) | Project rules, layout limits, git-flow |
| [security.md](security.md) | Release/container hardening, CI audit, run flags |

Start with **overview** then **architecture** if you are new to the repo.
