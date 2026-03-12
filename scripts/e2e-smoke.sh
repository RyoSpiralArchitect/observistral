#!/usr/bin/env bash
set -euo pipefail

HOST="127.0.0.1"
PORT="18090"
WORKSPACE_ROOT=""

usage() {
  cat <<'USAGE'
Usage: bash ./scripts/e2e-smoke.sh [--host <host>] [--port <port>] [--workspace <dir>]

Starts the Python Lite server and verifies /api/status + /api/exec end-to-end.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host)
      HOST="${2:-}"
      shift 2
      ;;
    --port)
      PORT="${2:-}"
      shift 2
      ;;
    --workspace)
      WORKSPACE_ROOT="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found. Install Python 3, then retry." >&2
  exit 1
fi
if ! command -v curl >/dev/null 2>&1; then
  echo "curl not found. Install curl, then retry." >&2
  exit 1
fi
if ! command -v git >/dev/null 2>&1; then
  echo "git not found. Install git, then retry." >&2
  exit 1
fi

if [[ -n "${WORKSPACE_ROOT}" ]]; then
  ws="${WORKSPACE_ROOT}"
else
  ws="${REPO_ROOT}/.tmp/e2e-smoke-work"
fi
mkdir -p "${ws}"

echo "Starting Lite server: http://${HOST}:${PORT}/  workspace=${ws}"
python3 scripts/serve_lite.py --host "${HOST}" --port "${PORT}" --workspace "${ws}" >/dev/null 2>&1 &
pid="$!"

cleanup() {
  kill -9 "${pid}" >/dev/null 2>&1 || true
  wait "${pid}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

sleep 0.8

status="$(curl -fsS --max-time 5 "http://${HOST}:${PORT}/api/status")"
python3 - <<'PY' <<<"${status}"
import json, sys
j = json.loads(sys.stdin.read() or "{}")
if not j.get("ok"):
    raise SystemExit("status.ok is false")
PY

tid="thread_e2e_smoke"
cwd=".tmp/${tid}"
cmd="mkdir -p demo-repo && cd demo-repo && git init"
body="$(CMD="${cmd}" CWD="${cwd}" python3 - <<'PY'
import json, os
print(json.dumps({"command": os.environ["CMD"], "cwd": os.environ["CWD"], "timeout_seconds": 60}))
PY
)"

ex="$(curl -fsS --max-time 15 -X POST -H "Content-Type: application/json" --data "${body}" "http://${HOST}:${PORT}/api/exec")"
python3 - <<'PY' <<<"${ex}"
import json, sys
j = json.loads(sys.stdin.read() or "{}")
exit_code = int(j.get("exit_code", 1))
if exit_code != 0:
    raise SystemExit(f"exec failed: exit_code={exit_code} stderr={j.get('stderr','')}")
PY

git_dir="${ws}/${cwd}/demo-repo/.git"
if [[ ! -d "${git_dir}" ]]; then
  echo "expected git dir missing: ${git_dir}" >&2
  exit 1
fi

echo "E2E smoke OK: ${git_dir}"
