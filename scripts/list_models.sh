#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${MISTRAL_API_KEY:-}" ]]; then
  echo "[list_models] Missing MISTRAL_API_KEY" >&2
  exit 2
fi

curl -fsS "https://api.mistral.ai/v1/models" \
  -H "Authorization: Bearer ${MISTRAL_API_KEY}" \
  | python3 - <<'PY'
import json, re, sys

doc = json.load(sys.stdin)
data = doc.get("data") or []
pat = re.compile(r"(dev|code|stral)", re.I)
ids = []
for item in data:
    if isinstance(item, dict):
        mid = str(item.get("id") or "")
        if pat.search(mid):
            ids.append(mid)
for mid in sorted(set(ids)):
    print(mid)
PY

