# Security notes

NEXUS is a **local shell**: it runs with the privileges of the user who starts it.
Container and release hardening below reduce accident and dependency risk; they are
not a substitute for least-privilege Docker/kube access or keeping heal off when
you do not need it.

## Runtime model (host)

- Children inherit only the **owned** shell environment (`env_clear` + shell map).
- Heal Wasm runs on the host Wasmtime runtime; Docker heal talks to the local
  daemon; `@kube` / Kube heal use the active kubeconfig.
- Prefer heal off, least-privilege Docker socket / kube credentials, and do not
  export secrets into the shell env you do not need.

## Release binary

`[profile.release]` in `Cargo.toml` enables LTO, single codegen unit, `panic =
"abort"`, and symbol stripping. Linux GNU targets also get PIE via
`.cargo/config.toml`.

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
