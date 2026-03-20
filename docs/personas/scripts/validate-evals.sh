#!/bin/bash
# validate-evals.sh - Gray-box validation
# Checks frontmatter structure exists without parsing YAML

set -e

ERRORS=0
WARNINGS=0

echo "=== Validating Eval Frontmatter Structure ==="
echo ""

# Check if evals directory exists
if [ ! -d "evals" ]; then
  echo "❌ ERROR: evals/ directory not found"
  exit 1
fi

# Check each eval file
for eval_file in evals/*-eval.md; do
  # Skip if no files match
  if [ ! -f "$eval_file" ]; then
    echo "⚠️  WARNING: No eval files found in evals/"
    WARNINGS=$((WARNINGS + 1))
    break
  fi

  eval_name=$(basename "$eval_file")

  # Check 1: Opening delimiter on line 1
  if ! head -1 "$eval_file" | grep -q "^---$"; then
    echo "❌ $eval_name: Missing opening frontmatter delimiter (line 1 should be '---')"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  # Check 2: Closing delimiter within first 60 lines
  if ! head -60 "$eval_file" | tail -n +2 | grep -q "^---$"; then
    echo "❌ $eval_name: Missing closing frontmatter delimiter (should appear within first 60 lines)"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  # Check 3: Required fields exist (simple grep, not parsed)
  MISSING_FIELDS=()
  for field in "eval_id:" "owner:" "intent:" "determinism:" "governance:"; do
    if ! head -60 "$eval_file" | grep -q "^$field"; then
      MISSING_FIELDS+=("$field")
      ERRORS=$((ERRORS + 1))
    fi
  done

  if [ ${#MISSING_FIELDS[@]} -gt 0 ]; then
    echo "❌ $eval_name: Missing required fields: ${MISSING_FIELDS[*]}"
  else
    echo "✓ $eval_name: Frontmatter structure valid"
  fi
done

echo ""
echo "=== Validating Registry ==="
echo ""

# Check registry file exists
if [ ! -f ".planning/registry/evals.yaml" ]; then
  echo "⚠️  WARNING: Registry file missing: .planning/registry/evals.yaml"
  echo "   (This is expected until plan 03-01 Task 3 completes)"
  WARNINGS=$((WARNINGS + 1))
else
  echo "✓ Registry file exists"

  # Check all registry paths exist
  # (Simple grep-based, no YAML parsing)
  while IFS= read -r line; do
    # Extract path value (handles leading spaces)
    path=$(echo "$line" | sed 's/.*path:[[:space:]]*//' | sed 's/[[:space:]]*$//')

    if [ -n "$path" ] && [ ! -f "$path" ]; then
      echo "❌ Registry references missing file: $path"
      ERRORS=$((ERRORS + 1))
    fi
  done < <(grep "path: evals/" .planning/registry/evals.yaml 2>/dev/null || true)

  if [ $ERRORS -eq 0 ]; then
    echo "✓ All registry path references exist"
  fi
fi

echo ""
echo "=== Validation Summary ==="
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
  echo "✅ All validations passed"
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
