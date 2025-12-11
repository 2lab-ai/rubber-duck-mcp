#!/usr/bin/env bash
# Poll for a published npm version and its GitHub release asset, then run npx.
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
    echo "Found ${PACKAGE} in npm."
    break
  fi
  if (( SECONDS >= DEADLINE )); then
    echo "Timed out after 10 minutes waiting for ${PACKAGE}. Try again later." >&2
    exit 1
  fi
  sleep 10
done

# Also wait for the GitHub release asset for this platform, matching install.js behavior.
platform="$(uname -s)-$(uname -m)"
target=""
case "$platform" in
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Linux-arm64) target="aarch64-unknown-linux-gnu" ;;
  Linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
  *) target="" ;;
esac

if [[ -n "$target" ]]; then
  asset="rubber-duck-mcp-${target}.tar.xz"
  url="https://github.com/2lab-ai/rubber-duck-mcp/releases/download/v${VERSION}/${asset}"
  echo "Waiting for GitHub release asset ${asset} (timeout 10m, poll 10s)..."
  while true; do
    if curl -sfI "$url" >/dev/null 2>&1; then
      echo "Found release asset for ${target}."
      break
    fi
    if (( SECONDS >= DEADLINE )); then
      echo "Timed out waiting for GitHub asset ${asset}. Try again later." >&2
      exit 1
    fi
    sleep 10
  done
else
  echo "Unknown platform $platform; skipping GitHub asset precheck."
fi

NPX_FLAGS=("--yes" "${PACKAGE}")
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  NPX_FLAGS+=("${EXTRA_ARGS[@]}")
fi

exec npx "${NPX_FLAGS[@]}"
