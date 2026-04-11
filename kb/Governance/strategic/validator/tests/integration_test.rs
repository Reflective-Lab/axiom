//! Integration test: run validator against all real fixtures
//!
//! This test ensures the validator passes all 21 fixtures in the repository.
//! It serves as a regression test for the semantic rules.

use std::process::Command;

#[test]
fn all_fixtures_pass_validation() {
    // Run the validator binary against the real fixtures
    // CARGO_MANIFEST_DIR is validator/, so we run from here
    let validator_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let output = Command::new("cargo")
        .args(["run", "--release"])
        .current_dir(validator_dir)
        .output()
        .expect("Failed to run validator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print output for debugging if test fails
    if !output.status.success() {
        eprintln!("STDOUT:\n{stdout}");
        eprintln!("STDERR:\n{stderr}");
    }

    assert!(output.status.success(), "Validator exited with non-zero status");

    // Verify expected counts in output
    assert!(
        stdout.contains("passed") && stdout.contains("0 failed"),
        "Expected all fixtures to pass. Output:\n{stdout}"
    );
}

#[test]
fn validator_loads_lens_packs_config() {
    // Verify the config file exists and is loadable
    let config_path = env!("CARGO_MANIFEST_DIR").to_owned() + "/../config/lens_packs.yaml";
    let content = std::fs::read_to_string(&config_path)
        .expect("lens_packs.yaml should exist");

    // Basic structure check
    assert!(content.contains("primary_voices"), "Config should define primary_voices");
    assert!(content.contains("counter_voices"), "Config should define counter_voices");
    assert!(content.contains("content-publish"), "Config should define content-publish gate");
    assert!(content.contains("funding-narrative"), "Config should define funding-narrative gate");
}
