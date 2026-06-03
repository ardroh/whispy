#!/usr/bin/env bash
# Merge one story branch into the integration branch, optionally regenerate OpenAPI,
# then run backend + frontend lint/test gates.

set -euo pipefail

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "$SCRIPT_DIR/common.sh"

EXIT_CONFLICT=100
EXIT_GATE_FAIL=101

usage() {
  cat <<'EOF'
Usage:
  merge-story.sh <integration-branch> <story-branch> <storyId> <openapi-touched> [--gate-only]

Arguments:
  integration-branch  Checked out before merge (e.g. 001-my-feature).
  story-branch        Branch to merge (typically <integration-branch>-StoryId).
  storyId             Merge message tag (e.g. US2).
  openapi-touched     "true" or "false". If true runs openapi export before gates.

Flags:
  --gate-only         Skip git merge (use after resolving conflicts).
                      Still applies openapi export (if openapi-touched) and gates.

Run from repository root.

Exit codes:
  0     Merge succeeded and gates passed.
  100   Unmerged conflicts (git status lists UU).
  101   Gate failure (pytest / lint / openapi drift).
  1     Bad arguments or git error before merge.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

GATE_ONLY=false
positional=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --gate-only)
      GATE_ONLY=true
      shift
      ;;
    *)
      positional+=("$1")
      shift
      ;;
  esac
done

if [[ ${#positional[@]} -lt 4 ]]; then
  usage >&2
  exit 1
fi

INTEGRATION_BRANCH="${positional[0]}"
STORY_BRANCH="${positional[1]}"
STORY_ID="${positional[2]}"
OPENAPI_TOUCHED="${positional[3]}"

REPO_ROOT="$(repo_root)"
cd "$REPO_ROOT"

current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$current_branch" != "$INTEGRATION_BRANCH" ]]; then
  log_status "checking out integration branch '${INTEGRATION_BRANCH}' (was on '${current_branch}')"
  git checkout "$INTEGRATION_BRANCH"
fi

has_unmerged() {
  git status --porcelain | grep -E '^UU' >/dev/null
}

merge_if_needed() {
  if $GATE_ONLY; then
    log_status "(gate-only mode: skipping git merge)"
    return 0
  fi
  log_status "git merge --no-ff '${STORY_BRANCH}'"
  if git merge --no-ff "${STORY_BRANCH}" -m "impl: merge ${STORY_ID}"; then
    return 0
  fi

  if has_unmerged; then
    exit "$EXIT_CONFLICT"
  fi

  log_status "git merge exited non-zero without unmerged conflict entries"
  exit 1
}

commit_openapi_regen() {
  if [[ "$OPENAPI_TOUCHED" != "true" ]]; then
    return 0
  fi
  if [[ ! -f backend/openapi.json ]]; then
    log_status "WARN: openapi-touched=true but backend/openapi.json missing — skipping openapi export"
    return 0
  fi

  log_status "pnpm run openapi:export"
  pnpm run openapi:export

  git add backend/openapi.json
  if git diff --cached --quiet; then
    log_status "(openapi export produced no staged diff)"
    return 0
  fi

  if git commit --amend --no-edit >/dev/null 2>&1; then
    log_status "amended openapi.json into HEAD"
    return 0
  fi

  git commit -m "impl: ${STORY_ID} regenerate openapi"
  log_status "committed openapi regeneration as separate commit"
}

openapi_drift_check() {
  if [[ ! -f backend/openapi.json ]]; then
    log_status "(no backend/openapi.json — skipping drift check)"
    return 0
  fi

  local snapshot
  snapshot="$(mktemp)"
  cp backend/openapi.json "$snapshot"

  log_status "gate: openapi.json matches canonical export output"
  pnpm run openapi:export >/dev/null

  if ! cmp -s "$snapshot" backend/openapi.json; then
    cp "$snapshot" backend/openapi.json
    rm -f "$snapshot"
    log_status "openapi.json differs from regenerated export output"
    return 1
  fi

  rm -f "$snapshot"
  return 0
}

run_gate() {
  log_status "gate: pytest openapi snapshot"
  (
    cd backend
    uv run pytest tests/test_openapi_snapshot.py
  ) || return 1

  log_status "gate: pytest backend/tests"
  (
    cd backend
    uv run pytest tests
  ) || return 1

  log_status "gate: uv run ruff check . (backend)"
  (
    cd backend
    uv run ruff check .
  ) || return 1

  log_status "gate: pnpm lint"
  pnpm lint || return 1

  openapi_drift_check || return 1

  log_status "all gates passed"
  return 0
}

merge_if_needed
commit_openapi_regen

if run_gate; then
  exit 0
fi
exit "$EXIT_GATE_FAIL"
