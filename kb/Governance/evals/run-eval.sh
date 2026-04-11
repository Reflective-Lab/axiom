#!/bin/bash
# Run a specific persona eval
# Usage: ./run-eval.sh <persona-name> [--claude] [--output <file>]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PERSONA=""
USE_CLAUDE=false
OUTPUT_FILE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --claude)
            USE_CLAUDE=true
            shift
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        *)
            PERSONA="$1"
            shift
            ;;
    esac
done

if [[ -z "$PERSONA" ]]; then
    echo "Usage: $0 <persona-name> [--claude] [--output <file>]"
    echo ""
    echo "Options:"
    echo "  --claude     Run eval directly in Claude Code (requires 'claude' CLI)"
    echo "  --output     Write report to specified file"
    echo ""
    echo "Available evals:"
    echo ""
    echo "  Core Technical:"
    echo "    system-architect    security-auditor    qa-engineer    sre-operations"
    echo ""
    echo "  Business & Legal:"
    echo "    legal-counsel    marketing-lead    sales-engineer    founder"
    echo ""
    echo "  Responsibility:"
    echo "    ethics-safety    sustainability    build-vs-buy    developer-advocate"
    echo ""
    echo "  External Perspective:"
    echo "    spiritual-advisor    curious-searcher    investor"
    echo "    end-user-advocate    skeptical-critic    external-perspective"
    echo ""
    exit 1
fi

EVAL_FILE="$SCRIPT_DIR/${PERSONA}-eval.md"

if [[ ! -f "$EVAL_FILE" ]]; then
    echo "Error: Eval file not found: $EVAL_FILE"
    echo "Run '$0' without arguments to see available evals."
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Converge Eval: $PERSONA"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Eval file: $EVAL_FILE"
echo ""

if $USE_CLAUDE; then
    # Check if claude CLI is available
    if ! command -v claude &> /dev/null; then
        echo "Error: 'claude' CLI not found. Install Claude Code first."
        echo "See: https://docs.anthropic.com/claude-code"
        exit 1
    fi

    echo "Running eval in Claude Code..."
    echo ""

    PROMPT="You are running the ${PERSONA} eval for Converge.

Read and follow the eval instructions below. Analyze the codebase and produce a report in the specified format.

---

$(cat "$EVAL_FILE")

---

Run this eval now against the Converge codebase. Produce a complete report."

    if [[ -n "$OUTPUT_FILE" ]]; then
        claude --print "$PROMPT" > "$OUTPUT_FILE"
        echo "✓ Report written to: $OUTPUT_FILE"
    else
        claude --print "$PROMPT"
    fi
else
    # Copy to clipboard if available
    if command -v pbcopy &> /dev/null; then
        cat "$EVAL_FILE" | pbcopy
        echo "✓ Eval copied to clipboard"
        echo ""
        echo "Paste into Claude with access to the Converge codebase."
    elif command -v xclip &> /dev/null; then
        cat "$EVAL_FILE" | xclip -selection clipboard
        echo "✓ Eval copied to clipboard"
        echo ""
        echo "Paste into Claude with access to the Converge codebase."
    else
        echo "Clipboard not available. Eval content:"
        echo ""
        echo "─────────────────────────────────────────────────────────────"
        cat "$EVAL_FILE"
        echo "─────────────────────────────────────────────────────────────"
    fi
fi
