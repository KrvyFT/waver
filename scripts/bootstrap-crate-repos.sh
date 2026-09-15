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

repo_exists() {
  local full="$1"
  # Prefer REST: GraphQL view can lag right after create / rename.
  local code
  code="$(gh api -i "repos/${full}" 2>/dev/null | head -n1 | awk '{print $2}')"
  [[ "$code" == "200" ]]
}

for r in "${REPOS[@]}"; do
  full="${ORG}/${r}"
  if repo_exists "$full"; then
    echo "exists: ${full}"
    continue
  fi
  echo "creating: ${full}"
  if out="$(gh repo create "${full}" \
      --public \
      --description "waver workspace crate — develop in https://github.com/${ORG}/waver" \
      --disable-issues \
      --disable-wiki 2>&1)"; then
    echo "created: ${full}"
  else
    # Idempotent: concurrent / lagged "exists" is OK.
    if grep -qiE 'already exists|name already exists' <<<"$out"; then
      echo "exists (create raced): ${full}"
    else
      echo "$out" >&2
      exit 1
    fi
  fi
done

./scripts/repos.sh remotes
./scripts/repos.sh push
echo "done."
