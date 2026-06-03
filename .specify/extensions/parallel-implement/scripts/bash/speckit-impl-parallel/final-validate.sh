#!/usr/bin/env bash
# Final lint + pytest + openapi consistency check for integration branch.

set -euo pipefail

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "$SCRIPT_DIR/common.sh"

usage() {
  cat <<'EOF'
Runs (from repo root):
  1. uv run ruff check .   (inside backend/)
  2. pnpm lint             (frontend workspace)
  3. uv run pytest tests   (inside backend/, full backend suite)
  4. openapi.json drift    (pnpm openapi:export must not change tracked file)

Exits non-zero on first failure section.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

REPO_ROOT="$(repo_root)"
cd "$REPO_ROOT"

log_status "final-validate: ruff (backend)"
(
  cd backend
  uv run ruff check .
) || exit 1

log_status "final-validate: pnpm lint"
pnpm lint || exit 1

log_status "final-validate: pytest (backend/tests)"
(
  cd backend
  uv run pytest tests
) || exit 1

if [[ -f backend/openapi.json ]]; then
  snapshot="$(mktemp)"
  cp backend/openapi.json "$snapshot"
  log_status "final-validate: openapi drift check"
  pnpm run openapi:export >/dev/null
  if ! cmp -s "$snapshot" backend/openapi.json; then
    cp "$snapshot" backend/openapi.json
    rm -f "$snapshot"
    log_status "openapi.json differs from regenerated export"
    exit 1
  fi
  rm -f "$snapshot"
fi

log_status "final-validate: all sections passed"
