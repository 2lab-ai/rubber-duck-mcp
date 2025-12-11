#!/usr/bin/env bash
# One-command release helper for rubber-duck-mcp.
# - Bumps patch version by default (or use: scripts/release.sh X.Y.Z)
# - Syncs Cargo.toml, Cargo.lock, npm/package.json
# - Commits, tags, pushes, runs npm publish (uses OTP if NPM_OTP is set)
# Prereqs: git clean tree, npm login, 2FA token (set NPM_OTP), cargo-dist CI configured.

set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is dirty. Commit or stash changes first." >&2
  exit 1
fi

CURRENT_VERSION=$(python - <<'PY'
import pathlib, re, sys
txt = pathlib.Path("Cargo.toml").read_text()
m = re.search(r'^version\s*=\s*"([^"]+)"', txt, re.M)
if not m:
    sys.exit("version not found in Cargo.toml")
print(m.group(1))
PY
)

REQUESTED_VERSION="${1:-}"
if [[ -n "$REQUESTED_VERSION" ]]; then
  NEW_VERSION="$REQUESTED_VERSION"
else
  IFS=. read -r MAJ MIN PATCH <<<"$CURRENT_VERSION"
  PATCH=$((PATCH + 1))
  NEW_VERSION="$MAJ.$MIN.$PATCH"
fi

if git tag -l "v$NEW_VERSION" >/dev/null 2>&1 && git tag -l "v$NEW_VERSION" | grep -q .; then
  echo "Tag v$NEW_VERSION already exists" >&2
  exit 1
fi

python - <<PY
import pathlib, re, sys
new = "$NEW_VERSION"

def bump_cargo_toml():
    path = pathlib.Path("Cargo.toml")
    txt = path.read_text()
    def repl(m): return f"{m.group(1)}{new}{m.group(3)}"
    out, n = re.subn(r'(?m)^(version\s*=\s*")([^"]+)(")', repl, txt, count=1)
    if n == 0:
        sys.exit("Failed to bump Cargo.toml version")
    path.write_text(out)

def bump_cargo_lock():
    path = pathlib.Path("Cargo.lock")
    txt = path.read_text()
    def repl(m): return f"{m.group(1)}{new}{m.group(3)}"
    out, n = re.subn(r'(\[\[package\]\]\nname = "rubber-duck-mcp"\nversion = ")([^"]+)(")', repl, txt, count=1)
    if n == 0:
        sys.exit("Failed to bump Cargo.lock version for rubber-duck-mcp")
    path.write_text(out)

def bump_npm():
    path = pathlib.Path("npm/package.json")
    pkg = __import__("json").loads(path.read_text())
    pkg["version"] = new
    path.write_text(__import__("json").dumps(pkg, indent=2) + "\n")

bump_cargo_toml()
bump_cargo_lock()
bump_npm()
PY

echo "Version: $CURRENT_VERSION -> $NEW_VERSION"

git add Cargo.toml Cargo.lock npm/package.json
git commit -m "Release v$NEW_VERSION"
git tag "v$NEW_VERSION"

git push
git push origin "v$NEW_VERSION"

OTP_ARG=""
if [[ -n "${NPM_OTP:-}" ]]; then
  OTP_ARG="--otp=$NPM_OTP"
fi

pushd npm >/dev/null
npm publish --access public $OTP_ARG
popd >/dev/null

echo "Release pipeline kicked off for v$NEW_VERSION"
