use std::sync::Arc;

use converge_core::{
    ContextKey, ContextView, FlowAction, FlowGateAuthorizer, FlowGateContext, FlowGateInput,
    FlowGatePrincipal, FlowGateResource,
};
use converge_policy::{FLOW_GOVERNANCE_POLICY, PolicyEngine};

pub(crate) fn default_flow_authorizer() -> Arc<dyn FlowGateAuthorizer> {
    Arc::new(
        PolicyEngine::from_policy_str(FLOW_GOVERNANCE_POLICY)
            .expect("built-in flow governance Cedar policy should parse"),
    )
}

pub(crate) fn has_approval(
    ctx: &dyn ContextView,
    scope: &str,
    target_id: &str,
    required_role: &str,
) -> bool {
    let expected_id = format!("approval:{scope}:{target_id}");
    ctx.get(ContextKey::Proposals).iter().any(|fact| {
        fact.id == expected_id
            || (fact.id.starts_with("approval:")
                && fact.content.contains(target_id)
                && fact.content.contains(required_role))
    })
}

pub(crate) fn flow_input(
    principal_id: &str,
    authority: &str,
    domain: &str,
    resource_id: String,
    kind: &str,
    gates_passed: Vec<String>,
    amount: Option<i64>,
    human_approval_present: bool,
    required_gates_met: bool,
    action: FlowAction,
) -> FlowGateInput {
    FlowGateInput {
        principal: FlowGatePrincipal {
            id: principal_id.into(),
            authority: authority.into(),
            domains: vec![domain.into()],
            policy_version: Some("flow_governance_v1".into()),
        },
        resource: FlowGateResource {
            id: resource_id,
            kind: kind.into(),
            phase: "commitment".into(),
            gates_passed,
        },
        action,
        context: FlowGateContext {
            commitment_type: Some(kind.into()),
            amount,
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(required_gates_met),
        },
    }
}

pub(crate) fn json_has_array_items(value: &serde_json::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| !items.is_empty())
}
