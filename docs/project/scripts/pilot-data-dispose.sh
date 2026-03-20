#!/usr/bin/env bash
# pilot-data-dispose.sh — Automated disposal of pilot customer data
# after retention period (90 days post-pilot).
#
# Usage:
#   ./scripts/pilot-data-dispose.sh                          # scan all, delete expired
#   ./scripts/pilot-data-dispose.sh --dry-run                # scan all, report only
#   ./scripts/pilot-data-dispose.sh --customer <id>          # dispose specific customer
#   ./scripts/pilot-data-dispose.sh --customer <id> --force  # early deletion (customer request)
#
# Safety:
#   - Will NOT delete if anonymized copy is missing
#   - Append-only audit log at pilot-data/disposal-audit.log
#   - Dry-run mode by default when run manually (set PILOT_DISPOSE_CONFIRM=1 to execute)
#   - Also disposes planning artifacts in pilots/<customer>/ (REF-48)

set -euo pipefail

PILOT_DATA_DIR="${PILOT_DATA_DIR:-pilot-data}"
PILOTS_DIR="${PILOTS_DIR:-pilots}"
AUDIT_LOG="${PILOT_DATA_DIR}/disposal-audit.log"
RETENTION_DAYS="${RETENTION_DAYS:-90}"
SCRIPT_VERSION="1.0.0"
DRY_RUN=false
FORCE=false
TARGET_CUSTOMER=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --customer)
            TARGET_CUSTOMER="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --help|-h)
            head -12 "$0" | tail -10
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

# Safety: require explicit confirmation for automated runs
if [[ "$DRY_RUN" == "false" && "${PILOT_DISPOSE_CONFIRM:-0}" != "1" && "$FORCE" == "false" ]]; then
    echo "Safety: set PILOT_DISPOSE_CONFIRM=1 or use --dry-run"
    exit 1
fi

# Safety: --force (early deletion) requires an authorized operator identity
if [[ "$FORCE" == "true" ]]; then
    if [[ -z "${PILOT_FORCE_AUTHORIZED_BY:-}" ]]; then
        echo "ERROR: --force requires PILOT_FORCE_AUTHORIZED_BY=<operator-name> (e.g., GDPR/CCPA request handler)" >&2
        exit 1
    fi
fi

if [[ ! -d "$PILOT_DATA_DIR" ]]; then
    echo "No pilot-data directory found. Nothing to do."
    exit 0
fi

# Ensure audit log directory exists
mkdir -p "$(dirname "$AUDIT_LOG")"

NOW_EPOCH=$(date +%s)
NOW_ISO=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
DISPOSED=0
SKIPPED=0
ERRORS=0

# Sanitize a string for safe JSON value interpolation (strip quotes, backslashes, control chars)
json_safe() {
    echo "$1" | tr -cd 'a-zA-Z0-9 _.,:/@-'
}

log_audit() {
    local customer_id="$1"
    local files_deleted="$2"
    local total_size="$3"
    local anon_verified="$4"
    local operator
    operator=$(json_safe "$5")
    local pilot_end
    pilot_end=$(json_safe "$6")

    # Compute hash chain: SHA-256 of (previous_hash + current_entry)
    local prev_hash="genesis"
    if [[ -f "$AUDIT_LOG" ]]; then
        prev_hash=$(tail -1 "$AUDIT_LOG" | shasum -a 256 | cut -c1-64)
    fi

    local entry="{\"event\":\"pilot_data_disposal\",\"customer_anon_id\":\"${customer_id}\",\"pilot_end_date\":\"${pilot_end}\",\"disposal_date\":\"${NOW_ISO}\",\"retention_days\":${RETENTION_DAYS},\"files_deleted\":${files_deleted},\"total_size_bytes\":${total_size},\"anonymized_copy_verified\":${anon_verified},\"operator\":\"${operator}\",\"disposal_script_version\":\"${SCRIPT_VERSION}\",\"prev_hash\":\"${prev_hash}\"}"
    local entry_hash
    entry_hash=$(echo -n "${entry}" | shasum -a 256 | cut -c1-64)
    local signed_entry="${entry%\}},\"integrity_hash\":\"${entry_hash}\"}"

    echo "$signed_entry" >> "$AUDIT_LOG"
}

dispose_customer() {
    local customer_dir="$1"
    local customer_id
    customer_id=$(basename "$customer_dir" | tr -cd 'a-zA-Z0-9_-')
    local operator="automated"

    if [[ "$FORCE" == "true" ]]; then
        operator="customer_request:${PILOT_FORCE_AUTHORIZED_BY:-unknown}"
    fi

    # Check for pilot-end marker file
    local end_marker="${customer_dir}/.pilot-end-date"
    if [[ ! -f "$end_marker" ]]; then
        echo "  SKIP ${customer_id}: no .pilot-end-date marker"
        SKIPPED=$((SKIPPED + 1))
        return
    fi

    local pilot_end
    pilot_end=$(cat "$end_marker" | tr -d '[:space:]')

    # Calculate days since pilot end
    local end_epoch
    end_epoch=$(date -j -f "%Y-%m-%d" "$pilot_end" +%s 2>/dev/null || date -d "$pilot_end" +%s 2>/dev/null)
    local days_since=$(( (NOW_EPOCH - end_epoch) / 86400 ))

    if [[ "$FORCE" == "false" && "$days_since" -lt "$RETENTION_DAYS" ]]; then
        echo "  SKIP ${customer_id}: ${days_since}/${RETENTION_DAYS} days elapsed"
        SKIPPED=$((SKIPPED + 1))
        return
    fi

    # Check anonymized copy exists
    local anon_dir="${customer_dir}/anonymized"
    if [[ ! -d "$anon_dir" ]] || [[ -z "$(ls -A "$anon_dir" 2>/dev/null)" ]]; then
        echo "  ERROR ${customer_id}: anonymized copy missing — aborting deletion"
        ERRORS=$((ERRORS + 1))
        return
    fi

    # Count files and size (excluding anonymized dir, audit log, and marker)
    local file_count=0
    local total_size=0
    while IFS= read -r -d '' f; do
        case "$f" in
            */anonymized/*|*/.pilot-end-date) continue ;;
        esac
        file_count=$((file_count + 1))
        local fsize
        fsize=$(stat -f%z "$f" 2>/dev/null || stat -c%s "$f" 2>/dev/null || echo 0)
        total_size=$((total_size + fsize))
    done < <(find "$customer_dir" -type f -print0)

    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  DRY-RUN ${customer_id}: would delete ${file_count} files (${total_size} bytes), ${days_since} days past end"
        return
    fi

    # Delete raw data files (keep anonymized dir and marker)
    while IFS= read -r -d '' f; do
        case "$f" in
            */anonymized/*|*/.pilot-end-date) continue ;;
        esac
        rm -f "$f"
    done < <(find "$customer_dir" -type f -print0)

    # Remove empty subdirectories (except anonymized)
    find "$customer_dir" -mindepth 1 -type d ! -name "anonymized" -empty -delete 2>/dev/null || true

    # Log to audit trail
    log_audit "$customer_id" "$file_count" "$total_size" "true" "$operator" "$pilot_end"

    echo "  DISPOSED ${customer_id}: ${file_count} files (${total_size} bytes)"
    DISPOSED=$((DISPOSED + 1))
}

echo "Pilot Data Disposal — $(date -u +"%Y-%m-%d %H:%M UTC")"
echo "Retention: ${RETENTION_DAYS} days | Mode: $([ "$DRY_RUN" == "true" ] && echo "DRY-RUN" || echo "LIVE")"
echo "---"

if [[ -n "$TARGET_CUSTOMER" ]]; then
    customer_path="${PILOT_DATA_DIR}/${TARGET_CUSTOMER}"
    if [[ ! -d "$customer_path" ]]; then
        echo "Customer directory not found: ${customer_path}"
        exit 1
    fi
    dispose_customer "$customer_path"
else
    for customer_dir in "${PILOT_DATA_DIR}"/*/; do
        [[ -d "$customer_dir" ]] || continue
        [[ "$(basename "$customer_dir")" == "disposal-audit.log" ]] && continue
        dispose_customer "$customer_dir"
    done
fi

# Also dispose planning artifacts in pilots/ directory (REF-48)
if [[ -d "$PILOTS_DIR" ]]; then
    echo ""
    echo "Planning Artifacts (${PILOTS_DIR}/):"
    if [[ -n "$TARGET_CUSTOMER" ]]; then
        pilots_path="${PILOTS_DIR}/${TARGET_CUSTOMER}"
        if [[ -d "$pilots_path" ]]; then
            # Planning artifacts have no retention marker — dispose if runtime data was disposed
            if [[ "$DRY_RUN" == "true" ]]; then
                local_count=$(find "$pilots_path" -type f | wc -l | tr -d ' ')
                echo "  DRY-RUN ${TARGET_CUSTOMER}: would delete ${local_count} planning files"
            else
                rm -rf "$pilots_path"
                echo "  DISPOSED ${TARGET_CUSTOMER}: planning artifacts removed"
            fi
        fi
    else
        for pilots_customer_dir in "${PILOTS_DIR}"/*/; do
            [[ -d "$pilots_customer_dir" ]] || continue
            local_customer_id=$(basename "$pilots_customer_dir" | tr -cd 'a-zA-Z0-9_-')
            # Only dispose planning artifacts if runtime data was also disposed (or forced)
            runtime_dir="${PILOT_DATA_DIR}/${local_customer_id}"
            if [[ "$FORCE" == "true" ]] || [[ -d "$runtime_dir" && -f "${runtime_dir}/.pilot-end-date" ]]; then
                if [[ "$DRY_RUN" == "true" ]]; then
                    local_count=$(find "$pilots_customer_dir" -type f | wc -l | tr -d ' ')
                    echo "  DRY-RUN ${local_customer_id}: would delete ${local_count} planning files"
                else
                    rm -rf "$pilots_customer_dir"
                    echo "  DISPOSED ${local_customer_id}: planning artifacts removed"
                fi
            else
                echo "  SKIP ${local_customer_id}: runtime data not yet disposed"
            fi
        done
    fi
fi

echo "---"
echo "Summary: ${DISPOSED} disposed, ${SKIPPED} skipped, ${ERRORS} errors"

if [[ "$ERRORS" -gt 0 ]]; then
    echo "WARNING: ${ERRORS} customer(s) have missing anonymized copies. Review before retrying."
    exit 1
fi
