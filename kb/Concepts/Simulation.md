---
tags: [concepts]
source: llm
---

# Simulation

The simulation module performs pre-flight analysis on truth files — detecting problems before execution, without running the Converge engine.

## Verdict

Every simulation produces one of three verdicts:

| Verdict | Meaning |
|---|---|
| **Ready** | Spec is well-formed and likely to converge |
| **Risky** | Spec has gaps that may cause issues |
| **WillNotConverge** | Spec has fundamental problems that will prevent convergence |

## Checks

### Governance Coverage

Are the governance blocks complete? Missing Intent, incomplete Authority, absent Evidence requirements are flagged.

### Scenario Analysis

Are there enough scenarios with enough assertions? Thin specs produce thin invariants.

### Resource Availability

Are all referenced context keys and resources declared? Missing resources mean the invariant will fail at runtime.

### Domain Profiles

Core simulation is domain-neutral by default. Downstream layers can opt into
domain-specific profiles when they want richer pre-flight checks for a known
business class.

Current profile:

| Profile | What it checks | Owner |
|---|---|---|
| `DomainProfile::VendorSelection` | vendor/procurement coverage: evaluation dimensions, candidate references, ranking criteria, and approval gates | downstream Organism/Helms truth workflows |

This keeps Axiom responsible for Truth completeness while letting richer
vendor, procurement, and diligence behavior become real Organism Suggestors or
Helms Truth bindings once more context is available.

## Output

`SimulationReport` contains:
- `verdict` — Ready / Risky / WillNotConverge
- `findings` — individual issues with severity
- `governance_coverage` — governance block completeness
- `resource_summary` — declared vs referenced resources
- `domain_profiles` — optional profile reports, empty unless enabled in `SimulationConfig`
