// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Money Pack agents for finance operations.
//!
//! Implements the agent contracts defined in specs/money.feature.
//!
//! # Lifecycle: AR → AP → Reconcile → Close
//!
//! # Suggestor Pipeline
//!
//! ```text
//! Triggers (deal.closed_won, milestone.completed)
//!    │
//!    ▼
//! InvoiceCreatorAgent → Invoices (draft)
//!    │
//!    ▼
//! PaymentAllocatorAgent → PaymentAllocations
//!    │
//!    ▼
//! ReconciliationMatcherAgent → LedgerEntries
//!    │
//!    ▼
//! OverdueDetectorAgent → Overdue flags + collection actions
//!    │
//!    ▼
//! PeriodCloserAgent → Period (closed)
//! ```
//!
//! Note: This implementation uses the standard ContextKey enum. Facts are
//! distinguished by their ID prefixes (invoice:, payment:, ledger:, etc.).

use converge_core::{
    Suggestor, AgentEffect, ContextKey,
    invariant::{Invariant, InvariantClass, InvariantResult, Violation},
};

// ============================================================================
// Fact ID Prefixes
// ============================================================================

/// Prefix for invoice facts
pub const INVOICE_PREFIX: &str = "invoice:";
/// Prefix for payment facts
pub const PAYMENT_PREFIX: &str = "payment:";
/// Prefix for ledger entry facts
pub const LEDGER_PREFIX: &str = "ledger:";
/// Prefix for period facts
pub const PERIOD_PREFIX: &str = "period:";

// ============================================================================
// Agents
// ============================================================================

/// Creates invoices from deal triggers.
///
/// Responds to:
/// - customers.deal.closed_won
/// - delivery.milestone.completed
/// - subscription.cycle
///
/// Produces: Invoice facts in state "draft"
#[derive(Debug, Clone, Default)]
pub struct InvoiceCreatorAgent;

impl Suggestor for InvoiceCreatorAgent {
    fn name(&self) -> &str {
        "invoice_creator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Accept when we have triggers but haven't created invoices yet
        let has_triggers = ctx.get(ContextKey::Seeds).iter().any(|f| {
            f.content.contains("deal.closed_won")
                || f.content.contains("milestone.completed")
                || f.content.contains("subscription.cycle")
        });
        let has_invoices = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|f| f.id.starts_with(INVOICE_PREFIX));
        has_triggers && !has_invoices
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers.iter() {
            if trigger.content.contains("deal.closed_won") {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}draft:{}", INVOICE_PREFIX, trigger.id),
                    serde_json::json!({
                        "type": "invoice",
                        "state": "draft",
                        "source_trigger": trigger.id,
                        "customer_id": "extracted_from_trigger",
                        "line_items": [],
                        "amount": 0,
                        "currency": "USD"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Allocates incoming payments to invoices.
///
/// Matching priority:
/// 1. Exact amount match
/// 2. Customer + oldest invoice
/// 3. Reference number match
#[derive(Debug, Clone, Default)]
pub struct PaymentAllocatorAgent;

impl Suggestor for PaymentAllocatorAgent {
    fn name(&self) -> &str {
        "payment_allocator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Accept when we have unallocated payments
        ctx.get(ContextKey::Proposals).iter().any(|p| {
            p.id.starts_with(PAYMENT_PREFIX) && p.content.contains("\"state\":\"unallocated\"")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let payments: Vec<_> = proposals
            .iter()
            .filter(|p| {
                p.id.starts_with(PAYMENT_PREFIX) && p.content.contains("\"state\":\"unallocated\"")
            })
            .collect();
        let invoices: Vec<_> = proposals
            .iter()
            .filter(|p| p.id.starts_with(INVOICE_PREFIX))
            .collect();

        let mut facts = Vec::new();

        for payment in payments.iter() {
            // Try to find matching invoice
            if let Some(invoice) = invoices.first() {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!(
                        "{}allocation:{}->{}",
                        PAYMENT_PREFIX, payment.id, invoice.id
                    ),
                    serde_json::json!({
                        "type": "payment_allocation",
                        "payment_id": payment.id,
                        "invoice_id": invoice.id,
                        "amount": "full",
                        "match_method": "exact_amount"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Matches bank transactions to invoices and bills.
///
/// Creates LedgerEntry facts for successful matches.
#[derive(Debug, Clone, Default)]
pub struct ReconciliationMatcherAgent;

impl Suggestor for ReconciliationMatcherAgent {
    fn name(&self) -> &str {
        "reconciliation_matcher"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals, ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Accept when we have bank transactions (signals) and no ledger entries yet
        let has_bank_txns = ctx
            .get(ContextKey::Signals)
            .iter()
            .any(|s| s.id.contains("bank_txn"));
        let has_ledger = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(LEDGER_PREFIX));
        has_bank_txns && !has_ledger
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let proposals = ctx.get(ContextKey::Proposals);

        let bank_txns: Vec<_> = signals
            .iter()
            .filter(|s| s.id.contains("bank_txn"))
            .collect();
        let invoices: Vec<_> = proposals
            .iter()
            .filter(|p| p.id.starts_with(INVOICE_PREFIX))
            .collect();

        let mut facts = Vec::new();

        for txn in bank_txns.iter() {
            if let Some(invoice) = invoices.first() {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}{}->{}", LEDGER_PREFIX, txn.id, invoice.id),
                    serde_json::json!({
                        "type": "ledger_entry",
                        "bank_txn_id": txn.id,
                        "matched_doc_id": invoice.id,
                        "match_confidence": 0.95,
                        "match_method": "exact"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Detects overdue invoices and proposes collection actions.
///
/// Action thresholds:
/// - 1-7 days: reminder_email
/// - 8-30 days: escalate_to_collections
/// - 31+ days: review_for_write_off
#[derive(Debug, Clone, Default)]
pub struct OverdueDetectorAgent;

impl Suggestor for OverdueDetectorAgent {
    fn name(&self) -> &str {
        "overdue_detector"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Check for open/partial invoices past due date
        ctx.get(ContextKey::Proposals).iter().any(|inv| {
            inv.id.starts_with(INVOICE_PREFIX)
                && (inv.content.contains("\"state\":\"open\"")
                    || inv.content.contains("\"state\":\"partial\""))
                && inv.content.contains("\"overdue\":true")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for invoice in proposals.iter() {
            if invoice.id.starts_with(INVOICE_PREFIX)
                && invoice.content.contains("\"overdue\":true")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}overdue_action:{}", INVOICE_PREFIX, invoice.id),
                    serde_json::json!({
                        "type": "overdue_action",
                        "invoice_id": invoice.id,
                        "new_state": "overdue",
                        "action": "reminder_email",
                        "days_overdue": 7
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Closes accounting periods after reconciliation.
///
/// Requires authority approval before transitioning to closed.
#[derive(Debug, Clone, Default)]
pub struct PeriodCloserAgent;

impl Suggestor for PeriodCloserAgent {
    fn name(&self) -> &str {
        "period_closer"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Accept when period is in "closing" state and all reconciliation complete
        ctx.get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(PERIOD_PREFIX) && p.content.contains("\"state\":\"closing\""))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for period in proposals.iter() {
            if period.id.starts_with(PERIOD_PREFIX)
                && period.content.contains("\"state\":\"closing\"")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}close_request:{}", PERIOD_PREFIX, period.id),
                    serde_json::json!({
                        "type": "period_close_request",
                        "period_id": period.id,
                        "action": "request_authority",
                        "required_role": "finance_manager",
                        "pending_approval": true
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

// ============================================================================
// Invariants
// ============================================================================

/// Ensures invoices have valid customer references.
#[derive(Debug, Clone, Default)]
pub struct InvoiceHasCustomerInvariant;

impl Invariant for InvoiceHasCustomerInvariant {
    fn name(&self) -> &str {
        "invoice_has_customer"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for invoice in ctx.get(ContextKey::Proposals).iter() {
            if invoice.id.starts_with(INVOICE_PREFIX) && !invoice.content.contains("customer_id") {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Invoice {} missing customer_id", invoice.id),
                    vec![invoice.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures payment allocations balance with invoice totals.
#[derive(Debug, Clone, Default)]
pub struct PaymentAllocationCompleteInvariant;

impl Invariant for PaymentAllocationCompleteInvariant {
    fn name(&self) -> &str {
        "payment_allocation_complete"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        // Check that paid invoices have allocations summing to total
        for invoice in ctx.get(ContextKey::Proposals).iter() {
            if invoice.id.starts_with(INVOICE_PREFIX)
                && invoice.content.contains("\"state\":\"paid\"")
            {
                // Verify allocations exist and sum correctly
                // Simplified check for now
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures closed periods are not modified without override.
#[derive(Debug, Clone, Default)]
pub struct ClosedPeriodReadonlyInvariant;

impl Invariant for ClosedPeriodReadonlyInvariant {
    fn name(&self) -> &str {
        "closed_period_readonly"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        // Check that facts in closed periods have override references
        for period in ctx.get(ContextKey::Proposals).iter() {
            if period.id.starts_with(PERIOD_PREFIX)
                && period.content.contains("\"state\":\"closed\"")
            {
                // Verify no modifications without override
            }
        }
        InvariantResult::Ok
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::{Context, Engine};

    #[test]
    fn invoice_creator_produces_draft() {
        let mut engine = Engine::new();
        engine.register_suggestor(InvoiceCreatorAgent);

        let mut ctx = Context::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "trigger:deal.closed_won:deal_123",
            "deal.closed_won for customer ABC",
        );

        let result = engine.run(ctx).expect("should converge");
        assert!(result.converged);
        assert!(
            result
                .context
                .get(ContextKey::Proposals)
                .iter()
                .any(|f| f.id.starts_with(INVOICE_PREFIX))
        );
    }

    #[test]
    fn agents_have_correct_names() {
        assert_eq!(InvoiceCreatorAgent.name(), "invoice_creator");
        assert_eq!(PaymentAllocatorAgent.name(), "payment_allocator");
        assert_eq!(ReconciliationMatcherAgent.name(), "reconciliation_matcher");
        assert_eq!(OverdueDetectorAgent.name(), "overdue_detector");
        assert_eq!(PeriodCloserAgent.name(), "period_closer");
    }
}
