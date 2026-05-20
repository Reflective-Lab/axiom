---
tags: [philosophy, strategy, vertical-ai, jtbd]
source: llm
---

# Composed Vertical AI

Vertical AI should be framed as a solution stack wrapped in a Jobs-to-be-Done
narrative, not as an LLM wrapper around a professional workflow.

The important product question is not "which model should answer this prompt?"
It is "what kind of work is this job asking the system to perform, what evidence
does that work require, what authority does it need, and which expert component is
best suited to each subproblem?"

## Thesis

LLMs are strong at semantic compression: reading, drafting, summarizing,
comparing clauses, explaining decisions, and turning messy language into
structured candidates. Many early vertical AI products are built mostly around
that capability.

That can be valuable, but it is also easy to copy. The more durable product
surface is the part of the job where language is only one layer above a harder
system:

| Work class | Better core system | LLM role |
|---|---|---|
| Constraint-heavy structure search | SAT, SMT, MILP, optimization | Explain options, propose hypotheses, translate tradeoffs |
| Exact financial or ownership modeling | Symbolic engines, spreadsheets, formal models | Narrate scenarios, catch missing assumptions |
| Regulatory and policy state spaces | Rule engines, SMT, policy evaluators | Summarize rules, draft risk memos |
| Anomaly and risk detection | Statistics, ML, graph analytics | Explain signals and ask follow-up questions |
| Relationship, control, and dependency reasoning | Graph systems | Label, summarize, and inspect graph neighborhoods |
| Behavior under uncertainty | Probabilistic models, simulations, game theory | Convert simulation output into operator language |
| Long-running process coordination | Workflow engines, ledgers, schedulers | Draft updates and surface missing evidence |
| Accountability and strategic ambiguity | Humans, institutions, operator gates | Prepare options without taking authority |

The product moat is composition, provenance, replay, authority boundaries, and
domain workflow fit. Model choice matters, but it is not the architecture.

## JTBD Envelope

The user should not have to think in terms of LLM plus solver plus graph plus
workflow. The application presents the job narrative:

- "Release escrow only when the agreed conditions are actually satisfied."
- "Compare portfolio risk language across comparable time windows."
- "Qualify an inbound account without taking customer-visible action before
  consent and approval are present."
- "Find a deal structure that preserves incentives while staying inside tax,
  regulatory, financing, and governance constraints."

Inside that envelope, the stack decomposes the job by computational nature.
Axiom should express the truth conditions, evidence requirements, verifier
expectations, concerns, and learning signals. Helm should surface operator
control, HITL review, missing evidence, and append-only receipts. Organism should
select and run formations. Mosaic extensions should provide specialized
Suggestors and analyzers. Converge should promote facts, integrity, stop reasons,
and authority-bound outcomes. The app should remain the domain UX and adapter
owner.

## Example: M&A Deal Architecture

An M&A lawyer does not only write contracts. High-end deal work coordinates
uncertainty across people, incentives, power, timing, law, data, and institutional
accountability.

The composed stack for a deal architecture product could look like this:

| Layer | Responsibility |
|---|---|
| LLM | Draft SPA, APA, NDA, diligence summaries, redlines, research memos, and negotiation language |
| Constraint solver | Search acquisition structures under tax, regulatory, debt, IP, employment, export-control, and antitrust constraints |
| Symbolic financial model | Simulate SAFEs, preferences, liquidation waterfalls, option pools, convertibles, pro rata rights, and earn-outs |
| Policy/rule evaluator | Check CFIUS, export controls, AI governance, data sovereignty, securities, and employment constraints |
| Graph system | Model ownership, control, subsidiaries, IP flow, data flow, diligence dependencies, and board approvals |
| Statistical/ML system | Detect revenue, transfer-pricing, concentration, compensation, fraud, or liability anomalies |
| Probabilistic simulation | Explore litigation, regulatory, negotiation, and counterparty behavior risks |
| Workflow and ledger | Coordinate signoffs, diligence tracks, bankers, board packets, deadlines, and review history |
| Human operator | Own trust arbitration, strategic ambiguity, opinion letters, regulator contact, and accountability |

Axiom's role in this product is not to become the M&A app. Its role is to make
the job's truth conditions and verifier boundaries explicit enough that the app,
Helm, Organism, Mosaic, and Converge can cooperate without smuggling authority
across layers.

## Design Principles

1. Start from the job narrative, not the model category.
2. Split the job by computational nature before choosing implementation tools.
3. Keep evidence, provenance, and authority explicit at every boundary.
4. Preserve disagreement when disagreement is part of the truth.
5. Treat human accountability as a first-class system component.
6. Prefer replayable receipts and verifier reports over opaque chat history.
7. Avoid app-specific wrappers when a repeated contract surface is emerging.

## Product Implication

The app probes are not only demos. They are a search process for repeated
surfaces:

- app-neutral adapter receipts
- Helm-owned readiness packets
- operator decision, approval, action, outcome, and temporal-evidence receipts
- Axiom truth packages, verifier reports, concerns, and calibration records
- Organism formation boundaries
- Mosaic expert-module boundaries
- Converge promotion and integrity boundaries

That is the path from individual vertical apps toward a reusable operating
contract for composed vertical AI.
