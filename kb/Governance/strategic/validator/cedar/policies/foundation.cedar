// Foundation policies for ConvergePersonas governance system
// These basic policies validate schema correctness and will be expanded in phases 22-23

// Core team full access policy
// Core team members (Blocking-by-Policy authority) have unrestricted access to all actions
permit (
    principal in ConvergePersonas::Team::"Core",
    action,
    resource
);

// Extended team gate evaluation policy
// Extended team members can evaluate gates when evidence is provided
permit (
    principal in ConvergePersonas::Team::"Extended",
    action == ConvergePersonas::Action::"EvaluateGate",
    resource
)
when {
    context.evidence_provided == true
};

// Self-approval forbid policy (placeholder for Phase 23)
// Prevents self-approval of overrides (two-person rule enforcement)
forbid (
    principal,
    action == ConvergePersonas::Action::"ApproveOverride",
    resource
)
when {
    context.is_self_approval == true
};
