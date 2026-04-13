---
tags: [moc, philosophy]
source: mixed
---
# Ecosystem — The Helicopter View

Converge is the foundation of a three-layer stack built by Reflective Labs. Understanding where Converge sits — and what it does NOT do — is essential for keeping the architecture clean.

## The Three Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    LAYER 3: PRODUCTS                         │
│                                                              │
│   Wolfgang          SaaS Killer           [Future Apps]      │
│   (wolfgang.bot)    (crm.prio.ai)                            │
│                                                              │
│   AI workspace      JTBD-driven CRM      Your product       │
│   Expert debate     Revenue substrate     on Converge        │
│   Knowledge RAG     20 capability modules                    │
│                     9 executable truths                      │
├──────────────────────────┬──────────────────────────────────┤
│                          │ runs on                           │
│               LAYER 2: ORGANISM                              │
│               (organism.zone)                                │
│                                                              │
│   Organizational Intelligence Runtime                        │
│                                                              │
│   Intent interpretation    Adversarial governance            │
│   Multi-model planning     Simulation swarm                  │
│   Huddle + debate loops    Organizational learning           │
│                                                              │
│   Translates human goals → governed, debated, simulated      │
│   plans → submits to Converge commit boundary                │
├──────────────────────────┬──────────────────────────────────┤
│                          │ commits through                   │
│               LAYER 1: CONVERGE                              │
│               (converge.zone)                                │
│                                                              │
│   [[Philosophy/Nine Axioms|9 Axioms]]                        │
│   Agents · Context · Facts · Promotion Gate                  │
│   Invariants · Budgets · HITL · Convergence                  │
│                                                              │
│   Owns: authority, truth, governance, convergence            │
│   Does NOT own: reasoning, planning, UX, business logic      │
└──────────────────────────────────────────────────────────────┘
```

## What Each Layer Owns

### Converge (Layer 1) — The Commit Boundary

**Owns:**
- The [[Philosophy/Nine Axioms|nine axioms]] — non-negotiable
- The [[Concepts/Proposals and Promotion|promotion gate]] — the only path from proposal to fact
- The [[Architecture/Engine Execution Model|convergence engine]] — 8-phase execution cycle
- [[Concepts/Context and Facts|Context, Facts, ProposedFacts]] — the type system
- [[Concepts/Invariants|Invariants]] — executable guarantees
- [[Concepts/Domain Packs|Domain packs]] — trust, money, delivery, knowledge, data_metrics
- [[Architecture/Ports|Ports]] — the trait boundaries
- [[Architecture/Providers|Providers]] — LLM, storage, search, optimization adapters
- Traceability and audit

**Does NOT own:**
- How plans are made (that's Organism)
- What the user sees (that's the product layer)
- Business domain logic (that's the product layer)
- Whether a plan is good enough (that's Organism's adversarial review)

### Organism (Layer 2) — The Thinking Layer

**Owns:**
- Intent interpretation — decomposing human goals into machine specs
- Multi-model collaborative planning (huddle loop)
- Adversarial governance — assumption breakers, constraint checkers, skeptics
- Simulation — outcome/cost/policy/causal stress testing
- Organizational learning — calibrating planning priors from execution outcomes

**Does NOT own:**
- Authority — recomputed at Converge's commit boundary, never inherited from reasoning
- The convergence engine — Organism submits to Converge, doesn't replace it
- Product UX — that's the product layer

**Critical invariant:** No plan reaches the Converge commit boundary without passing BOTH adversarial review AND simulation. Authority is never inherited from reasoning.

### Products (Layer 3) — What Users Touch

**Wolfgang** (wolfgang.bot)
- AI workspace with knowledge-grounded expert discussion
- Professor Wolfgang persona — contrarian, challenges your thinking
- Desktop (Tauri + Svelte) and web (SvelteKit + Firebase)
- Uses: converge-provider (LLM routing), converge-storage (GCS), converge-runtime (auth)
- RAG over curated knowledge bases via LanceDB + OpenAI embeddings

**SaaS Killer** (crm.prio.ai)
- JTBD-driven CRM/ERP substrate — job-centric, not record-centric
- 20 capability modules across 7 suites (Foundation, Relationship, Commercial, Revenue, Work, Trust, Intelligence)
- 9 executable truths that compile to Converge intent packets
- Each truth maps to a `TypesRootIntent` via `TruthConvergeBinding`
- Desktop (Tauri + Svelte) with live convergence visibility
- Uses: converge-core directly (proposals, facts, promotion gates)

## The Flow

```
Human intent
    ↓
Organism decomposes → plans → debates → simulates
    ↓
Converge receives intent packet → runs agents → converges → promotes facts
    ↓
Product projects converged facts into domain records → shows results to user
```

Products never bypass Organism to reach Converge directly for complex decisions. Simple CRUD doesn't need convergence. Governed decisions always go through the full stack.

## The Dependency Rule

```
Products depend on Organism and/or Converge
Organism depends on Converge
Converge depends on nothing above it
```

This is the hexagonal architecture applied at the organizational level. Converge defines [[Architecture/Ports|ports]]. Organism and products are adapters. Converge never imports from them.

When a product needs a capability that doesn't exist:
1. If it's a governance/convergence capability → build it in **Converge**
2. If it's a reasoning/planning capability → build it in **Organism**
3. If it's product-specific → build it in the **product**

Never work around Converge. Patch it.

## Domains and Branding

| Layer | Brand | Domain | Repo |
|---|---|---|---|
| Foundation | Converge | converge.zone | Reflective-Lab/converge |
| Intelligence | Organism | organism.zone | Reflective-Labs/organism.zone |
| Product | Wolfgang | wolfgang.bot | Reflective-Labs/wolfgang-app |
| Product | SaaS Killer | crm.prio.ai | saas-killer |
| Company | Reflective Labs | reflective.se | — |

See also: [[Philosophy/Why Converge]], [[Philosophy/Nine Axioms]], [[Architecture/Hexagonal Architecture]]
