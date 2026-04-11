// Authority rules for Extended team elevation and high-risk gate restrictions
// Phase 22: Authority Rules
//
// These policies work with foundation.cedar which already provides:
// - Core team permit all (AUTH-01)
// - Extended team EvaluateGate with evidence
// - Self-approval forbid (Phase 23)

// AUTH-02: Extended team can approve gates where they are elevated
// This policy permits Extended team members to ApproveGate when they are
// in the gate's elevated_approvers set (configured from GATES.md)
permit (
    principal in ConvergePersonas::Team::"Extended",
    action == ConvergePersonas::Action::"ApproveGate",
    resource
)
when {
    resource.elevated_approvers.contains(principal)
};

// AUTH-03: High-risk gates require Core team approval regardless of elevation
// This FORBID policy overrides any permit policies (including AUTH-02 elevation)
// Cedar guarantees: if any forbid evaluates true, result is Deny
//
// Effect: Extended team cannot approve high-risk gates even if elevated
// Examples: release-candidate, release-approval, customer-commitment, production-deploy
forbid (
    principal in ConvergePersonas::Team::"Extended",
    action == ConvergePersonas::Action::"ApproveGate",
    resource
)
when {
    resource.risk_class == "high"
};
