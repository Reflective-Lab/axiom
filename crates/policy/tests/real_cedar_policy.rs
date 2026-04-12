use std::path::PathBuf;

use converge_policy::{
    ContextIn, DecideRequest, PolicyEngine, PolicyOutcome, PrincipalIn, ResourceIn,
};

fn expense_engine() -> PolicyEngine {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("policies/expense_approval.cedar");
    let policy = std::fs::read_to_string(path).expect("expense approval policy should exist");
    PolicyEngine::from_policy_str(&policy).expect("expense approval policy should parse")
}

fn expense_request(
    authority: &str,
    domains: Vec<&str>,
    action: &str,
    amount: i64,
    human_approval_present: bool,
    gates_passed: Vec<&str>,
) -> DecideRequest {
    DecideRequest {
        principal: PrincipalIn {
            id: "agent:test".into(),
            authority: authority.into(),
            domains: domains.into_iter().map(str::to_string).collect(),
            policy_version: Some("expense_v1".into()),
        },
        resource: ResourceIn {
            id: "expense:2026-04-12-001".into(),
            resource_type: Some("expense".into()),
            phase: Some("commitment".into()),
            gates_passed: Some(gates_passed.into_iter().map(str::to_string).collect()),
        },
        action: action.into(),
        context: Some(ContextIn {
            commitment_type: Some("expense".into()),
            amount: Some(amount),
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(true),
        }),
        delegation_b64: None,
    }
}

#[test]
fn finance_supervisor_can_validate_receipted_expense() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["finance"],
        "validate",
        2_500,
        false,
        vec!["receipt"],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Promote);
}

#[test]
fn non_finance_supervisor_cannot_validate_expense() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["operations"],
        "validate",
        2_500,
        false,
        vec!["receipt"],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[test]
fn finance_validation_requires_receipt_gate() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["finance"],
        "validate",
        2_500,
        false,
        vec![],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[test]
fn finance_supervisor_commit_over_threshold_without_approval_escalates() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["finance"],
        "commit",
        8_400,
        false,
        vec!["receipt", "manager_approval"],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Escalate);
}

#[test]
fn finance_supervisor_commit_requires_manager_gate_even_with_approval() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["finance"],
        "commit",
        4_200,
        true,
        vec!["receipt"],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[test]
fn finance_supervisor_can_commit_low_value_expense_with_approval() {
    let engine = expense_engine();
    let req = expense_request(
        "supervisory",
        vec!["finance"],
        "commit",
        4_200,
        true,
        vec!["receipt", "manager_approval"],
    );

    let decision = engine
        .evaluate(&req)
        .expect("policy evaluation should succeed");
    assert_eq!(decision.outcome, PolicyOutcome::Promote);
}
