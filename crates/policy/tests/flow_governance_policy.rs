use converge_core::{
    FlowAction, FlowGateContext, FlowGateInput, FlowGateOutcome, FlowGatePrincipal,
    FlowGateResource,
};
use converge_policy::{FLOW_GOVERNANCE_POLICY, FlowGateAuthorizer, PolicyEngine};

fn engine() -> PolicyEngine {
    PolicyEngine::from_policy_str(FLOW_GOVERNANCE_POLICY)
        .expect("flow governance policy should parse")
}

fn input(
    domain: &str,
    kind: &str,
    authority: &str,
    human_approval_present: bool,
    gates_passed: Vec<&str>,
    required_gates_met: bool,
    amount: Option<i64>,
) -> FlowGateInput {
    FlowGateInput {
        principal: FlowGatePrincipal {
            id: format!("agent:{domain}:{authority}"),
            authority: authority.into(),
            domains: vec![domain.into()],
            policy_version: Some("flow_governance_v1".into()),
        },
        resource: FlowGateResource {
            id: format!("{kind}:001"),
            kind: kind.into(),
            phase: "commitment".into(),
            gates_passed: gates_passed.into_iter().map(str::to_string).collect(),
        },
        action: FlowAction::Commit,
        context: FlowGateContext {
            commitment_type: Some(kind.into()),
            amount,
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(required_gates_met),
        },
    }
}

#[test]
fn supervisory_sales_quote_commit_without_approval_escalates() {
    let decision = engine()
        .decide(&input(
            "sales",
            "quote",
            "supervisory",
            false,
            vec![],
            true,
            Some(50_000),
        ))
        .expect("quote policy should evaluate");

    assert_eq!(decision.outcome, FlowGateOutcome::Escalate);
}

#[test]
fn supervisory_procurement_spend_commit_with_approval_promotes() {
    let decision = engine()
        .decide(&input(
            "procurement",
            "spend",
            "supervisory",
            true,
            vec!["vendor_due_diligence", "competitive_review"],
            true,
            Some(15_000),
        ))
        .expect("spend policy should evaluate");

    assert_eq!(decision.outcome, FlowGateOutcome::Promote);
}

#[test]
fn supervisory_legal_contract_commit_without_approval_escalates() {
    let decision = engine()
        .decide(&input(
            "legal",
            "contract",
            "supervisory",
            false,
            vec!["legal_review", "counterparty_signed"],
            true,
            None,
        ))
        .expect("contract policy should evaluate");

    assert_eq!(decision.outcome, FlowGateOutcome::Escalate);
}

#[test]
fn supervisory_finance_invoice_commit_with_approval_promotes() {
    let decision = engine()
        .decide(&input(
            "finance",
            "invoice",
            "supervisory",
            true,
            vec!["customer_validated", "line_items_balanced"],
            true,
            Some(12_500),
        ))
        .expect("invoice policy should evaluate");

    assert_eq!(decision.outcome, FlowGateOutcome::Promote);
}

#[test]
fn invoice_commit_missing_balanced_line_items_rejects() {
    let decision = engine()
        .decide(&input(
            "finance",
            "invoice",
            "supervisory",
            true,
            vec!["customer_validated"],
            true,
            Some(12_500),
        ))
        .expect("invoice policy should evaluate");

    assert_eq!(decision.outcome, FlowGateOutcome::Reject);
}
