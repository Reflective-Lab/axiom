use ed25519_dalek::SigningKey;

use converge_policy::{ContextIn, DecideRequest, PolicyEngine, PrincipalIn, ResourceIn};

pub fn test_engine() -> PolicyEngine {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("policies/policy.cedar");
    let policy = std::fs::read_to_string(path).expect("policy file should exist");
    PolicyEngine::from_policy_str(&policy).expect("policy should parse")
}

pub fn make_request(authority: &str, action: &str) -> DecideRequest {
    DecideRequest {
        principal: PrincipalIn {
            id: "agent:test".into(),
            authority: authority.into(),
            domains: vec!["test".into()],
            policy_version: None,
        },
        resource: ResourceIn {
            id: "flow:test-001".into(),
            resource_type: Some("quote".into()),
            phase: Some("convergence".into()),
            gates_passed: Some(vec!["evidence".into()]),
        },
        action: action.into(),
        context: Some(ContextIn {
            commitment_type: Some("quote".into()),
            amount: Some(5_000),
            human_approval_present: Some(false),
            required_gates_met: Some(true),
        }),
        delegation_b64: None,
    }
}

#[allow(dead_code)]
pub fn fixed_signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}
