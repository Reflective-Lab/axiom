# Converge Axioms

The 9 axioms that define Converge's system guarantees. Every content claim must reference these.

1. **Monotonicity** — Context only grows; facts are never retracted
2. **Determinism** — Same inputs produce same outputs (within a single convergence run)
3. **Idempotency** — Running an agent twice on same context produces same proposals
4. **Commutativity** — Agent execution order does not affect the converged result
5. **Termination** — Every convergence run reaches a fixed point or exhausts budget
6. **Consistency** — No invariant is ever violated in the final converged state
7. **Starvation freedom** — Every agent gets to propose in every cycle
8. **Confluence** — Different execution paths converge to the same fixed point
9. **Observability** — Every state transition is logged and auditable

## Open Questions (to verify with Eli/Kira)
- Which axioms are proven in converge-core vs. designed-for but unproven?
- Commutativity: does this actually hold with LLM agents whose proposals depend on context order?
- Confluence: under what conditions does this break?
