#!/usr/bin/env bash
set -euo pipefail

REMOTE="origin"
BRANCH="main"
HOSTNAME="ssh.github.com"
PORT="443"
IDENTITY_FILE="${HOME}/.ssh/id_ed25519"

usage() {
  cat <<'USAGE'
Usage: bash ./scripts/push_ssh.sh [--remote <name>] [--branch <name>] [--hostname <host>] [--port <n>] [--identity-file <path>]

Push to GitHub over SSH pinned to ssh.github.com:443 (works in restricted networks).
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
    --hostname)
      HOSTNAME="${2:-}"
      shift 2
      ;;
    --port)
      PORT="${2:-}"
      shift 2
      ;;
    --identity-file)
      IDENTITY_FILE="${2:-}"
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

if [[ ! -f "${IDENTITY_FILE}" ]]; then
  echo "[push_ssh] Missing IdentityFile: ${IDENTITY_FILE}" >&2
  echo "[push_ssh] Generate one with: ssh-keygen -t ed25519 -f \"${IDENTITY_FILE}\"" >&2
  exit 2
fi

unset HTTP_PROXY HTTPS_PROXY ALL_PROXY GIT_HTTP_PROXY GIT_HTTPS_PROXY || true
unset http_proxy https_proxy all_proxy git_http_proxy git_https_proxy || true
export GIT_TERMINAL_PROMPT=0

remote_url="$(git remote get-url "${REMOTE}" 2>/dev/null || true)"
remote_url="$(printf '%s' "${remote_url}" | tr -d '\r\n')"
if [[ -z "${remote_url}" ]]; then
  echo "[push_ssh] Missing remote URL for: ${REMOTE}" >&2
  exit 3
fi

path=""
if [[ "${remote_url}" == git@github.com:* ]]; then
  path="${remote_url#git@github.com:}"
elif [[ "${remote_url}" == ssh://git@github.com/* ]]; then
  path="${remote_url#ssh://git@github.com/}"
elif [[ "${remote_url}" == https://github.com/* ]]; then
  path="${remote_url#https://github.com/}"
elif [[ "${remote_url}" =~ ^ssh://git@ssh\.github\.com:[0-9]+/?(.+)$ ]]; then
  path="${BASH_REMATCH[1]}"
fi

if [[ -z "${path}" ]]; then
  echo "[push_ssh] Unsupported remote URL (expected GitHub): ${remote_url}" >&2
  exit 4
fi

if [[ "${path}" != *.git ]]; then
  path="${path}.git"
fi

push_url="ssh://git@${HOSTNAME}:${PORT}/${path}"

# Best-effort host key accept without prompting. Ignore failures.
ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -p "${PORT}" -T "git@${HOSTNAME}" >/dev/null 2>&1 || true

echo "[push_ssh] git push ${REMOTE} ${BRANCH} (via ${HOSTNAME}:${PORT})"
export GIT_SSH_COMMAND="ssh -i ${IDENTITY_FILE} -o BatchMode=yes -o StrictHostKeyChecking=accept-new -p ${PORT}"
git push "${push_url}" "${BRANCH}"

