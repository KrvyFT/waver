#!/usr/bin/env bash
# Helpers for crate submodules under crates/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ORG="${WAVER_GITHUB_ORG:-KrvyFT}"

declare -A CRATES=(
  [core]="waver-core"
  [dsp]="waver-dsp"
  [engine]="waver-engine"
  [ui]="waver-ui"
)

usage() {
  cat <<'EOF'
Usage: scripts/repos.sh <command> [crate...]

Commands:
  status    Submodule status (+ dirty check inside each)
  push      In each crate: git push (current branch → its origin)
  pull      git submodule update --init --remote --merge
  sync      After crate commits: stage updated submodule pointers in umbrella

Crate aliases: core dsp engine ui  (default: all)

Typical flow after editing crates/waver-core:
  cd crates/waver-core
  git add -A && git commit -m "…" && git push
  cd ../..
  ./scripts/repos.sh sync
  git commit -m "chore: bump waver-core submodule"
  git push
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

path_for() {
  echo "crates/${CRATES[$1]}"
}

cmd_status() {
  git submodule status
  echo
  local alias path
  while read -r alias; do
    path="$(path_for "$alias")"
    if [[ ! -e "$path/.git" && ! -f "$path/.git" ]]; then
      echo "$alias  NOT initialized (run: git submodule update --init)"
      continue
    fi
    (
      cd "$path"
      branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')"
      dirty=""
      [[ -n "$(git status --porcelain)" ]] && dirty=" DIRTY"
      remote="$(git remote get-url origin 2>/dev/null || echo '(no origin)')"
      echo "$alias  $branch$dirty  $remote"
    )
  done < <(resolve_aliases "$@")
}

cmd_push() {
  local alias path
  while read -r alias; do
    path="$(path_for "$alias")"
    echo "==> push $path"
    (
      cd "$path"
      if [[ -n "$(git status --porcelain)" ]]; then
        echo "dirty worktree in $path; commit first" >&2
        exit 1
      fi
      git push -u origin HEAD
    )
  done < <(resolve_aliases "$@")
}

cmd_pull() {
  git submodule update --init --remote --merge
}

cmd_sync() {
  local alias path
  while read -r alias; do
    path="$(path_for "$alias")"
    git add "$path"
  done < <(resolve_aliases "$@")
  git status -sb
  echo "staged submodule pointers; commit in the umbrella when ready"
}

main() {
  local cmd="${1:-}"
  shift || true
  case "$cmd" in
    status) cmd_status "$@" ;;
    push) cmd_push "$@" ;;
    pull) cmd_pull ;;
    sync) cmd_sync "$@" ;;
    -h | --help | help | "") usage ;;
    *)
      echo "unknown command: $cmd" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
