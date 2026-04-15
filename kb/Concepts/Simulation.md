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

## Output

`SimulationReport` contains:
- `verdict` — Ready / Risky / WillNotConverge
- `findings` — individual issues with severity
- `coverage` — governance block completeness
- `resource_summary` — declared vs referenced resources
