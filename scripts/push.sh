#!/usr/bin/env bash
set -euo pipefail

REMOTE="origin"
BRANCH="main"
TOKEN=""
USERNAME="x-access-token"

usage() {
  cat <<'USAGE'
Usage: bash ./scripts/push.sh [--remote <name>] [--branch <name>] [--token <pat>] [--username <user>]

Push to GitHub over HTTPS using an Authorization extraheader (no remote rewrite).
If --token is omitted, uses $GITHUB_TOKEN.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote)
      REMOTE="${2:-}"
      shift 2
      ;;
    --branch)
      BRANCH="${2:-}"
      shift 2
      ;;
    --token)
      TOKEN="${2:-}"
      shift 2
      ;;
    --username)
      USERNAME="${2:-}"
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

if [[ -z "${TOKEN}" ]]; then
  TOKEN="${GITHUB_TOKEN:-}"
fi
if [[ -z "${TOKEN}" ]]; then
  echo "[push] Missing token. Set GITHUB_TOKEN or pass --token." >&2
  exit 2
fi

unset HTTP_PROXY HTTPS_PROXY ALL_PROXY GIT_HTTP_PROXY GIT_HTTPS_PROXY || true
unset http_proxy https_proxy all_proxy git_http_proxy git_https_proxy || true
export GIT_TERMINAL_PROMPT=0

pair="${USERNAME}:${TOKEN}"
b64="$(printf '%s' "${pair}" | base64 | tr -d '\n')"
hdr="AUTHORIZATION: basic ${b64}"

echo "[push] git push ${REMOTE} ${BRANCH}"
git \
  -c http.proxy= \
  -c https.proxy= \
  -c http.sslBackend=openssl \
  -c "http.extraheader=${hdr}" \
  push "${REMOTE}" "${BRANCH}"

