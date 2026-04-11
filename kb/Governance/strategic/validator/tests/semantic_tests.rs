//! Unit tests for semantic validation rules VAL-01, VAL-02, VAL-03
//!
//! Each test uses minimal inline YAML fixtures to prove specific rule behavior.

// Import types directly from main.rs since we don't have a lib.rs
// This is a common pattern for binary-only crates that need testing
#[path = "../src/main.rs"]
mod validator;

use validator::LensPacksConfig;

/// Helper to create a minimal lens packs config for testing
fn test_lens_packs() -> LensPacksConfig {
    let yaml = r#"
version: "1.0"
gates:
  test-gate:
    primary_voices:
      - voice_a
      - voice_b
    secondary_voices:
      - voice_c
    counter_voices:
      - counter_x
      - counter_y
"#;
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn val01_passes_when_all_primaries_present() {
    let yaml = r"
fixture_id: test-gate.pass
gate_id: test-gate
lens_pack_version: v1.0
artifact:
  type: documentation
  ref: test.md
  summary: Test artifact
lens_results:
  - check_id: voice_a.check.v1
    voice: voice_a
    class: A
    severity: PASS
    summary: ok
  - check_id: voice_b.check.v1
    voice: voice_b
    class: A
    severity: PASS
    summary: ok
decision:
  outcome: PASS
";

    let lens_packs = test_lens_packs();
    let errors = validator::validate_semantic_str(
        yaml,
        &lens_packs,
        "test-gate",
    );

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
}

#[test]
fn val01_fails_when_primary_missing() {
    let yaml = r"
fixture_id: test-gate.missing
gate_id: test-gate
lens_pack_version: v1.0
artifact:
  type: documentation
  ref: test.md
  summary: Test artifact
lens_results:
  - check_id: voice_a.check.v1
    voice: voice_a
    class: A
    severity: PASS
    summary: ok
decision:
  outcome: PASS
";

    let lens_packs = test_lens_packs();
    let errors = validator::validate_semantic_str(
        yaml,
        &lens_packs,
        "test-gate",
    );

    assert_eq!(errors.len(), 1);
    let error_str = errors[0].to_string();
    assert!(error_str.contains("E_VAL01_MISSING_PRIMARY"), "Expected VAL01 error, got: {error_str}");
    assert!(error_str.contains("voice_b"), "Expected missing voice_b, got: {error_str}");
}

#[test]
fn val02_fails_when_warn_missing_ack_ref() {
    let yaml = r"
fixture_id: test-gate.warn-no-ack
gate_id: test-gate
lens_pack_version: v1.0
artifact:
  type: documentation
  ref: test.md
  summary: Test artifact
lens_results:
  - check_id: voice_a.check.v1
    voice: voice_a
    class: B
    severity: WARN
    summary: needs attention
  - check_id: voice_b.check.v1
    voice: voice_b
    class: A
    severity: PASS
    summary: ok
decision:
  outcome: WARN
";

    let lens_packs = test_lens_packs();
    let errors = validator::validate_semantic_str(
        yaml,
        &lens_packs,
        "test-gate",
    );

    assert!(!errors.is_empty());
    let error_str = errors[0].to_string();
    assert!(error_str.contains("E_VAL02_ACK_MISSING"), "Expected VAL02 ack missing error, got: {error_str}");
}

#[test]
fn val03_fails_when_stop_has_no_class_a_stop() {
    let yaml = r"
fixture_id: test-gate.stop-no-class-a
gate_id: test-gate
lens_pack_version: v1.0
artifact:
  type: documentation
  ref: test.md
  summary: Test artifact
lens_results:
  - check_id: voice_a.check.v1
    voice: voice_a
    class: B
    severity: WARN
    summary: just a warning
  - check_id: voice_b.check.v1
    voice: voice_b
    class: A
    severity: PASS
    summary: ok
decision:
  outcome: STOP
";

    let lens_packs = test_lens_packs();
    let errors = validator::validate_semantic_str(
        yaml,
        &lens_packs,
        "test-gate",
    );

    assert!(!errors.is_empty());
    let error_str = errors[0].to_string();
    assert!(error_str.contains("E_VAL03_STOP_NOT_CLASS_A"), "Expected VAL03 error, got: {error_str}");
}

#[test]
fn val03_passes_when_stop_has_class_a_stop() {
    let yaml = r"
fixture_id: test-gate.stop-valid
gate_id: test-gate
lens_pack_version: v1.0
artifact:
  type: documentation
  ref: test.md
  summary: Test artifact
lens_results:
  - check_id: voice_a.check.v1
    voice: voice_a
    class: A
    severity: STOP
    summary: blocking issue
  - check_id: voice_b.check.v1
    voice: voice_b
    class: A
    severity: PASS
    summary: ok
decision:
  outcome: STOP
";

    let lens_packs = test_lens_packs();
    let errors = validator::validate_semantic_str(
        yaml,
        &lens_packs,
        "test-gate",
    );

    // Should have no VAL-03 errors (may still have VAL-01 if primaries logic applies)
    let val03_errors: Vec<_> = errors.iter()
        .filter(|e| e.to_string().contains("E_VAL03"))
        .collect();
    assert!(val03_errors.is_empty(), "Expected no VAL03 errors, got: {val03_errors:?}");
}
