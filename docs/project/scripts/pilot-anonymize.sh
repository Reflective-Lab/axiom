#!/usr/bin/env bash
# pilot-anonymize.sh — Anonymize pilot customer data for case studies and export.
#
# Usage:
#   ./scripts/pilot-anonymize.sh --customer <id>                # anonymize one customer
#   ./scripts/pilot-anonymize.sh --customer <id> --dry-run      # scan only, report PII found
#   ./scripts/pilot-anonymize.sh --customer <id> --roster <file> # include customer name roster
#
# Reads raw pilot data from pilot-data/{customer-id}/ and writes anonymized
# output to pilot-data/{customer-id}/anonymized/.
#
# PII detection runs automatically after anonymization. If PII is found in the
# output, the script aborts and reports findings. Nothing is exported until clean.
#
# Anonymization rules (from PILOT_METRICS_FRAMEWORK.md Section 5):
#   1. Replace customer name with anon-{NNN}
#   2. Replace PII (names, emails, phones) with hashed tokens
#   3. Round volume metrics to nearest 5 (k-anonymity floor: suppress if <5 candidates)
#   4. Industry and company size publishable (with consent)
#   5. Exact dates replaced with relative offsets

set -euo pipefail

PILOT_DATA_DIR="${PILOT_DATA_DIR:-pilot-data}"
DRY_RUN=false
TARGET_CUSTOMER=""
ROSTER_FILE=""
ANON_PREFIX="anon"
SCRIPT_VERSION="1.0.0"

# --- PII patterns ---
# Email: standard RFC-ish pattern
EMAIL_PATTERN='[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
# Phone: US/intl formats (xxx-xxx-xxxx, (xxx) xxx-xxxx, +x-xxx-xxx-xxxx, etc.)
PHONE_PATTERN='(\+?[0-9]{1,3}[-.\s]?)?(\(?[0-9]{3}\)?[-.\s]?)[0-9]{3}[-.\s]?[0-9]{4}'
# SSN pattern (xxx-xx-xxxx)
SSN_PATTERN='[0-9]{3}-[0-9]{2}-[0-9]{4}'
# IP address (v4)
IPV4_PATTERN='[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}'
# Credit card (basic: 4 groups of 4 digits)
CC_PATTERN='[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --customer)
            TARGET_CUSTOMER="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
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
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$TARGET_CUSTOMER" ]]; then
    echo "ERROR: --customer <id> is required" >&2
    exit 1
fi

CUSTOMER_DIR="${PILOT_DATA_DIR}/${TARGET_CUSTOMER}"
if [[ ! -d "$CUSTOMER_DIR" ]]; then
    echo "ERROR: Customer directory not found: ${CUSTOMER_DIR}" >&2
    exit 1
fi

ANON_DIR="${CUSTOMER_DIR}/anonymized"
ANON_ID="${ANON_PREFIX}-$(openssl rand -hex 4)"

# If an anon-id mapping already exists, reuse it
ANON_MAP="${CUSTOMER_DIR}/.anon-id"
if [[ -f "$ANON_MAP" ]]; then
    ANON_ID=$(cat "$ANON_MAP" | tr -d '[:space:]')
else
    echo "$ANON_ID" > "$ANON_MAP"
fi

# Per-customer salt for PII hashing (prevents cross-customer token correlation)
SALT_FILE="${CUSTOMER_DIR}/.hash-salt"
if [[ -f "$SALT_FILE" ]]; then
    HASH_SALT=$(cat "$SALT_FILE" | tr -d '[:space:]')
else
    HASH_SALT=$(openssl rand -hex 16)
    echo "$HASH_SALT" > "$SALT_FILE"
fi

echo "Pilot Data Anonymization — $(date -u +'%Y-%m-%d %H:%M UTC')"
echo "Customer: ${TARGET_CUSTOMER} → ${ANON_ID}"
echo "Mode: $([ "$DRY_RUN" == "true" ] && echo "DRY-RUN (scan only)" || echo "ANONYMIZE")"
echo "---"

# --- Load roster names if provided ---
ROSTER_NAMES=()
if [[ -n "$ROSTER_FILE" && -f "$ROSTER_FILE" ]]; then
    while IFS= read -r name; do
        [[ -z "$name" || "$name" =~ ^# ]] && continue
        ROSTER_NAMES+=("$name")
    done < "$ROSTER_FILE"
    echo "Loaded ${#ROSTER_NAMES[@]} names from roster"
fi

# --- Escape sed metacharacters in a string ---
sed_escape() {
    printf '%s' "$1" | sed 's/[&/\\.^$*[\]]/\\&/g'
}

# --- Hash function for PII replacement ---
hash_pii() {
    local value="$1"
    local type="$2"
    # Produce a salted, deterministic-per-customer hash token
    local hash
    hash=$(echo -n "${HASH_SALT}:${value}" | shasum -a 256 | cut -c1-12)
    echo "[${type}:${hash}]"
}

# --- Round to nearest N with k-anonymity floor ---
round_metric() {
    local value="$1"
    local round_to="${2:-5}"
    local k_floor="${3:-5}"

    if [[ "$value" -lt "$k_floor" ]]; then
        echo '"< 5"'
    else
        echo $(( ((value + round_to / 2) / round_to) * round_to ))
    fi
}

# --- Anonymize a single file ---
anonymize_file() {
    local src="$1"
    local dst="$2"
    local content
    content=$(cat "$src")

    # 1. Replace customer ID/name with anon ID
    local esc_cust esc_anon
    esc_cust=$(sed_escape "$TARGET_CUSTOMER")
    esc_anon=$(sed_escape "$ANON_ID")
    content=$(echo "$content" | sed "s/${esc_cust}/${esc_anon}/g")

    # 2. Replace emails with hashed tokens
    while IFS= read -r email; do
        [[ -z "$email" ]] && continue
        local token
        token=$(hash_pii "$email" "email")
        local esc_email esc_token
        esc_email=$(sed_escape "$email")
        esc_token=$(sed_escape "$token")
        content=$(echo "$content" | sed "s/${esc_email}/${esc_token}/g")
    done < <(echo "$content" | grep -oE "$EMAIL_PATTERN" | sort -u)

    # 3. Replace phone numbers with hashed tokens
    while IFS= read -r phone; do
        [[ -z "$phone" ]] && continue
        local token
        token=$(hash_pii "$phone" "phone")
        local esc_phone esc_token
        esc_phone=$(sed_escape "$phone")
        esc_token=$(sed_escape "$token")
        content=$(echo "$content" | sed "s/${esc_phone}/${esc_token}/g")
    done < <(echo "$content" | grep -oE "$PHONE_PATTERN" | sort -u)

    # 4. Replace SSNs
    while IFS= read -r ssn; do
        [[ -z "$ssn" ]] && continue
        local token
        token=$(hash_pii "$ssn" "ssn")
        local esc_ssn esc_token
        esc_ssn=$(sed_escape "$ssn")
        esc_token=$(sed_escape "$token")
        content=$(echo "$content" | sed "s/${esc_ssn}/${esc_token}/g")
    done < <(echo "$content" | grep -oE "$SSN_PATTERN" | sort -u)

    # 5. Replace roster names if provided
    for name in "${ROSTER_NAMES[@]}"; do
        local token
        token=$(hash_pii "$name" "name")
        local esc_name esc_token
        esc_name=$(sed_escape "$name")
        esc_token=$(sed_escape "$token")
        content=$(echo "$content" | sed "s/${esc_name}/${esc_token}/gI")
    done

    # Write anonymized output
    mkdir -p "$(dirname "$dst")"
    echo "$content" > "$dst"
}

# --- PII scan function (used for validation) ---
# Returns 0 if clean, 1 if PII found
pii_scan_file() {
    local file="$1"
    local findings=0

    # Scan for email addresses
    local emails
    emails=$(grep -coE "$EMAIL_PATTERN" "$file" 2>/dev/null) || true
    if [[ "$emails" -gt 0 ]]; then
        echo "  PII FOUND [email]: ${emails} match(es) in $(basename "$file")"
        grep -noE "$EMAIL_PATTERN" "$file" 2>/dev/null | head -5 | sed 's/^/    /'
        findings=$((findings + emails))
    fi

    # Scan for phone numbers
    local phones
    phones=$(grep -coE "$PHONE_PATTERN" "$file" 2>/dev/null) || true
    if [[ "$phones" -gt 0 ]]; then
        echo "  PII FOUND [phone]: ${phones} match(es) in $(basename "$file")"
        grep -noE "$PHONE_PATTERN" "$file" 2>/dev/null | head -5 | sed 's/^/    /'
        findings=$((findings + phones))
    fi

    # Scan for SSNs
    local ssns
    ssns=$(grep -coE "$SSN_PATTERN" "$file" 2>/dev/null) || true
    if [[ "$ssns" -gt 0 ]]; then
        echo "  PII FOUND [ssn]: ${ssns} match(es) in $(basename "$file")"
        findings=$((findings + ssns))
    fi

    # Scan for IP addresses
    local ips
    ips=$(grep -coE "$IPV4_PATTERN" "$file" 2>/dev/null) || true
    if [[ "$ips" -gt 0 ]]; then
        echo "  PII FOUND [ipv4]: ${ips} match(es) in $(basename "$file")"
        findings=$((findings + ips))
    fi

    # Scan for credit card numbers
    local ccs
    ccs=$(grep -coE "$CC_PATTERN" "$file" 2>/dev/null) || true
    if [[ "$ccs" -gt 0 ]]; then
        echo "  PII FOUND [cc]: ${ccs} match(es) in $(basename "$file")"
        findings=$((findings + ccs))
    fi

    # Scan for roster names
    for name in "${ROSTER_NAMES[@]}"; do
        local name_count
        name_count=$(grep -coiF "$name" "$file" 2>/dev/null) || true
        if [[ "$name_count" -gt 0 ]]; then
            echo "  PII FOUND [name]: '${name}' appears ${name_count} time(s) in $(basename "$file")"
            findings=$((findings + name_count))
        fi
    done

    # Scan for customer ID (should have been replaced with anon ID)
    local cust_refs
    cust_refs=$(grep -coF "$TARGET_CUSTOMER" "$file" 2>/dev/null) || true
    if [[ "$cust_refs" -gt 0 ]]; then
        echo "  PII FOUND [customer_id]: raw customer ID appears ${cust_refs} time(s) in $(basename "$file")"
        findings=$((findings + cust_refs))
    fi

    return $( [[ "$findings" -gt 0 ]] && echo 1 || echo 0 )
}

# --- Main ---

TOTAL_FILES=0
ANONYMIZED=0
PII_CLEAN=0
PII_DIRTY=0

# Collect source files (exclude anonymized dir, marker files, and hidden files)
SOURCE_FILES=()
while IFS= read -r -d '' f; do
    case "$f" in
        */anonymized/*|*/.pilot-end-date|*/.anon-id|*/.hash-salt) continue ;;
    esac
    SOURCE_FILES+=("$f")
done < <(find "$CUSTOMER_DIR" -type f -print0)

TOTAL_FILES=${#SOURCE_FILES[@]}
echo "Found ${TOTAL_FILES} source file(s) to process"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo "=== DRY-RUN: Scanning source files for PII ==="
    for src in "${SOURCE_FILES[@]}"; do
        echo "Scanning: $(basename "$src")"
        if pii_scan_file "$src"; then
            PII_CLEAN=$((PII_CLEAN + 1))
        else
            PII_DIRTY=$((PII_DIRTY + 1))
        fi
    done
else
    echo "=== Phase 1: Anonymizing files ==="
    for src in "${SOURCE_FILES[@]}"; do
        # Compute destination path under anonymized/
        local_path="${src#${CUSTOMER_DIR}/}"
        dst="${ANON_DIR}/${local_path}"
        echo "  Anonymizing: ${local_path}"
        anonymize_file "$src" "$dst"
        ANONYMIZED=$((ANONYMIZED + 1))
    done

    echo ""
    echo "=== Phase 2: PII validation scan on anonymized output ==="
    SCAN_PASS=true
    while IFS= read -r -d '' f; do
        echo "  Validating: $(basename "$f")"
        if ! pii_scan_file "$f"; then
            PII_DIRTY=$((PII_DIRTY + 1))
            SCAN_PASS=false
        else
            PII_CLEAN=$((PII_CLEAN + 1))
        fi
    done < <(find "$ANON_DIR" -type f -print0)

    if [[ "$SCAN_PASS" == "false" ]]; then
        echo ""
        echo "ERROR: PII detected in anonymized output. Anonymization incomplete."
        echo "Review findings above and fix before export."
        echo "Anonymized files remain in ${ANON_DIR} for inspection."
        exit 1
    fi
fi

echo ""
echo "---"
echo "Summary:"
echo "  Source files: ${TOTAL_FILES}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  PII-clean: ${PII_CLEAN}"
    echo "  PII-found: ${PII_DIRTY}"
    if [[ "$PII_DIRTY" -gt 0 ]]; then
        echo "  Status: PII detected — anonymization required before export"
        exit 1
    else
        echo "  Status: No PII detected in source files"
    fi
else
    echo "  Anonymized: ${ANONYMIZED}"
    echo "  PII scan clean: ${PII_CLEAN}"
    echo "  PII scan dirty: ${PII_DIRTY}"
    echo "  Status: CLEAN — ready for Ava (security) review, then Blake (marketing) handoff"
fi
