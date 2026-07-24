#!/usr/bin/env bash
# Hot-path timing: builtin loop vs external spawn (compare with bash/zsh locally).
# Usage: ./scripts/bench_hotpath.sh [iterations]
# Example results (placeholder — run on your machine):
#   nexus true x10000: ~0.05s (in-process builtin)
#   nexus /bin/echo x1000:  ~0.8s  (PATH resolve + fork/exec per call)
#   bash -c '…' equivalents vary by shell and hardware.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NEXUS="${NEXUS_BIN:-$ROOT/target/release/nexus}"
ITERS="${1:-1000}"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/nexus_bench.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

if [[ ! -x "$NEXUS" ]]; then
  echo "Building release nexus…" >&2
  (cd "$ROOT" && cargo build --release)
fi

printf 'repeat %s true\n' "$ITERS" >"$TMP/builtin.nx"
printf 'repeat %s /bin/echo ok\n' "$ITERS" >"$TMP/external.nx"

echo "=== bench_hotpath (iters=$ITERS, nexus=$NEXUS) ==="

echo -n "builtin true: "
/usr/bin/time -p "$NEXUS" "$TMP/builtin.nx" 2>&1 | awk '/real/ {print $2 "s"}'

echo -n "external /bin/echo: "
/usr/bin/time -p "$NEXUS" "$TMP/external.nx" 2>&1 | awk '/real/ {print $2 "s"}'

echo "Done. Compare: time bash -c \"for i in \$(seq 1 $ITERS); do true; done\""
echo "              time bash -c \"for i in \$(seq 1 $ITERS); do /bin/echo ok; done\""
