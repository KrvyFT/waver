#!/usr/bin/env bash
# Create empty per-crate GitHub repos (if missing) and subtree-push from the umbrella.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.local/bin:${PATH}"

ORG="${WAVER_GITHUB_ORG:-KrvyFT}"
REPOS=(waver-core waver-dsp waver-engine waver-ui)

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI not found. Install: https://cli.github.com/" >&2
  exit 1
fi

if ! gh auth status -h github.com >/dev/null 2>&1; then
  echo "gh is not logged in. Run: gh auth login" >&2
  exit 1
fi

for r in "${REPOS[@]}"; do
  if gh repo view "${ORG}/${r}" >/dev/null 2>&1; then
    echo "exists: ${ORG}/${r}"
  else
    echo "creating: ${ORG}/${r}"
    gh repo create "${ORG}/${r}" \
      --public \
      --description "waver workspace crate — develop in https://github.com/${ORG}/waver" \
      --disable-issues \
      --disable-wiki
  fi
done

./scripts/repos.sh remotes
./scripts/repos.sh push
echo "done."
