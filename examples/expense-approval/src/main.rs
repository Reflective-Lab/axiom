// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Expense Approval Workflow — multi-level approval with HITL gates.
//!
//! Demonstrates: long-running workflows, humans in the loop, multi-tier approvals.

use converge_core::{
    Agent, AgentEffect, Context, ContextKey, Engine, EngineHitlPolicy, Fact, ProposedFact,
    RunResult,
    gates::hitl::GateDecision,
    gates::{TimeoutAction, TimeoutPolicy},
};

const MANAGER_THRESHOLD: f64 = 1_000.0;
const FINANCE_THRESHOLD: f64 = 10_000.0;

struct ExpenseParsingAgent;

impl Agent for ExpenseParsingAgent {
    fn name(&self) -> &str {
        "ExpenseParsingAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Seeds) && !ctx.has(ContextKey::Strategies)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let seed = seeds.first();

        let parsed = if let Some(s) = seed {
            let json: serde_json::Value = serde_json::from_str(&s.content).unwrap_or_default();
            Fact {
                key: ContextKey::Strategies,
                id: "parsed-expense".to_string(),
                content: serde_json::to_string(&json).unwrap_or_default(),
            }
        } else {
            Fact {
                key: ContextKey::Strategies,
                id: "parsed-expense".to_string(),
                content: "{}".to_string(),
            }
        };

        AgentEffect::with_facts(vec![parsed])
    }
}

struct PolicyValidationAgent;

impl Agent for PolicyValidationAgent {
    fn name(&self) -> &str {
        "PolicyValidationAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Strategies) && !ctx.has(ContextKey::Evaluations)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let strategies = ctx.get(ContextKey::Strategies);
        let strategy = strategies.first();

        let mut is_compliant = true;
        let mut violations = Vec::new();

        if let Some(s) = strategy {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s.content) {
                let amount = json.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let category = json.get("category").and_then(|v| v.as_str()).unwrap_or("");

                if category == "entertainment" && amount > 500.0 {
                    is_compliant = false;
                    violations.push("Entertainment over $500 requires executive approval");
                }
                if amount > 50_000.0 {
                    is_compliant = false;
                    violations.push("Amount exceeds single approval limit");
                }
            }
        }

        let result = serde_json::json!({
            "compliant": is_compliant,
            "violations": violations
        });

        AgentEffect::with_facts(vec![Fact {
            key: ContextKey::Evaluations,
            id: "policy-validation".to_string(),
            content: result.to_string(),
        }])
    }
}

struct ApprovalRoutingAgent;

impl Agent for ApprovalRoutingAgent {
    fn name(&self) -> &str {
        "ApprovalRoutingAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Evaluations) && !ctx.has(ContextKey::Constraints)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let evaluations = ctx.get(ContextKey::Evaluations);
        let strategies = ctx.get(ContextKey::Strategies);

        let mut required_approvers = vec!["manager".to_string()];

        if let (Some(e), Some(s)) = (evaluations.first(), strategies.first()) {
            let eval: serde_json::Value = serde_json::from_str(&e.content).unwrap_or_default();
            let strat: serde_json::Value = serde_json::from_str(&s.content).unwrap_or_default();

            let amount = strat.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let compliant = eval
                .get("compliant")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            if !compliant {
                required_approvers.push("finance".to_string());
            } else if amount >= FINANCE_THRESHOLD {
                required_approvers.push("finance".to_string());
                required_approvers.push("executive".to_string());
            } else if amount >= MANAGER_THRESHOLD {
                required_approvers.push("finance".to_string());
            }
        }

        let routing = serde_json::json!({
            "required_approvers": required_approvers,
            "current_approver": "manager",
            "pending": required_approvers.len()
        });

        AgentEffect::with_facts(vec![Fact {
            key: ContextKey::Constraints,
            id: "approval-routing".to_string(),
            content: routing.to_string(),
        }])
    }
}

struct ApprovalSimulationAgent;

impl Agent for ApprovalSimulationAgent {
    fn name(&self) -> &str {
        "ApprovalSimulationAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Constraints]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Constraints)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let constraints = ctx.get(ContextKey::Constraints);

        if let Some(c) = constraints.first() {
            if let Ok(routing) = serde_json::from_str::<serde_json::Value>(&c.content) {
                let current = routing
                    .get("current_approver")
                    .and_then(|v| v.as_str())
                    .unwrap_or("manager");

                let proposal = ProposedFact {
                    key: ContextKey::Proposals,
                    id: format!("{}-approval", current),
                    content: format!("Approved by {}", current),
                    confidence: 0.95,
                    provenance: format!("{} approval agent", current),
                };

                return AgentEffect::with_proposal(proposal);
            }
        }

        AgentEffect::default()
    }
}

fn main() {
    println!("=== Expense Approval Workflow Example ===\n");

    let mut engine = Engine::new();

    engine.register(ExpenseParsingAgent);
    engine.register(PolicyValidationAgent);
    engine.register(ApprovalRoutingAgent);
    engine.register(ApprovalSimulationAgent);

    let hitl_policy = EngineHitlPolicy {
        confidence_threshold: Some(0.8),
        gated_keys: vec![ContextKey::Proposals],
        timeout: TimeoutPolicy {
            timeout_secs: 300,
            action: TimeoutAction::Reject,
        },
    };
    engine.set_hitl_policy(hitl_policy);

    let expense = serde_json::json!({
        "employee": "john.doe@example.com",
        "amount": 600.00,
        "category": "entertainment",
        "description": "Client dinner",
        "date": "2026-04-15"
    });

    let mut ctx = Context::new();
    let _ = ctx.add_fact(Fact {
        key: ContextKey::Seeds,
        id: "expense-1".to_string(),
        content: expense.to_string(),
    });

    println!(
        "Expense submitted: ${} {} - {}\n",
        expense["amount"], expense["category"], expense["description"]
    );
    println!("Running approval workflow...\n");

    match engine.run_with_hitl(ctx) {
        RunResult::HitlPause(pause) => {
            println!("⏸️  HITL Gate: Approval Required");
            println!("    Proposal: {}", pause.request.summary);
            println!(
                "    Approver: {}",
                pause.request.rationale.as_deref().unwrap_or("manager")
            );
            println!();

            let decision =
                GateDecision::approve(pause.request.gate_id.clone(), "manager@company.com");

            println!("▶️  Manager approved. Resuming workflow...\n");

            match engine.resume(*pause, decision) {
                RunResult::HitlPause(pause2) => {
                    println!("⏸️  HITL Gate: Finance Approval Required");
                    let decision2 = GateDecision::approve(
                        pause2.request.gate_id.clone(),
                        "finance@company.com",
                    );
                    println!("▶️  Finance approved. Resuming...\n");

                    match engine.resume(*pause2, decision2) {
                        RunResult::Complete(Ok(result)) => {
                            println!("✅ Expense Approved!\n");
                            for fact in result.context.get(ContextKey::Proposals) {
                                println!("  {}", fact.content);
                            }
                        }
                        _ => println!("❌ Final approval failed"),
                    }
                }
                RunResult::Complete(Ok(result)) => {
                    println!("✅ Expense Approved!\n");
                    for fact in result.context.get(ContextKey::Proposals) {
                        println!("  {}", fact.content);
                    }
                }
                _ => println!("❌ Approval workflow failed"),
            }
        }
        RunResult::Complete(Ok(result)) => {
            println!("✅ Expense Approved (no HITL needed)!\n");
            for fact in result.context.get(ContextKey::Proposals) {
                println!("  {}", fact.content);
            }
        }
        RunResult::Complete(Err(e)) => {
            println!("❌ Workflow failed: {e}");
        }
    }

    println!("\n=== Done ===");
}
