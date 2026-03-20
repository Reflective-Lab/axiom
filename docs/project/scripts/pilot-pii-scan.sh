#!/usr/bin/env bash
# pilot-pii-scan.sh — Scan files for PII before export or commit.
#
# Usage:
#   ./scripts/pilot-pii-scan.sh <file-or-directory>       # scan and report
#   ./scripts/pilot-pii-scan.sh --roster <file> <path>     # also check customer names
#   ./scripts/pilot-pii-scan.sh --strict <path>            # exit 1 on any finding
#
# Exit codes:
#   0 — no PII detected
#   1 — PII detected (--strict mode) or error
#
# Designed to run as a CI gate or pre-commit hook for pilot data files.

set -euo pipefail

STRICT=false
ROSTER_FILE=""
SCAN_TARGET=""
SCRIPT_VERSION="1.0.0"

# --- PII patterns ---
EMAIL_PATTERN='[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
PHONE_PATTERN='(\+?[0-9]{1,3}[-.\s]?)?(\(?[0-9]{3}\)?[-.\s]?)[0-9]{3}[-.\s]?[0-9]{4}'
SSN_PATTERN='[0-9]{3}-[0-9]{2}-[0-9]{4}'
IPV4_PATTERN='[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}'
CC_PATTERN='[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}'

# Known safe patterns to exclude (hashed tokens from anonymizer, metric labels, etc.)
SAFE_EMAIL_PATTERN='\[email:[a-f0-9]+\]'
SAFE_PHONE_PATTERN='\[phone:[a-f0-9]+\]'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --strict)
            STRICT=true
            shift
            ;;
        --roster)
            ROSTER_FILE="$2"
            shift 2
            ;;
        --help|-h)
            head -14 "$0" | tail -12
            exit 0
            ;;
        *)
            if [[ -z "$SCAN_TARGET" ]]; then
                SCAN_TARGET="$1"
            else
                echo "ERROR: unexpected argument: $1" >&2
                exit 1
            fi
            shift
            ;;
    esac
done

if [[ -z "$SCAN_TARGET" ]]; then
    echo "ERROR: provide a file or directory to scan" >&2
    echo "Usage: $0 [--strict] [--roster <file>] <file-or-directory>" >&2
    exit 1
fi

if [[ ! -e "$SCAN_TARGET" ]]; then
    echo "ERROR: target not found: ${SCAN_TARGET}" >&2
    exit 1
fi

# Load roster names
ROSTER_NAMES=()
if [[ -n "$ROSTER_FILE" && -f "$ROSTER_FILE" ]]; then
    while IFS= read -r name; do
        [[ -z "$name" || "$name" =~ ^# ]] && continue
        ROSTER_NAMES+=("$name")
    done < "$ROSTER_FILE"
fi

echo "PII Scan — $(date -u +'%Y-%m-%d %H:%M UTC')"
echo "Target: ${SCAN_TARGET}"
echo "Mode: $([ "$STRICT" == "true" ] && echo "STRICT (fail on findings)" || echo "REPORT")"
[[ ${#ROSTER_NAMES[@]} -gt 0 ]] && echo "Roster: ${#ROSTER_NAMES[@]} names loaded"
echo "---"

TOTAL_FILES=0
TOTAL_FINDINGS=0
DIRTY_FILES=0

scan_file() {
    local file="$1"
    local file_findings=0

    # Skip binary files
    if file "$file" | grep -q "binary"; then
        return 0
    fi

    # Skip already-anonymized token references (don't flag [email:abc123] as PII)
    local content
    content=$(cat "$file")

    # Email scan (exclude hashed tokens)
    local raw_emails
    raw_emails=$(echo "$content" | grep -oE "$EMAIL_PATTERN" 2>/dev/null | grep -vE '\[email:[a-f0-9]+\]' | sort -u || true)
    if [[ -n "$raw_emails" ]]; then
        local count
        count=$(echo "$raw_emails" | wc -l | tr -d ' ')
        echo "  [email] ${count} address(es) in $(basename "$file"):"
        echo "$raw_emails" | head -3 | sed 's/^/    /'
        [[ "$count" -gt 3 ]] && echo "    ... and $((count - 3)) more"
        file_findings=$((file_findings + count))
    fi

    # Phone scan
    local raw_phones
    raw_phones=$(echo "$content" | grep -oE "$PHONE_PATTERN" 2>/dev/null | sort -u || true)
    if [[ -n "$raw_phones" ]]; then
        local count
        count=$(echo "$raw_phones" | wc -l | tr -d ' ')
        echo "  [phone] ${count} number(s) in $(basename "$file"):"
        echo "$raw_phones" | head -3 | sed 's/^/    /'
        file_findings=$((file_findings + count))
    fi

    # SSN scan
    local raw_ssns
    raw_ssns=$(echo "$content" | grep -oE "$SSN_PATTERN" 2>/dev/null | sort -u || true)
    if [[ -n "$raw_ssns" ]]; then
        local count
        count=$(echo "$raw_ssns" | wc -l | tr -d ' ')
        echo "  [ssn] ${count} SSN(s) in $(basename "$file")"
        file_findings=$((file_findings + count))
    fi

    # IPv4 scan (skip 0.0.0.0, 127.0.0.1, and version-like strings)
    local raw_ips
    raw_ips=$(echo "$content" | grep -oE "$IPV4_PATTERN" 2>/dev/null | grep -vE '^(0\.0\.0\.0|127\.0\.0\.1|255\.|10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.)' | sort -u || true)
    if [[ -n "$raw_ips" ]]; then
        local count
        count=$(echo "$raw_ips" | wc -l | tr -d ' ')
        echo "  [ipv4] ${count} public IP(s) in $(basename "$file"):"
        echo "$raw_ips" | head -3 | sed 's/^/    /'
        file_findings=$((file_findings + count))
    fi

    # Credit card scan
    local raw_ccs
    raw_ccs=$(echo "$content" | grep -oE "$CC_PATTERN" 2>/dev/null | sort -u || true)
    if [[ -n "$raw_ccs" ]]; then
        local count
        count=$(echo "$raw_ccs" | wc -l | tr -d ' ')
        echo "  [cc] ${count} card number(s) in $(basename "$file")"
        file_findings=$((file_findings + count))
    fi

    # Roster name scan
    for name in "${ROSTER_NAMES[@]}"; do
        local name_count
        name_count=$(echo "$content" | grep -coiF "$name" 2>/dev/null || echo 0)
        if [[ "$name_count" -gt 0 ]]; then
            echo "  [name] '${name}' appears ${name_count} time(s) in $(basename "$file")"
            file_findings=$((file_findings + name_count))
        fi
    done

    if [[ "$file_findings" -gt 0 ]]; then
        DIRTY_FILES=$((DIRTY_FILES + 1))
    fi
    TOTAL_FINDINGS=$((TOTAL_FINDINGS + file_findings))
}

# Collect files to scan
if [[ -f "$SCAN_TARGET" ]]; then
    echo "Scanning 1 file..."
    echo ""
    TOTAL_FILES=1
    scan_file "$SCAN_TARGET"
elif [[ -d "$SCAN_TARGET" ]]; then
    FILES=()
    while IFS= read -r -d '' f; do
        FILES+=("$f")
    done < <(find "$SCAN_TARGET" -type f -print0)
    TOTAL_FILES=${#FILES[@]}
    echo "Scanning ${TOTAL_FILES} file(s)..."
    echo ""
    for f in "${FILES[@]}"; do
        scan_file "$f"
    done
fi

echo ""
echo "---"
echo "Results:"
echo "  Files scanned: ${TOTAL_FILES}"
echo "  Files with PII: ${DIRTY_FILES}"
echo "  Total findings: ${TOTAL_FINDINGS}"

if [[ "$TOTAL_FINDINGS" -gt 0 ]]; then
    echo "  Verdict: PII DETECTED"
    if [[ "$STRICT" == "true" ]]; then
        echo ""
        echo "FAIL: PII found in ${DIRTY_FILES} file(s). Clean before export."
        exit 1
    fi
else
    echo "  Verdict: CLEAN — no PII detected"
fi
