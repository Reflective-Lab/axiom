#!/usr/bin/env bash
set -euo pipefail

# Cedar Policy Validation Script
# Validates Cedar schema and policy files using cedar-policy-cli

echo "=== Cedar Policy Validation ==="
echo ""

# Add ~/.cargo/bin to PATH if it exists and isn't already in PATH
if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Check for cedar CLI
if ! command -v cedar &> /dev/null; then
    echo "Cedar CLI not found. Installing cedar-policy-cli@4.8.2..."
    cargo install cedar-policy-cli@4.8.2
    echo ""
fi

# Verify cedar is now available
if ! command -v cedar &> /dev/null; then
    echo "ERROR: Failed to install cedar CLI"
    exit 1
fi

echo "Using: $(cedar --version)"
echo ""

# Define paths
SCHEMA_PATH="strategic/validator/cedar/schema.cedarschema"
POLICIES_DIR="strategic/validator/cedar/policies"

# Validate schema exists
if [ ! -f "$SCHEMA_PATH" ]; then
    echo "ERROR: Schema not found at $SCHEMA_PATH"
    exit 1
fi

# Validate schema
echo "Validating schema: $SCHEMA_PATH"
if cedar validate --schema "$SCHEMA_PATH" > /dev/null 2>&1; then
    echo "✓ Schema is valid"
else
    echo "✗ Schema validation failed"
    cedar validate --schema "$SCHEMA_PATH"
    exit 1
fi
echo ""

# Check if policies directory exists
if [ ! -d "$POLICIES_DIR" ]; then
    echo "WARNING: Policies directory not found at $POLICIES_DIR"
    echo "Skipping policy validation (no policies to validate)"
    exit 0
fi

# Count policy files
POLICY_COUNT=$(find "$POLICIES_DIR" -type f -name "*.cedar" | wc -l | tr -d ' ')

if [ "$POLICY_COUNT" -eq 0 ]; then
    echo "WARNING: No .cedar policy files found in $POLICIES_DIR"
    echo "Skipping policy validation (no policies to validate)"
    exit 0
fi

echo "Found $POLICY_COUNT policy file(s) to validate"
echo ""

# Validate each policy file
FAILED=0
for policy in "$POLICIES_DIR"/*.cedar; do
    if [ ! -f "$policy" ]; then
        continue
    fi

    POLICY_NAME=$(basename "$policy")
    echo "Validating: $POLICY_NAME"

    if cedar validate --schema "$SCHEMA_PATH" --policies "$policy" > /dev/null 2>&1; then
        echo "✓ $POLICY_NAME is valid"
    else
        echo "✗ $POLICY_NAME validation failed:"
        cedar validate --schema "$SCHEMA_PATH" --policies "$policy"
        FAILED=$((FAILED + 1))
    fi
    echo ""
done

# Report results
echo "=== Validation Summary ==="
if [ $FAILED -eq 0 ]; then
    echo "✓ All validations passed"
    echo "  - Schema: valid"
    echo "  - Policies: $POLICY_COUNT/$POLICY_COUNT valid"
    exit 0
else
    echo "✗ Validation failed"
    echo "  - Failed policies: $FAILED/$POLICY_COUNT"
    exit 1
fi
