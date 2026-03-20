#!/bin/bash
# Run all persona evals
# Usage: ./run-all.sh [converge-repo-path]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_PATH="${1:-../converge}"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║           Converge Persona Evals - Full Suite              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Target repo: $REPO_PATH"
echo "Evals directory: $SCRIPT_DIR"
echo ""

# All personas with evals
PERSONAS=(
    "system-architect"
    "security-auditor"
    "qa-engineer"
    "sre-operations"
    "legal-counsel"
    "marketing-lead"
    "sales-engineer"
    "founder"
    "developer-advocate"
    "ethics-safety"
    "sustainability"
    "build-vs-buy"
)

echo "Available evals:"
for persona in "${PERSONAS[@]}"; do
    eval_file="$SCRIPT_DIR/${persona}-eval.md"
    if [[ -f "$eval_file" ]]; then
        echo "  ✓ $persona"
    else
        echo "  ✗ $persona (not found)"
    fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "To run an eval, copy its content and paste into Claude with"
echo "access to the Converge codebase:"
echo ""
echo "  cat $SCRIPT_DIR/<persona>-eval.md | pbcopy"
echo ""
echo "Or use the run-eval.sh script:"
echo ""
echo "  $SCRIPT_DIR/run-eval.sh <persona>"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "RECOMMENDED PRE-RELEASE SUITE:"
echo ""
echo "Critical (must pass):"
echo "  ./run-eval.sh system-architect"
echo "  ./run-eval.sh security-auditor"
echo "  ./run-eval.sh qa-engineer"
echo "  ./run-eval.sh legal-counsel"
echo ""
echo "Important (should pass):"
echo "  ./run-eval.sh ethics-safety"
echo "  ./run-eval.sh sre-operations"
echo "  ./run-eval.sh developer-advocate"
echo ""
echo "Good practice (review):"
echo "  ./run-eval.sh build-vs-buy"
echo "  ./run-eval.sh sustainability"
echo "  ./run-eval.sh founder"
echo ""
