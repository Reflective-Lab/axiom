---
tags: [stack, optimization]
source: mixed
---
# OR-Tools

Google OR-Tools provides constraint programming and optimization capabilities through `converge-optimization`.

## Role in Converge

Implements optimization [[Architecture/Ports|ports]] for agents that need constraint satisfaction, scheduling, or resource allocation.

## FFI Architecture

```
converge-optimization
    └── ortools-sys (FFI bindings)
            └── C++ wrapper
                    └── Google OR-Tools native library
```

`ortools-sys` compiles a custom C++ wrapper at build time and links against the OR-Tools native library.

## Capabilities

| Solver | Use Case |
|---|---|
| CP-SAT | Constraint programming, scheduling |
| Linear/Integer Programming | Resource allocation, budget optimization |
| Multi-criteria optimization | Weighted scoring across dimensions |

## Agent Example

An optimization agent reads constraints and signals from the context, formulates a CP-SAT problem, solves it, and proposes the solution as a `ProposedFact`. The engine's governance gate validates the solution before it becomes a fact.

The solver produces observations, not decisions ([[Philosophy/Nine Axioms#4. Agents Suggest, Engine Decides|Axiom 4]]).

See also: [[Architecture/Providers]], [[Concepts/Agents]]
