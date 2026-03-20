#!/bin/bash
# validate-structure.sh - JSON Schema validation for YAML fixtures
# Validates gate and escalation fixtures against JSON Schemas

set -euo pipefail

ERRORS=0

echo "=== Validating Fixture Structure ==="
echo ""

# Check dependencies
if ! command -v yq &> /dev/null; then
  echo "❌ ERROR: yq not installed (required for YAML to JSON conversion)"
  echo "   Install: brew install yq"
  exit 1
fi

if ! command -v ajv &> /dev/null; then
  echo "❌ ERROR: ajv-cli not installed (required for JSON Schema validation)"
  echo "   Install: npm install -g ajv-cli"
  exit 1
fi

# Function to validate a fixture against a schema
validate_fixture() {
  local fixture_path="$1"
  local schema_path="$2"
  local fixture_name
  fixture_name=$(basename "$fixture_path")

  # Convert YAML to JSON
  if ! yq eval -o=json "$fixture_path" > /tmp/fixture.json 2>/dev/null; then
    echo "❌ $fixture_name: Failed to parse YAML"
    ERRORS=$((ERRORS + 1))
    return
  fi

  # Validate against schema
  if ajv validate -s "$schema_path" -d /tmp/fixture.json --spec=draft7 2>&1 | grep -q "valid"; then
    echo "✓ $fixture_name: Schema validation passed"
  else
    echo "❌ $fixture_name: Schema validation failed"
    ajv validate -s "$schema_path" -d /tmp/fixture.json --spec=draft7 2>&1 | grep -v "^ajv" || true
    ERRORS=$((ERRORS + 1))
  fi

  # Cleanup
  rm -f /tmp/fixture.json
}

# Validate gate fixtures
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
    validate_fixture "$fixture" "schemas/gate_execution.schema.json"
    gate_count=$((gate_count + 1))
  done

  if [ $gate_count -gt 0 ]; then
    echo ""
    echo "Validated $gate_count gate fixture(s)"
    echo ""
  fi
fi

# Validate escalation packet fixtures
echo "--- Escalation Packet Fixtures ---"
echo ""

if [ ! -d "fixtures/escalations" ]; then
  echo "⚠️  WARNING: fixtures/escalations/ directory not found (may not exist yet)"
  echo ""
else
  packet_count=0
  for fixture in fixtures/escalations/packet.*.yaml; do
    if [ ! -f "$fixture" ]; then
      echo "⚠️  WARNING: No escalation packet fixtures found in fixtures/escalations/"
      break
    fi
    validate_fixture "$fixture" "schemas/escalation_packet.schema.json"
    packet_count=$((packet_count + 1))
  done

  if [ $packet_count -gt 0 ]; then
    echo ""
    echo "Validated $packet_count escalation packet fixture(s)"
    echo ""
  fi
fi

# Validate escalation disposition fixtures
echo "--- Escalation Disposition Fixtures ---"
echo ""

if [ ! -d "fixtures/escalations" ]; then
  echo "⚠️  WARNING: fixtures/escalations/ directory not found (may not exist yet)"
  echo ""
else
  disposition_count=0
  for fixture in fixtures/escalations/disposition.*.yaml; do
    if [ ! -f "$fixture" ]; then
      echo "⚠️  WARNING: No escalation disposition fixtures found in fixtures/escalations/"
      break
    fi
    validate_fixture "$fixture" "schemas/escalation_disposition.schema.json"
    disposition_count=$((disposition_count + 1))
  done

  if [ $disposition_count -gt 0 ]; then
    echo ""
    echo "Validated $disposition_count escalation disposition fixture(s)"
    echo ""
  fi
fi

# Summary
echo "=== Validation Summary ==="
echo ""

if [ $ERRORS -eq 0 ]; then
  echo "✅ All structure validations passed"
  exit 0
else
  echo "❌ $ERRORS validation error(s) found"
  exit 1
fi
