// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Vendor Selection — swarm evaluation with consensus and HITL gates.
//!
//! Demonstrates: swarms, consensus/aggregation, multi-criteria scoring.

use converge_core::{
    Agent, AgentEffect, Context, ContextKey, Engine, EngineHitlPolicy, Fact, ProposedFact,
    RunResult,
    gates::hitl::GateDecision,
    gates::{TimeoutAction, TimeoutPolicy},
};

struct VendorDataAgent;

impl Agent for VendorDataAgent {
    fn name(&self) -> &str {
        "VendorDataAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Seeds) && !ctx.has(ContextKey::Signals)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let seed = seeds.first();

        let vendors = if let Some(s) = seed {
            let json: serde_json::Value = serde_json::from_str(&s.content).unwrap_or_default();
            json.get("vendors").cloned().unwrap_or_default()
        } else {
            serde_json::json!([])
        };

        let mut facts = Vec::new();
        for vendor in vendors
            .as_array()
            .map_or(&[] as &[serde_json::Value], |v| v)
        {
            facts.push(Fact {
                key: ContextKey::Signals,
                id: format!(
                    "vendor-{}",
                    vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?")
                ),
                content: vendor.to_string(),
            });
        }

        AgentEffect::with_facts(facts)
    }
}

struct PriceEvaluatorAgent;

impl Agent for PriceEvaluatorAgent {
    fn name(&self) -> &str {
        "PriceEvaluatorAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            if let Ok(vendor) = serde_json::from_str::<serde_json::Value>(&signal.content) {
                let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let price: f64 = vendor
                    .get("price")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(999999.0);

                let score = if price < 10000.0 {
                    1.0
                } else if price < 25000.0 {
                    0.7
                } else if price < 50000.0 {
                    0.4
                } else {
                    0.1
                };

                evaluations.push(Fact {
                    key: ContextKey::Evaluations,
                    id: format!("price:{}", id),
                    content: serde_json::json!({
                        "vendor_id": id,
                        "criterion": "price",
                        "score": score,
                        "raw_value": price
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(evaluations)
    }
}

struct ComplianceEvaluatorAgent;

impl Agent for ComplianceEvaluatorAgent {
    fn name(&self) -> &str {
        "ComplianceEvaluatorAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            if let Ok(vendor) = serde_json::from_str::<serde_json::Value>(&signal.content) {
                let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let compliant: bool = vendor
                    .get("compliant")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let score = if compliant { 1.0 } else { 0.0 };

                evaluations.push(Fact {
                    key: ContextKey::Evaluations,
                    id: format!("compliance:{}", id),
                    content: serde_json::json!({
                        "vendor_id": id,
                        "criterion": "compliance",
                        "score": score,
                        "raw_value": compliant
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(evaluations)
    }
}

struct RiskEvaluatorAgent;

impl Agent for RiskEvaluatorAgent {
    fn name(&self) -> &str {
        "RiskEvaluatorAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            if let Ok(vendor) = serde_json::from_str::<serde_json::Value>(&signal.content) {
                let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let years: u32 = vendor
                    .get("years_in_business")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let score = if years > 10 {
                    1.0
                } else if years > 5 {
                    0.7
                } else if years > 2 {
                    0.4
                } else {
                    0.1
                };

                evaluations.push(Fact {
                    key: ContextKey::Evaluations,
                    id: format!("risk:{}", id),
                    content: serde_json::json!({
                        "vendor_id": id,
                        "criterion": "risk",
                        "score": score,
                        "raw_value": years
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(evaluations)
    }
}

struct TimelineEvaluatorAgent;

impl Agent for TimelineEvaluatorAgent {
    fn name(&self) -> &str {
        "TimelineEvaluatorAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            if let Ok(vendor) = serde_json::from_str::<serde_json::Value>(&signal.content) {
                let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let weeks: u32 = vendor
                    .get("delivery_weeks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(52) as u32;

                let score = if weeks <= 4 {
                    1.0
                } else if weeks <= 8 {
                    0.8
                } else if weeks <= 12 {
                    0.5
                } else {
                    0.2
                };

                evaluations.push(Fact {
                    key: ContextKey::Evaluations,
                    id: format!("timeline:{}", id),
                    content: serde_json::json!({
                        "vendor_id": id,
                        "criterion": "timeline",
                        "score": score,
                        "raw_value": weeks
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(evaluations)
    }
}

struct ConsensusAgent;

impl Agent for ConsensusAgent {
    fn name(&self) -> &str {
        "ConsensusAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Evaluations) && !ctx.has(ContextKey::Proposals)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let evaluations = ctx.get(ContextKey::Evaluations);

        let mut vendor_scores: std::collections::HashMap<String, (f64, u32)> =
            std::collections::HashMap::new();

        for eval in evaluations {
            if let Ok(eval_json) = serde_json::from_str::<serde_json::Value>(&eval.content) {
                let vendor_id = eval_json
                    .get("vendor_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let score: f64 = eval_json
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let entry = vendor_scores
                    .entry(vendor_id.to_string())
                    .or_insert((0.0, 0));
                entry.0 += score;
                entry.1 += 1;
            }
        }

        let weights = serde_json::json!({
            "price": 0.30,
            "compliance": 0.25,
            "risk": 0.20,
            "timeline": 0.15,
            "quality": 0.10
        });

        let mut weighted_scores: Vec<(String, f64)> = Vec::new();
        for (vendor_id, (total_score, count)) in vendor_scores {
            let avg_score = total_score / count as f64;
            weighted_scores.push((vendor_id, avg_score));
        }

        weighted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let proposals: Vec<ProposedFact> = weighted_scores
            .iter()
            .enumerate()
            .map(|(i, (vendor_id, score))| ProposedFact {
                key: ContextKey::Proposals,
                id: format!("recommendation-{}", i + 1),
                content: serde_json::json!({
                    "vendor_id": vendor_id,
                    "rank": i + 1,
                    "score": score,
                    "recommendation": if i == 0 { "recommended" } else { "alternative" }
                })
                .to_string(),
                confidence: if i == 0 { 0.85 } else { 0.6 },
                provenance: "consensus-agent".to_string(),
            })
            .collect();

        AgentEffect {
            facts: Vec::new(),
            proposals,
        }
    }
}

fn main() {
    println!("=== Vendor Selection Example ===\n");

    let mut engine = Engine::new();

    engine.register(VendorDataAgent);
    engine.register(PriceEvaluatorAgent);
    engine.register(ComplianceEvaluatorAgent);
    engine.register(RiskEvaluatorAgent);
    engine.register(TimelineEvaluatorAgent);
    engine.register(ConsensusAgent);

    let hitl_policy = EngineHitlPolicy {
        confidence_threshold: Some(0.75),
        gated_keys: vec![ContextKey::Proposals],
        timeout: TimeoutPolicy {
            timeout_secs: 300,
            action: TimeoutAction::Reject,
        },
    };
    engine.set_hitl_policy(hitl_policy);

    let rfp = serde_json::json!({
        "vendors": [
            {
                "id": "vendor-a",
                "name": "Acme Corp",
                "price": 15000,
                "compliant": true,
                "years_in_business": 15,
                "delivery_weeks": 6
            },
            {
                "id": "vendor-b",
                "name": "Beta Solutions",
                "price": 22000,
                "compliant": true,
                "years_in_business": 8,
                "delivery_weeks": 4
            },
            {
                "id": "vendor-c",
                "name": "Gamma Industries",
                "price": 8000,
                "compliant": false,
                "years_in_business": 3,
                "delivery_weeks": 10
            }
        ]
    });

    let mut ctx = Context::new();
    let _ = ctx.add_fact(Fact {
        key: ContextKey::Seeds,
        id: "rfp-1".to_string(),
        content: rfp.to_string(),
    });

    println!("Evaluating 3 vendors with swarm of 5 agents...\n");

    match engine.run_with_hitl(ctx) {
        RunResult::HitlPause(pause) => {
            println!("⏸️  HITL Gate: Procurement Approval Required");
            println!("    Recommendation: {}", pause.request.summary);
            println!();

            if let Ok(proposal) = serde_json::from_str::<serde_json::Value>(&pause.request.summary)
            {
                if let Some(vendor) = proposal.get("vendor_id").and_then(|v| v.as_str()) {
                    println!("    Top vendor: {}", vendor);
                }
            }

            let decision =
                GateDecision::approve(pause.request.gate_id.clone(), "procurement@company.com");

            println!("▶️  Approved by procurement. Finalizing...\n");

            match engine.resume(*pause, decision) {
                RunResult::Complete(Ok(result)) => {
                    println!("✅ Vendor Selected!\n");
                    for fact in result.context.get(ContextKey::Proposals) {
                        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&fact.content) {
                            let rank = p.get("rank").and_then(|v| v.as_u64()).unwrap_or(0);
                            let vendor = p.get("vendor_id").and_then(|v| v.as_str()).unwrap_or("?");
                            let score = p.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let rec = p
                                .get("recommendation")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            println!("  #{}. {} (score: {:.2}) - {}", rank, vendor, score, rec);
                        }
                    }
                }
                _ => println!("❌ Selection failed"),
            }
        }
        RunResult::Complete(Ok(result)) => {
            println!("✅ Vendor Selected!\n");
            for fact in result.context.get(ContextKey::Proposals) {
                if let Ok(p) = serde_json::from_str::<serde_json::Value>(&fact.content) {
                    let rank = p.get("rank").and_then(|v| v.as_u64()).unwrap_or(0);
                    let vendor = p.get("vendor_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let score = p.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let rec = p
                        .get("recommendation")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    println!("  #{}. {} (score: {:.2}) - {}", rank, vendor, score, rec);
                }
            }
        }
        RunResult::Complete(Err(e)) => {
            println!("❌ Selection failed: {e}");
        }
    }

    println!("\n=== Done ===");
}
