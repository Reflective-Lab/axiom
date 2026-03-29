#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "SECURITY.md"
  "docs/security/README.md"
  "docs/security/SECURITY_OVERVIEW.md"
  "docs/security/DATA_HANDLING_DECLARATION.md"
  "docs/security/COMPLIANCE_READINESS.md"
  "docs/security/THREAT_MODEL.md"
)

for file in "${required_files[@]}"; do
  [[ -f "$file" ]] || {
    echo "Missing required security/compliance document: $file" >&2
    exit 1
  }
done

for needle in \
  "Designed To Handle" \
  "Not Declared As Supported By Default" \
  "What We Should Not Claim Without Evidence" \
  "Shared Responsibility Model"
do
  rg -q "$needle" docs/security SECURITY.md || {
    echo "Missing required declaration text: $needle" >&2
    exit 1
  }
done

claim_pattern='SOC 2 certified|ISO 27001 certified|HIPAA compliant|PCI compliant|GDPR compliant'
if rg -n "$claim_pattern" README.md SECURITY.md docs/security scripts .github \
  --glob '!docs/security/COMPLIANCE_READINESS.md' \
  --glob '!scripts/validate-security-docs.sh'
then
  echo "Found unsupported compliance claim outside the approved disclaimer docs." >&2
  exit 1
fi

echo "Security/compliance docs validation passed."
