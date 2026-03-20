#!/usr/bin/env bash
# test-pilot-data-dispose.sh — Tests for pilot-data-dispose.sh security fixes (REF-28)
#
# Tests:
#   1. Log injection via malicious directory name is neutralized
#   2. --force requires PILOT_FORCE_AUTHORIZED_BY
#   3. Audit log entries include integrity hash chain

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DISPOSE_SCRIPT="${SCRIPT_DIR}/pilot-data-dispose.sh"
TEST_DIR=$(mktemp -d)
PASS=0
FAIL=0

cleanup() {
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

assert_eq() {
    local desc="$1" expected="$2" actual="$3"
    if [[ "$expected" == "$actual" ]]; then
        echo "  PASS: ${desc}"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: ${desc}"
        echo "    expected: ${expected}"
        echo "    actual:   ${actual}"
        FAIL=$((FAIL + 1))
    fi
}

assert_contains() {
    local desc="$1" needle="$2" haystack="$3"
    if echo "$haystack" | grep -qF "$needle"; then
        echo "  PASS: ${desc}"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: ${desc}"
        echo "    expected to contain: ${needle}"
        echo "    in: ${haystack}"
        FAIL=$((FAIL + 1))
    fi
}

assert_not_contains() {
    local desc="$1" needle="$2" haystack="$3"
    if echo "$haystack" | grep -qF "$needle"; then
        echo "  FAIL: ${desc}"
        echo "    should NOT contain: ${needle}"
        echo "    in: ${haystack}"
        FAIL=$((FAIL + 1))
    else
        echo "  PASS: ${desc}"
        PASS=$((PASS + 1))
    fi
}

# ─── Setup helper ───────────────────────────────────────────────
setup_customer() {
    local base_dir="$1"
    local customer_name="$2"
    local days_ago="${3:-100}"

    local cust_dir="${base_dir}/${customer_name}"
    mkdir -p "${cust_dir}/anonymized"
    echo "anon data" > "${cust_dir}/anonymized/data.csv"
    echo "raw PII" > "${cust_dir}/raw-data.csv"

    # Set pilot end date to N days ago
    local end_date
    end_date=$(date -v-${days_ago}d +"%Y-%m-%d" 2>/dev/null || date -d "${days_ago} days ago" +"%Y-%m-%d" 2>/dev/null)
    echo "$end_date" > "${cust_dir}/.pilot-end-date"
}

echo "=== pilot-data-dispose.sh Security Tests ==="
echo ""

# ─── Test 1: Log injection neutralized ──────────────────────────
echo "Test 1: Log injection via malicious directory name"

INJECT_DIR="${TEST_DIR}/test1"
mkdir -p "$INJECT_DIR"

# Create a directory with JSON injection characters in the name
MALICIOUS_NAME='foo","event":"injected","x":"y'
setup_customer "$INJECT_DIR" "$MALICIOUS_NAME"

PILOT_DATA_DIR="$INJECT_DIR" \
PILOT_DISPOSE_CONFIRM=1 \
RETENTION_DAYS=90 \
    bash "$DISPOSE_SCRIPT" > /dev/null 2>&1 || true

AUDIT="${INJECT_DIR}/disposal-audit.log"
if [[ -f "$AUDIT" ]]; then
    audit_line=$(cat "$AUDIT")
    # The sanitized customer_id should only contain alphanumeric, underscore, hyphen
    assert_not_contains "Injected JSON event field not present" '"event":"injected"' "$audit_line"
    assert_contains "Sanitized customer_id present" '"customer_anon_id":"fooeventinjectedxy"' "$audit_line"
    assert_contains "Original event type preserved" '"event":"pilot_data_disposal"' "$audit_line"
else
    echo "  FAIL: Audit log not created"
    FAIL=$((FAIL + 1))
fi

echo ""

# ─── Test 1b: Operator field JSON injection neutralized ─────────
echo "Test 1b: Operator field injection via PILOT_FORCE_AUTHORIZED_BY"

INJECT2_DIR="${TEST_DIR}/test1b"
mkdir -p "$INJECT2_DIR"
setup_customer "$INJECT2_DIR" "safe-customer" 30

PILOT_DATA_DIR="$INJECT2_DIR" \
PILOT_FORCE_AUTHORIZED_BY='evil","event":"injected","admin":"true' \
PILOT_DISPOSE_CONFIRM=1 \
RETENTION_DAYS=90 \
    bash "$DISPOSE_SCRIPT" --customer safe-customer --force > /dev/null 2>&1 || true

AUDIT1B="${INJECT2_DIR}/disposal-audit.log"
if [[ -f "$AUDIT1B" ]]; then
    audit_1b=$(cat "$AUDIT1B")
    assert_not_contains "Operator injection blocked" '"admin":"true"' "$audit_1b"
    assert_not_contains "Operator event injection blocked" '"event":"injected"' "$audit_1b"
    assert_contains "Operator field sanitized" '"operator":"customer_request:evil' "$audit_1b"
    # Verify the audit entry is valid JSON
    if echo "$audit_1b" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
        echo "  PASS: Audit entry is valid JSON"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: Audit entry is not valid JSON"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  FAIL: Audit log not created"
    FAIL=$((FAIL + 1))
fi

echo ""

# ─── Test 2: --force requires authorization ─────────────────────
echo "Test 2: --force requires PILOT_FORCE_AUTHORIZED_BY"

FORCE_DIR="${TEST_DIR}/test2"
mkdir -p "$FORCE_DIR"
setup_customer "$FORCE_DIR" "test-customer" 30  # Only 30 days, needs --force

# 2a: --force WITHOUT PILOT_FORCE_AUTHORIZED_BY should fail
output=$(PILOT_DATA_DIR="$FORCE_DIR" \
    bash "$DISPOSE_SCRIPT" --customer test-customer --force 2>&1 || true)
assert_contains "--force without auth var fails" "PILOT_FORCE_AUTHORIZED_BY" "$output"

# 2b: --force WITH PILOT_FORCE_AUTHORIZED_BY should succeed
output=$(PILOT_DATA_DIR="$FORCE_DIR" \
PILOT_FORCE_AUTHORIZED_BY="gdpr-handler-jane" \
PILOT_DISPOSE_CONFIRM=1 \
    bash "$DISPOSE_SCRIPT" --customer test-customer --force 2>&1 || true)
assert_contains "--force with auth var disposes" "DISPOSED test-customer" "$output"

# 2c: Verify operator identity is logged
AUDIT2="${FORCE_DIR}/disposal-audit.log"
if [[ -f "$AUDIT2" ]]; then
    audit_line2=$(cat "$AUDIT2")
    assert_contains "Operator identity logged" "customer_request:gdpr-handler-jane" "$audit_line2"
else
    echo "  FAIL: Audit log not created for --force disposal"
    FAIL=$((FAIL + 1))
fi

echo ""

# ─── Test 3: Integrity hash chain ──────────────────────────────
echo "Test 3: Audit log integrity hash chain"

HASH_DIR="${TEST_DIR}/test3"
mkdir -p "$HASH_DIR"
setup_customer "$HASH_DIR" "customer-a" 100
setup_customer "$HASH_DIR" "customer-b" 100

PILOT_DATA_DIR="$HASH_DIR" \
PILOT_DISPOSE_CONFIRM=1 \
RETENTION_DAYS=90 \
    bash "$DISPOSE_SCRIPT" > /dev/null 2>&1 || true

AUDIT3="${HASH_DIR}/disposal-audit.log"
if [[ -f "$AUDIT3" ]]; then
    line_count=$(wc -l < "$AUDIT3" | tr -d ' ')
    assert_eq "Two audit entries created" "2" "$line_count"

    # First entry's prev_hash should reference "genesis" (SHA of genesis line doesn't exist)
    first_entry=$(head -1 "$AUDIT3")
    assert_contains "First entry has prev_hash" '"prev_hash":' "$first_entry"
    assert_contains "First entry has integrity_hash" '"integrity_hash":' "$first_entry"

    # Second entry's prev_hash should be SHA-256 of the first line
    # Note: tail -1 in the script produces output with a trailing newline, so echo (not echo -n)
    second_entry=$(tail -1 "$AUDIT3")
    expected_prev=$(echo "$first_entry" | shasum -a 256 | cut -c1-64)
    assert_contains "Second entry chains to first" "\"prev_hash\":\"${expected_prev}\"" "$second_entry"
    assert_contains "Second entry has integrity_hash" '"integrity_hash":' "$second_entry"

    # Verify integrity_hash is correct for second entry
    # Extract the entry without the integrity_hash suffix to recompute
    entry_body=$(echo "$second_entry" | sed 's/,"integrity_hash":"[a-f0-9]*"}$/}/')
    recomputed=$(echo -n "$entry_body" | shasum -a 256 | cut -c1-64)
    actual_hash=$(echo "$second_entry" | grep -o '"integrity_hash":"[a-f0-9]*"' | cut -d'"' -f4)
    assert_eq "Integrity hash is valid SHA-256 of entry body" "$recomputed" "$actual_hash"
else
    echo "  FAIL: Audit log not created"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Results: ${PASS} passed, ${FAIL} failed ==="

if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
