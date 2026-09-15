#!/usr/bin/env bash
# One-shot: convert crates/* from in-tree (subtree) paths to git submodules.
# Run from a clean umbrella worktree. Requires network + gh remotes already populated.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ORG="${WAVER_GITHUB_ORG:-KrvyFT}"
BRANCH="${WAVER_CRATE_BRANCH:-main}"
CRATES=(waver-core waver-dsp waver-engine waver-ui)

if [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree not clean; commit or stash first" >&2
  git status -sb >&2
  exit 1
fi

if [[ -f .gitmodules ]] && grep -q 'crates/waver-core' .gitmodules 2>/dev/null; then
  echo "submodules already configured; nothing to do"
  git submodule status
  exit 0
fi

echo "==> syncing current in-tree crates to remotes (last subtree push)"
for name in "${CRATES[@]}"; do
  remote="crate-${name}"
  url="https://github.com/${ORG}/${name}.git"
  if git remote get-url "$remote" >/dev/null 2>&1; then
    git remote set-url "$remote" "$url"
  else
    git remote add "$remote" "$url"
  fi
  echo "    subtree push crates/${name} -> ${remote}:${BRANCH}"
  git subtree push --prefix="crates/${name}" "$remote" "$BRANCH"
done

echo "==> removing in-tree crate paths from umbrella"
for name in "${CRATES[@]}"; do
  git rm -rf "crates/${name}"
done

echo "==> adding submodules"
for name in "${CRATES[@]}"; do
  git submodule add -b "$BRANCH" "https://github.com/${ORG}/${name}.git" "crates/${name}"
done

git add .gitmodules
git status -sb
echo
echo "Next:"
echo "  git commit -m 'chore: track workspace crates as git submodules'"
echo "  git push origin HEAD"
echo
echo "Afterwards, in any crate:"
echo "  cd crates/waver-core && git checkout -b topic && … && git push -u origin HEAD"
echo
echo "Optional cleanup of old subtree remotes:"
echo "  git remote remove crate-waver-core"
echo "  git remote remove crate-waver-dsp"
echo "  git remote remove crate-waver-engine"
echo "  git remote remove crate-waver-ui"
