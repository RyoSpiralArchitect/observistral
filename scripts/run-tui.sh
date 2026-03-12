#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust (rustup), then retry." >&2
  exit 1
fi

export CARGO_TARGET_DIR="${REPO_ROOT}/.tmp/cargo-target-tui"
mkdir -p "${CARGO_TARGET_DIR}"

echo "[run-tui] cargo run -- tui $*"
echo "[run-tui] CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
exec cargo run -- tui "$@"
