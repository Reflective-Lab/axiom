#!/bin/bash
# validate-policy.sh - Policy logic validation for governance rules
# Validates constraints that cannot be expressed in JSON Schema

set -euo pipefail

ERRORS=0
WARNINGS=0

echo "=== Validating Policy Logic ==="
echo ""

# Check dependencies
if ! command -v yq &> /dev/null; then
  echo "❌ ERROR: yq not installed (required for YAML parsing)"
  echo "   Install: brew install yq"
  exit 1
fi

# Helper: Get gate risk class from GATES.md
get_gate_risk_class() {
  local gate_id="$1"
  # Parse gate table to extract risk_class for given gate_id
  grep "^| $gate_id " GATES.md | awk -F'|' '{gsub(/^[[:space:]]+|[[:space:]]+$/, "", $5); print $5}' | head -1
}

# Helper: Get elevated blocking evals for a gate from GATES.md
get_elevated_evals() {
  local gate_id="$1"
  # Extract elevated_blocking_evals column from Elevated Evals by Gate table
  grep "^| $gate_id " GATES.md | grep -A0 "elevated_blocking_evals" | \
    awk -F'|' '{gsub(/^[[:space:]]+|[[:space:]]+$/, "", $3); print $3}' | \
    grep -v "^—$" | grep -v "^$" | head -1
}

# Helper: Check if eval is in elevated list
is_eval_elevated() {
  local gate_id="$1"
  local eval_id="$2"
  local elevated
  elevated=$(get_elevated_evals "$gate_id")

  if [ -z "$elevated" ] || [ "$elevated" = "—" ]; then
    return 1  # No elevated evals for this gate
  fi

  # Check if eval_id appears in comma-separated list
  echo "$elevated" | grep -q "$eval_id"
}

# Check 1: Determinism constraints
# Class C evals can only have outcome "advisory" (never "pass" or "fail" that blocks)
check_determinism_constraints() {
  local fixture="$1"
  local fixture_name
  fixture_name=$(basename "$fixture")

  # Get all eval results with Class C
  local class_c_count
  class_c_count=$(yq eval '.eval_results[] | select(.determinism_class == "C") | .eval_id' "$fixture" 2>/dev/null | wc -l | tr -d ' ')

  if [ "$class_c_count" -eq 0 ]; then
    return 0  # No Class C evals, nothing to check
  fi

  # Check each Class C eval
  while IFS= read -r eval_id; do
    if [ -z "$eval_id" ]; then
      continue
    fi

    local outcome
    outcome=$(yq eval ".eval_results[] | select(.eval_id == \"$eval_id\") | .outcome" "$fixture")

    if [ "$outcome" != "advisory" ]; then
      echo "❌ $fixture_name: Class C eval '$eval_id' has outcome '$outcome' (must be 'advisory')"
      ERRORS=$((ERRORS + 1))
    fi
  done < <(yq eval '.eval_results[] | select(.determinism_class == "C") | .eval_id' "$fixture")
}

# Check 2: Authority intersection
# Gate requires eval AND eval owner has authority at gate
check_authority_intersection() {
  local fixture="$1"
  local fixture_name
  fixture_name=$(basename "$fixture")
  local gate_id
  gate_id=$(yq eval '.gate_id' "$fixture")
  local decision_outcome
  decision_outcome=$(yq eval '.decision.outcome' "$fixture")

  # Check each eval_result that has outcome "fail"
  while IFS= read -r line; do
    if [ -z "$line" ]; then
      continue
    fi

    local eval_id outcome
    eval_id=$(echo "$line" | awk '{print $1}')
    outcome=$(echo "$line" | awk '{print $2}')

    if [ "$outcome" != "fail" ]; then
      continue
    fi

    # If eval fails and decision is block, check if eval has authority
    if [ "$decision_outcome" = "block" ]; then
      # Check if eval is Class C (which should never block)
      local determinism_class
      determinism_class=$(yq eval ".eval_results[] | select(.eval_id == \"$eval_id\") | .determinism_class" "$fixture")

      if [ "$determinism_class" = "C" ]; then
        echo "❌ $fixture_name: Class C eval '$eval_id' contributing to block (Class C evals can only be advisory)"
        ERRORS=$((ERRORS + 1))
      fi
    fi

    # Check if Extended eval fails but is not elevated
    # (This is a warning, not an error - Extended advisory fails are allowed)
    if ! is_eval_elevated "$gate_id" "$eval_id"; then
      # Check if eval is Extended by looking for common Extended eval patterns
      if echo "$eval_id" | grep -qE "(sre-operations|developer-advocate|marketing-lead|sales-engineer|sustainability|regulator-lens|insurance-underwriter|curious-searcher|investor|skeptical-critic|future-historian|journalist|academic-researcher|end-user-advocate|external-perspective)"; then
        if [ "$decision_outcome" = "block" ]; then
          echo "⚠️  $fixture_name: Extended eval '$eval_id' contributing to block but not in elevated_blocking_evals for $gate_id"
          echo "    (This may be intentional if Core evals also failed)"
          WARNINGS=$((WARNINGS + 1))
        fi
      fi
    fi
  done < <(yq eval '.eval_results[] | .eval_id + " " + .outcome' "$fixture" 2>/dev/null)
}

# Check 3: Override policy
# If decision.outcome = "override" and gate is high-risk:
# - Check override.policy_type = "two-person-required"
# - Check override.approvers has at least 2 entries
check_override_policy() {
  local fixture="$1"
  local fixture_name
  fixture_name=$(basename "$fixture")
  local gate_id
  gate_id=$(yq eval '.gate_id' "$fixture")
  local decision_outcome
  decision_outcome=$(yq eval '.decision.outcome' "$fixture")

  if [ "$decision_outcome" != "override" ]; then
    return 0  # No override, nothing to check
  fi

  local risk_class
  risk_class=$(get_gate_risk_class "$gate_id")

  if [ "$risk_class" = "high" ]; then
    local policy_type
    policy_type=$(yq eval '.override.policy_type' "$fixture")

    if [ "$policy_type" != "two-person-required" ]; then
      echo "❌ $fixture_name: High-risk gate '$gate_id' has override policy_type '$policy_type' (must be 'two-person-required')"
      ERRORS=$((ERRORS + 1))
    fi

    local approver_count
    approver_count=$(yq eval '.override.approvers | length' "$fixture" 2>/dev/null || echo "0")

    if [ "$approver_count" -lt 2 ]; then
      echo "❌ $fixture_name: High-risk gate '$gate_id' override has only $approver_count approver(s) (requires at least 2)"
      ERRORS=$((ERRORS + 1))
    fi
  fi
}

# Check 4: Escalation linkage
# If gate fixture has escalation.packet_ref:
# - Check packet file exists
# - If escalation.disposition_ref exists, check disposition file exists
# - Check escalation_id matches between packet and disposition
check_escalation_linkage() {
  local fixture="$1"
  local fixture_name
  fixture_name=$(basename "$fixture")

  local packet_ref
  packet_ref=$(yq eval '.escalation.packet_ref // ""' "$fixture")

  if [ -z "$packet_ref" ] || [ "$packet_ref" = "null" ]; then
    return 0  # No escalation, nothing to check
  fi

  # Check packet file exists
  if [ ! -f "$packet_ref" ]; then
    echo "❌ $fixture_name: Escalation packet reference '$packet_ref' does not exist"
    ERRORS=$((ERRORS + 1))
    return
  fi

  local disposition_ref
  disposition_ref=$(yq eval '.escalation.disposition_ref // ""' "$fixture")

  if [ -n "$disposition_ref" ] && [ "$disposition_ref" != "null" ]; then
    # Check disposition file exists
    if [ ! -f "$disposition_ref" ]; then
      echo "❌ $fixture_name: Escalation disposition reference '$disposition_ref' does not exist"
      ERRORS=$((ERRORS + 1))
      return
    fi

    # Check escalation_id matches between packet and disposition
    local packet_id disposition_id
    packet_id=$(yq eval '.escalation_id' "$packet_ref" 2>/dev/null)
    disposition_id=$(yq eval '.escalation_id' "$disposition_ref" 2>/dev/null)

    # Handle versioned escalation IDs (ESC-YYYY-NNN-v2 should match ESC-YYYY-NNN-v2)
    if [ "$packet_id" != "$disposition_id" ]; then
      echo "❌ $fixture_name: Escalation ID mismatch: packet has '$packet_id', disposition has '$disposition_id'"
      ERRORS=$((ERRORS + 1))
    fi
  fi
}

# Main validation
echo "--- Gate Fixtures ---"
echo ""

if [ ! -d "fixtures/gates" ]; then
  echo "⚠️  WARNING: fixtures/gates/ directory not found (may not exist yet)"
  echo ""
else
  gate_count=0
  for fixture in fixtures/gates/*.yaml; do
    if [ ! -f "$fixture" ]; then
      echo "⚠️  WARNING: No gate fixtures found in fixtures/gates/"
      break
    fi

    fixture_name=$(basename "$fixture")
    echo "Checking $fixture_name..."

    check_determinism_constraints "$fixture"
    check_authority_intersection "$fixture"
    check_override_policy "$fixture"
    check_escalation_linkage "$fixture"

    gate_count=$((gate_count + 1))
  done

  if [ $gate_count -gt 0 ]; then
    echo ""
    echo "Validated $gate_count gate fixture(s)"
    echo ""
  fi
fi

# Summary
echo "=== Validation Summary ==="
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
  echo "✅ All policy validations passed"
  exit 0
elif [ $ERRORS -eq 0 ]; then
  echo "⚠️  $WARNINGS warning(s) found (non-blocking)"
  exit 0
else
  echo "❌ $ERRORS validation error(s) found"
  if [ $WARNINGS -gt 0 ]; then
    echo "⚠️  $WARNINGS warning(s) found"
  fi
  exit 1
fi
