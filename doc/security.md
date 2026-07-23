# Security notes

NEXUS is a **local shell**: it runs with the privileges of the user who starts it.
Container and release hardening below reduce accident and dependency risk; they are
not a substitute for least-privilege Docker/kube access or keeping heal off when
you do not need it.

## Runtime model (host)

- Children inherit only the **owned** shell environment (`env_clear` + shell map).
- **PATH jail** (default on): empty / relative `PATH` components are dropped before
  search and before passing `PATH` to children (blocks cwd hijack via `PATH=.:…`).
  Disable with `path_jail=0` or `NEXUS_PATH_JAIL=0`. Explicit `./cmd` and absolute
  paths are unchanged.
- **Child rlimits** (default off): set `rlimit=1` / `NEXUS_RLIMIT=1` for modest
  `setrlimit` on externals (`RLIMIT_CPU` 30s, `RLIMIT_NOFILE` 256; on Linux also
  `RLIMIT_AS` 512MiB and `RLIMIT_NPROC` 64 — macOS rejects finite address-space
  limits with `EINVAL`). Override with `rlimit_cpu` / `NEXUS_RLIMIT_CPU` (and
  `_nofile` / `_as` / `_nproc`). Not a multi-tenant sandbox; no seccomp.
- Heal Wasm runs on the host Wasmtime runtime; Docker heal talks to the local
  daemon; `@kube` / Kube heal use the active kubeconfig.
- Docker/Kube heal forward **no** exported vars by default (`heal_env=none`).
  Opt in with `heal_env` / `NEXUS_HEAL_ENV` (`*`/`all` or `FOO,BAR`); `PATH` /
  loader-path keys are always scrubbed. Prefer heal off and least-privilege
  Docker socket / kube credentials.

## Release binary

`[profile.release]` in `Cargo.toml` enables LTO, single codegen unit, `panic =
"abort"`, and symbol stripping. Do **not** set global `link-arg=-pie` via
`.cargo/config.toml` or `RUSTFLAGS` — Cargo/rustc apply those flags to
proc-macro dylibs as well, which breaks Linux CI (`undefined reference to
main` when linking e.g. `serde_derive`). rustc already emits PIE for Linux GNU
binaries by default.

## Container image

`Dockerfile` is a multi-stage build:

1. **Builder** — `rust:1.89-bookworm`, `cargo build --release --locked --bin nexus`
2. **Runtime** — `gcr.io/distroless/cc-debian12:nonroot` (non-root UID, minimal glibc
   userland)

`distroless/static` is intentionally not used: Wasmtime/cranelift and related
native deps expect glibc / libgcc, so a fully static musl image is not practical
today.

### Recommended `docker run` flags

```bash
docker build -t nexus:local .
docker run --rm -it \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --read-only \
  --tmpfs /tmp:rw,nosuid,nodev,size=64m \
  nexus:local
```

Notes:

- Distroless has no shell and almost no PATH tools — use the image as a hardened
  entrypoint for `nexus`, or mount tools/cwd as needed.
- Dropping capabilities and using a read-only rootfs limits what a compromised
  process can do inside the container; `/tmp` tmpfs keeps temporary writes
  possible without a writable root.
- Mounting the Docker socket or a kubeconfig into this container re-expands the
  trust boundary — only do that when you intend heal / `@docker` / `@kube`.

## CI

GitHub Actions (`.github/workflows/ci.yml`) runs `fmt`, `clippy -D warnings`,
integration tests, a **release** build, and **`cargo audit`** (RustSec) on pushes
and PRs to `develop`.

### Known `cargo audit` hits (as of this slice)

Pinned **wasmtime 24** currently reports several RustSec advisories (including
criticals fixed in **≥36.0.7** / **≥42.0.2** / **≥43.0.1**). Until the sandbox
deps are bumped, the **audit** CI job will fail loudly — that is intentional.
Bump `wasmtime` / `wasmtime-wasi` (and re-gate sandbox tests) in a follow-up
slice; do not silence critical advisories in `audit.toml` without a tracked
exception.
