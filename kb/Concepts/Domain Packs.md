---
tags: [concepts]
source: mixed
---
# Domain Packs

Pre-built suggestor collections from `converge-domain`. Register and run — no custom suggestors needed for cross-cutting concerns.

## Available Packs

| Pack | Purpose | Key Suggestors |
|---|---|---|
| `trust` | Audit, access control, provenance, compliance | SessionValidatorAgent, RbacEnforcerAgent, AuditWriterAgent, ProvenanceTrackerAgent, ComplianceScannerAgent, ViolationRemediatorAgent, PiiRedactorAgent |
| `money` | Financial transactions, invoicing, reconciliation | InvoiceCreatorAgent, PaymentAllocatorAgent, ReconciliationMatcherAgent, PeriodCloserAgent |
| `delivery` | Promise fulfillment, scope tracking, blockers | PromiseCreatorAgent, WorkBreakdownAgent, BlockerDetectorAgent, RiskAssessorAgent, StatusAggregatorAgent |
| `knowledge` | Signal capture, hypothesis testing, canonical decisions | SignalCaptureAgent, HypothesisGeneratorAgent, ExperimentRunnerAgent, DecisionMemoAgent, CanonicalKnowledgeAgent |
| `data_metrics` | Metrics, dashboards, anomaly detection, alerting | MetricRegistrarAgent, DataValidatorAgent, DashboardBuilderAgent, AnomalyDetectorAgent, AlertEvaluatorAgent |

## Usage

```rust
let mut engine = Engine::new();
engine.register_suggestor_in_pack("trust-pack", AuditWriterAgent);
engine.register_suggestor_in_pack("trust-pack", ProvenanceTrackerAgent);
```

## Mixing with Custom Suggestors

Domain packs handle cross-cutting concerns. Your custom suggestors handle business logic. Both run in the same engine under the same governance.

```rust
// Business logic
engine.register_suggestor_in_pack("evaluation-pack", ComplianceScreenerAgent { .. });
engine.register_suggestor_in_pack("evaluation-pack", CostAnalysisAgent);

// Governance (free audit trails)
engine.register_suggestor_in_pack("trust-pack", AuditWriterAgent);
engine.register_suggestor_in_pack("trust-pack", ProvenanceTrackerAgent);
```

## Invariants

Domain packs ship their own [[Concepts/Invariants|invariants]]:

- `AllActionsAuditedInvariant`
- `AuditImmutabilityInvariant`
- `ViolationsHaveRemediationInvariant`

See also: [[Concepts/Agents]], [[Building/Crate Catalog]]
