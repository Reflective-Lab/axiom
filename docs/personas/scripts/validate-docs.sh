#!/bin/bash
# validate-docs.sh - Documentation validation (Layer 4)
# Checks markdown formatting, link integrity, and fixture references

set -euo pipefail

ERRORS=0
WARNINGS=0

echo "=== Documentation Validation ==="
echo ""

# Check for required dependencies
check_dependency() {
  local cmd=$1
  local install_cmd=$2
  if ! command -v "$cmd" &> /dev/null; then
    echo "ERROR: $cmd is not installed"
    echo "  Install: $install_cmd"
    return 1
  fi
  return 0
}

DEPS_MISSING=0

echo "--- Checking Dependencies ---"
echo ""

if ! check_dependency "markdownlint-cli2" "npm install -g markdownlint-cli2"; then
  DEPS_MISSING=1
fi

if ! check_dependency "markdown-link-check" "npm install -g markdown-link-check"; then
  DEPS_MISSING=1
fi

if [ $DEPS_MISSING -eq 1 ]; then
  echo ""
  echo "Install missing dependencies and re-run."
  exit 1
fi

echo "Dependencies satisfied."
echo ""

# Collect markdown files to validate
MD_FILES=()

# Root level markdown files
for f in *.md; do
  [ -f "$f" ] && MD_FILES+=("$f")
done

# docs/**/*.md
if [ -d "docs" ]; then
  while IFS= read -r -d '' f; do
    MD_FILES+=("$f")
  done < <(find docs -name "*.md" -type f -print0 2>/dev/null)
fi

# examples/**/*.md
if [ -d "examples" ]; then
  while IFS= read -r -d '' f; do
    MD_FILES+=("$f")
  done < <(find examples -name "*.md" -type f -print0 2>/dev/null)
fi

# schemas/**/*.md (markdown files only, not .json)
if [ -d "schemas" ]; then
  while IFS= read -r -d '' f; do
    MD_FILES+=("$f")
  done < <(find schemas -name "*.md" -type f -print0 2>/dev/null)
fi

echo "Found ${#MD_FILES[@]} markdown files to validate"
echo ""

# Layer 4a: Markdownlint
echo "--- Layer 4a: Markdown Linting ---"
echo ""

if [ ${#MD_FILES[@]} -gt 0 ]; then
  if markdownlint-cli2 "${MD_FILES[@]}" 2>&1; then
    echo "Markdown linting passed"
  else
    echo "Markdown linting found issues"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "No markdown files to lint"
fi

echo ""

# Layer 4b: Link checking
echo "--- Layer 4b: Link Checking ---"
echo ""

LINK_ERRORS=0
for f in "${MD_FILES[@]}"; do
  if ! markdown-link-check -q -c .markdown-link-check.json "$f" 2>/dev/null; then
    echo "Broken links in: $f"
    LINK_ERRORS=$((LINK_ERRORS + 1))
  fi
done

if [ $LINK_ERRORS -gt 0 ]; then
  echo ""
  echo "$LINK_ERRORS file(s) have broken links"
  ERRORS=$((ERRORS + 1))
else
  echo "Link checking passed"
fi

echo ""

# Layer 4c: Fixture reference validation
echo "--- Layer 4c: Fixture Reference Validation ---"
echo ""

FIXTURE_ERRORS=0

# Find fixture references in markdown files
while IFS= read -r line; do
  # Extract file path and the referenced fixture path
  file=$(echo "$line" | cut -d: -f1)
  # Extract path from the line (handles various reference formats)
  fixture_path=$(echo "$line" | grep -oE 'fixtures/[a-zA-Z0-9_/-]+\.(yaml|yml|json)' | head -1)

  if [ -n "$fixture_path" ] && [ ! -f "$fixture_path" ]; then
    echo "Missing fixture: $fixture_path (referenced in $file)"
    FIXTURE_ERRORS=$((FIXTURE_ERRORS + 1))
  fi
done < <(grep -rn "fixtures/" "${MD_FILES[@]}" 2>/dev/null || true)

if [ $FIXTURE_ERRORS -gt 0 ]; then
  echo ""
  echo "$FIXTURE_ERRORS missing fixture reference(s) found"
  ERRORS=$((ERRORS + 1))
else
  echo "Fixture reference validation passed"
fi

echo ""
echo "=== Documentation Validation Summary ==="
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
  echo "All documentation validations passed"
  exit 0
elif [ $ERRORS -eq 0 ]; then
  echo "$WARNINGS warning(s) found (non-blocking)"
  exit 0
else
  echo "$ERRORS validation error(s) found"
  if [ $WARNINGS -gt 0 ]; then
    echo "$WARNINGS warning(s) found"
  fi
  exit 1
fi
