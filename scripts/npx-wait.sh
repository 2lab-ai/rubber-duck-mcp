#!/usr/bin/env bash
# Poll npm for a specific version until it exists, then run npx for that version.
# Usage: scripts/npx-wait.sh [version] [-- extra npx args...]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(node -e "console.log(require('./npm/package.json').version)")"
fi

# Allow passing extra args to npx after --
EXTRA_ARGS=()
if [[ "$#" -gt 0 ]]; then
  shift
fi
if [[ "${1:-}" == "--" ]]; then
  shift
  EXTRA_ARGS=("$@")
fi

PACKAGE="@2lab.ai/rubber-duck-mcp@${VERSION}"
DEADLINE=$((SECONDS + 600)) # 10 minutes

echo "Waiting for ${PACKAGE} to appear in npm registry (timeout 10m, poll 10s)..."
while true; do
  if npm view "${PACKAGE}" version >/dev/null 2>&1; then
    echo "Found ${PACKAGE}, launching npx..."
    break
  fi
  if (( SECONDS >= DEADLINE )); then
    echo "Timed out after 10 minutes waiting for ${PACKAGE}. Try again later." >&2
    exit 1
  fi
  sleep 10
done

NPX_FLAGS=("--yes" "${PACKAGE}")
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  NPX_FLAGS+=("${EXTRA_ARGS[@]}")
fi

exec npx "${NPX_FLAGS[@]}"
