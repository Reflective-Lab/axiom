#!/bin/bash
# Run an eval suite for a specific gate
# Usage: ./run-suite.sh <suite> [--claude] [--output-dir <dir>]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SUITE=""
USE_CLAUDE=false
OUTPUT_DIR=""
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --claude)
            USE_CLAUDE=true
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            SUITE="$1"
            shift
            ;;
    esac
done

# Get evals for suite (bash 3.2 compatible)
get_suite_evals() {
    case "$1" in
        pr-merge)
            echo "system-architect qa-engineer"
            ;;
        release-candidate)
            echo "system-architect security-auditor qa-engineer sre-operations"
            ;;
        release-critical)
            echo "system-architect security-auditor qa-engineer legal-counsel"
            ;;
        release-full)
            echo "system-architect security-auditor qa-engineer legal-counsel ethics-safety sre-operations developer-advocate marketing-lead sales-engineer build-vs-buy sustainability investor curious-searcher skeptical-critic future-historian regulator-lens journalist-investigator academic-researcher insurance-underwriter"
            ;;
        technical)
            echo "system-architect security-auditor qa-engineer sre-operations"
            ;;
        business)
            echo "legal-counsel marketing-lead sales-engineer founder"
            ;;
        responsibility)
            echo "ethics-safety sustainability build-vs-buy developer-advocate"
            ;;
        external)
            echo "spiritual-advisor curious-searcher investor end-user-advocate skeptical-critic future-historian regulator-lens journalist-investigator academic-researcher insurance-underwriter"
            ;;
        marketing)
            echo "marketing-lead legal-counsel ethics-safety curious-searcher"
            ;;
        sales)
            echo "sales-engineer legal-counsel ethics-safety"
            ;;
        deploy)
            echo "sre-operations security-auditor qa-engineer"
            ;;
        weekly)
            echo "security-auditor sre-operations"
            ;;
        monthly)
            echo "system-architect security-auditor qa-engineer sre-operations legal-counsel ethics-safety build-vs-buy"
            ;;
        *)
            echo ""
            ;;
    esac
}

if [[ -z "$SUITE" ]]; then
    echo "Usage: $0 <suite> [--claude] [--output-dir <dir>]"
    echo ""
    echo "Options:"
    echo "  --claude         Run evals directly in Claude Code"
    echo "  --output-dir     Write reports to specified directory"
    echo ""
    echo "Available suites:"
    echo ""
    echo "  Gate Suites:"
    echo "    pr-merge           Required for PR merge (architect, qa)"
    echo "    release-candidate  Required for RC tag (technical suite)"
    echo "    release-critical   Critical for release approval"
    echo "    release-full       Full release approval suite"
    echo "    marketing          Marketing content publish gate"
    echo "    sales              Sales commitment gate"
    echo "    deploy             Production deploy gate"
    echo ""
    echo "  Domain Suites:"
    echo "    technical          All technical evals"
    echo "    business           All business evals"
    echo "    responsibility     All responsibility evals"
    echo "    external           All external perspective evals"
    echo ""
    echo "  Scheduled Suites:"
    echo "    weekly             Weekly security/ops check"
    echo "    monthly            Monthly comprehensive check"
    echo ""
    exit 1
fi

EVALS=$(get_suite_evals "$SUITE")

if [[ -z "$EVALS" ]]; then
    echo "Error: Unknown suite '$SUITE'"
    echo "Run '$0' without arguments to see available suites."
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Converge Eval Suite: $SUITE"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Evals to run:"
for eval in $EVALS; do
    echo "  - $eval"
done
echo ""

# Set up output directory
if [[ -n "$OUTPUT_DIR" ]]; then
    mkdir -p "$OUTPUT_DIR"
    echo "Reports will be written to: $OUTPUT_DIR"
    echo ""
fi

# Track results
PASSED=0
PARTIAL=0
FAILED=0
TOTAL=0

# Run each eval
for eval in $EVALS; do
    TOTAL=$((TOTAL + 1))
    echo "────────────────────────────────────────────────────────────────"
    echo "Running: $eval"
    echo "────────────────────────────────────────────────────────────────"

    if $USE_CLAUDE; then
        if [[ -n "$OUTPUT_DIR" ]]; then
            OUTPUT_FILE="$OUTPUT_DIR/${eval}-${TIMESTAMP}.md"
            "$SCRIPT_DIR/run-eval.sh" "$eval" --claude --output "$OUTPUT_FILE"

            # Try to extract status from report
            if grep -q "Status.*PASS" "$OUTPUT_FILE" 2>/dev/null; then
                PASSED=$((PASSED + 1))
                echo "  → PASS"
            elif grep -q "Status.*FAIL" "$OUTPUT_FILE" 2>/dev/null; then
                FAILED=$((FAILED + 1))
                echo "  → FAIL"
            else
                PARTIAL=$((PARTIAL + 1))
                echo "  → PARTIAL (or status unclear)"
            fi
        else
            "$SCRIPT_DIR/run-eval.sh" "$eval" --claude
        fi
    else
        # Manual mode - just show instructions
        echo ""
        echo "To run this eval manually:"
        echo "  $SCRIPT_DIR/run-eval.sh $eval"
        echo ""
    fi

    echo ""
done

# Summary
echo "════════════════════════════════════════════════════════════════"
echo "Suite Summary: $SUITE"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Total evals: $TOTAL"

if $USE_CLAUDE && [[ -n "$OUTPUT_DIR" ]]; then
    echo "Passed:      $PASSED"
    echo "Partial:     $PARTIAL"
    echo "Failed:      $FAILED"
    echo ""

    if [[ $FAILED -gt 0 ]]; then
        echo "⛔ SUITE RESULT: FAIL"
        echo ""
        echo "Failed evals block this gate. Review reports in $OUTPUT_DIR"
        exit 1
    elif [[ $PARTIAL -gt 0 ]]; then
        echo "⚠️  SUITE RESULT: PARTIAL"
        echo ""
        echo "Some evals need review. Check reports in $OUTPUT_DIR"
        exit 0
    else
        echo "✅ SUITE RESULT: PASS"
        echo ""
        echo "All evals passed. Gate may proceed."
        exit 0
    fi
else
    echo ""
    echo "Run with --claude --output-dir <dir> for automated execution."
fi
