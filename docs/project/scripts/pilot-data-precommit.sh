#!/usr/bin/env bash
# pilot-data-precommit.sh — Pre-commit hook to block commits containing PII
# in pilot data files.
#
# Install as a git hook:
#   ln -sf ../../scripts/pilot-data-precommit.sh .git/hooks/pre-commit
#
# Or add to your CI pipeline:
#   ./scripts/pilot-data-precommit.sh
#
# Scans files under pilot-data/ and pilots/ that are staged for commit.
# Exits 1 if PII is detected, blocking the commit.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PII_SCANNER="${SCRIPT_DIR}/pilot-pii-scan.sh"

if [[ ! -x "$PII_SCANNER" ]]; then
    echo "ERROR: PII scanner not found at ${PII_SCANNER}" >&2
    exit 1
fi

# Get staged files under pilot-data/ or pilots/
STAGED_PILOT_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '^(pilot-data|pilots)/' || true)

if [[ -z "$STAGED_PILOT_FILES" ]]; then
    # No pilot data files staged — nothing to check
    exit 0
fi

echo "Pre-commit: scanning staged pilot data files for PII..."
echo ""

FAILURES=0
while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    if ! "$PII_SCANNER" --strict "$file" 2>/dev/null; then
        FAILURES=$((FAILURES + 1))
    fi
done <<< "$STAGED_PILOT_FILES"

if [[ "$FAILURES" -gt 0 ]]; then
    echo ""
    echo "BLOCKED: ${FAILURES} pilot data file(s) contain PII."
    echo "Run './scripts/pilot-anonymize.sh --customer <id>' to anonymize first."
    exit 1
fi

echo "Pre-commit: all pilot data files clean."
exit 0
