// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! YAML schema validator for domain packs.
//!
//! Enforces the contract: YAML = wiring, Gherkin = semantics.
//!
//! This validator rejects YAML files that contain semantic keys
//! (validation rules, invariants, etc.) that should be in Gherkin.

use std::collections::HashSet;
use thiserror::Error;

/// Errors from pack validation.
#[derive(Debug, Error)]
pub enum PackValidationError {
    #[error("Forbidden key in pack YAML: '{key}' (semantics must be in Gherkin spec)")]
    ForbiddenKey { key: String },

    #[error("Multiple forbidden keys in pack YAML: {keys:?}")]
    ForbiddenKeys { keys: Vec<String> },

    #[error("Missing required key: '{key}'")]
    MissingKey { key: String },

    #[error("Invalid YAML: {0}")]
    ParseError(String),
}

/// Keys that are forbidden in pack YAML (must be in Gherkin).
const FORBIDDEN_KEYS: &[&str] = &[
    "validation",
    "invariants",
    "forbidden_terms",
    "acceptance_criteria",
    "business_rules",
    "semantic_rules",
    "min_confidence",
    "max_content_length",
    "require_provenance",
];

/// Keys that are required in pack YAML.
const REQUIRED_KEYS: &[&str] = &["name", "version", "description"];

/// Keys that are allowed in pack YAML.
const ALLOWED_KEYS: &[&str] = &[
    "name",
    "version",
    "description",
    "spec",
    "requires",
    "budget",
    "agents",
    "metadata",
];

/// Validate a pack YAML value against the schema contract.
pub fn validate_pack_yaml(yaml: &serde_yaml::Value) -> Result<(), PackValidationError> {
    let mapping = yaml
        .as_mapping()
        .ok_or_else(|| PackValidationError::ParseError("Root must be a mapping".to_string()))?;

    // Check for forbidden keys
    let mut forbidden_found = Vec::new();
    for (key, _) in mapping {
        if let Some(key_str) = key.as_str() {
            if FORBIDDEN_KEYS.contains(&key_str) {
                forbidden_found.push(key_str.to_string());
            }
        }
    }

    if forbidden_found.len() == 1 {
        return Err(PackValidationError::ForbiddenKey {
            key: forbidden_found.into_iter().next().unwrap(),
        });
    } else if forbidden_found.len() > 1 {
        return Err(PackValidationError::ForbiddenKeys {
            keys: forbidden_found,
        });
    }

    // Check for required keys
    let keys: HashSet<&str> = mapping.keys().filter_map(|k| k.as_str()).collect();

    for required in REQUIRED_KEYS {
        if !keys.contains(required) {
            return Err(PackValidationError::MissingKey {
                key: (*required).to_string(),
            });
        }
    }

    Ok(())
}

/// Validate a pack YAML string.
pub fn validate_pack_yaml_str(yaml_str: &str) -> Result<(), PackValidationError> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
        .map_err(|e| PackValidationError::ParseError(e.to_string()))?;
    validate_pack_yaml(&value)
}

/// Get the list of allowed keys for documentation.
pub fn allowed_keys() -> &'static [&'static str] {
    ALLOWED_KEYS
}

/// Get the list of forbidden keys for documentation.
pub fn forbidden_keys() -> &'static [&'static str] {
    FORBIDDEN_KEYS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pack_yaml() {
        let yaml = r#"
name: test-pack
version: "1.0.0"
description: A test pack
spec: specs/test.feature

budget:
  max_cycles: 50
  max_facts: 500

agents:
  - name: TestAgent
    requirements: fast_extraction
"#;

        let result = validate_pack_yaml_str(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_forbidden_validation_key() {
        let yaml = r#"
name: bad-pack
version: "1.0.0"
description: Has forbidden key

validation:
  min_confidence: 0.8
"#;

        let result = validate_pack_yaml_str(yaml);
        assert!(matches!(
            result,
            Err(PackValidationError::ForbiddenKey { key }) if key == "validation"
        ));
    }

    #[test]
    fn test_forbidden_invariants_key() {
        let yaml = r#"
name: bad-pack
version: "1.0.0"
description: Has forbidden key

invariants:
  - BrandSafetyInvariant
"#;

        let result = validate_pack_yaml_str(yaml);
        assert!(matches!(
            result,
            Err(PackValidationError::ForbiddenKey { key }) if key == "invariants"
        ));
    }

    #[test]
    fn test_multiple_forbidden_keys() {
        let yaml = r#"
name: bad-pack
version: "1.0.0"
description: Has multiple forbidden keys

validation:
  min_confidence: 0.8

invariants:
  - SomeInvariant

forbidden_terms:
  - guaranteed
"#;

        let result = validate_pack_yaml_str(yaml);
        assert!(
            matches!(result, Err(PackValidationError::ForbiddenKeys { keys }) if keys.len() == 3)
        );
    }

    #[test]
    fn test_missing_required_key() {
        let yaml = r#"
name: incomplete-pack
description: Missing version
"#;

        let result = validate_pack_yaml_str(yaml);
        assert!(matches!(
            result,
            Err(PackValidationError::MissingKey { key }) if key == "version"
        ));
    }
}
