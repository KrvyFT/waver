#!/usr/bin/env bash
# Sync workspace crates to/from per-crate GitHub remotes via git subtree.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ORG="${WAVER_GITHUB_ORG:-KrvyFT}"
BRANCH="${WAVER_CRATE_BRANCH:-main}"

declare -A CRATES=(
  [core]="waver-core:crates/waver-core"
  [dsp]="waver-dsp:crates/waver-dsp"
  [engine]="waver-engine:crates/waver-engine"
  [ui]="waver-ui:crates/waver-ui"
)

usage() {
  cat <<'EOF'
Usage: scripts/repos.sh <command> [crate...]

Commands:
  remotes   Add git remotes for each crate (idempotent)
  push      git subtree push to crate remotes
  pull      git subtree pull from crate remotes (onto current branch)
  status    Show remotes and whether working tree is clean

Crate aliases: core dsp engine ui  (default: all)

Env:
  WAVER_GITHUB_ORG     default KrvyFT
  WAVER_CRATE_BRANCH   default main
EOF
}

all_aliases() {
  printf '%s\n' "${!CRATES[@]}" | sort
}

resolve_aliases() {
  if [[ $# -eq 0 ]]; then
    all_aliases
    return
  fi
  local a
  for a in "$@"; do
    [[ -n "${CRATES[$a]+x}" ]] || {
      echo "unknown crate alias: $a" >&2
      exit 1
    }
    echo "$a"
  done
}

parse_entry() {
  # entry = "repo:prefix"
  REPO="${1%%:*}"
  PREFIX="${1#*:}"
  REMOTE="crate-$REPO"
  URL="https://github.com/${ORG}/${REPO}.git"
}

cmd_remotes() {
  local alias entry
  while read -r alias; do
    parse_entry "${CRATES[$alias]}"
    if git remote get-url "$REMOTE" >/dev/null 2>&1; then
      git remote set-url "$REMOTE" "$URL"
      echo "updated remote $REMOTE -> $URL"
    else
      git remote add "$REMOTE" "$URL"
      echo "added remote $REMOTE -> $URL"
    fi
  done < <(resolve_aliases "$@")
}

require_clean() {
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "working tree not clean; commit or stash before subtree push/pull" >&2
    git status -sb >&2
    exit 1
  fi
}

cmd_push() {
  require_clean
  cmd_remotes "$@"
  local alias
  while read -r alias; do
    parse_entry "${CRATES[$alias]}"
    echo "==> subtree push $PREFIX -> $REMOTE:$BRANCH"
    git subtree push --prefix="$PREFIX" "$REMOTE" "$BRANCH"
  done < <(resolve_aliases "$@")
}

cmd_pull() {
  require_clean
  cmd_remotes "$@"
  local alias
  while read -r alias; do
    parse_entry "${CRATES[$alias]}"
    echo "==> subtree pull $REMOTE:$BRANCH -> $PREFIX"
    git subtree pull --prefix="$PREFIX" "$REMOTE" "$BRANCH" --squash
  done < <(resolve_aliases "$@")
}

cmd_status() {
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "working tree: DIRTY"
    git status -sb
  else
    echo "working tree: clean"
  fi
  echo
  local alias
  while read -r alias; do
    parse_entry "${CRATES[$alias]}"
    if git remote get-url "$REMOTE" >/dev/null 2>&1; then
      echo "$alias  $PREFIX  $(git remote get-url "$REMOTE")"
    else
      echo "$alias  $PREFIX  (no remote; run: $0 remotes)"
    fi
  done < <(all_aliases)
}

main() {
  local cmd="${1:-}"
  shift || true
  case "$cmd" in
    remotes) cmd_remotes "$@" ;;
    push) cmd_push "$@" ;;
    pull) cmd_pull "$@" ;;
    status) cmd_status ;;
    -h | --help | help | "") usage ;;
    *)
      echo "unknown command: $cmd" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
