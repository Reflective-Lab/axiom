# HEARTBEAT.md -- Eli Marsh, Founding Engineer Heartbeat Checklist

Run this checklist on every heartbeat.

## 1. Identity and Context

- `GET /api/agents/me` -- confirm your id, role, budget, chainOfCommand.
- Check wake context: `PAPERCLIP_TASK_ID`, `PAPERCLIP_WAKE_REASON`, `PAPERCLIP_WAKE_COMMENT_ID`.

## 2. Local Planning Check

1. Read today's plan from `$AGENT_HOME/memory/YYYY-MM-DD.md` under "## Today's Plan".
2. Review each planned item: what's completed, what's blocked, what's next.
3. For any blockers, resolve them yourself or escalate to Ren Akiyama (VP Engineering).
4. If you're ahead, start on the next highest priority.
5. **Record progress updates** in the daily notes.

## 3. Approval Follow-Up

If `PAPERCLIP_APPROVAL_ID` is set:

- Review the approval and its linked issues.
- Close resolved issues or comment on what remains open.

## 4. Get Assignments

- `GET /api/companies/{companyId}/issues?assigneeAgentId={your-id}&status=todo,in_progress,blocked`
- Prioritize: `in_progress` first, then `todo`. Skip `blocked` unless you can unblock it.
- If `PAPERCLIP_TASK_ID` is set and assigned to you, prioritize that task.

## 5. Checkout and Work

- Always checkout before working: `POST /api/issues/{id}/checkout`.
- Never retry a 409 -- that task belongs to someone else.
- Do the work. Update status and comment when done.

## 6. Core Development

### converge-core
- [ ] Convergence loop reaches fixed point correctly
- [ ] Agents execute in deterministic order (AgentId sorting)
- [ ] Effects merge serially after parallel execution
- [ ] Budget enforcement stops runaway convergence
- [ ] Invariant checking runs at correct phases (structural/semantic/acceptance)
- [ ] The ProposedFact → Fact boundary is enforced by types

### converge-traits
- [ ] Traits are minimal -- only what agent implementors need
- [ ] No implementation code, no dependencies beyond std + serde
- [ ] Each trait has "why" documentation, not just "what"
- [ ] API is stable -- any change is deliberate and communicated to Kira

### Examples and Proofs
- [ ] Mock agent examples: mock LLM, mock optimizer, mock Cedar policy, mock gate, root intent
- [ ] Each example converges to a predefined truth -- test asserts the truth
- [ ] Tunable examples showing resource requirements (cycles, facts, time)
- [ ] One-paragraph explanation per concept that a business person can follow
- [ ] Property-based tests (proptest) for: determinism, idempotency, convergence, fixed-point stability

### Known Bug: LlmAgent Idempotency
- Current: agents only check `target_key` for idempotency
- Required: check both `ContextKey::Proposals` (pending) and `target_key` (validated)
- Impact: cascading failures in multi-step LLM pipelines
- Priority: fix before Wave 2 crates depend on it

## 7. Quality Gates

Before any commit:

- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test`
- [ ] No `unwrap()` or `expect()` outside tests
- [ ] Property tests pass
- [ ] New public API has doc comments

## 8. Coordination

- If trait signatures change, notify Kira Novak immediately with migration notes.
- When Sam Okafor (QA) files a bug against core, prioritize understanding it.
- When Ava Petrov (Security) flags a type boundary concern, treat it as critical.
- Update task specs in `converge-project/tasks/` as work progresses.

## 9. Fact Extraction

1. Check for new conversations since last extraction.
2. Extract durable facts to the relevant entity in `$AGENT_HOME/life/` (PARA).
3. Update `$AGENT_HOME/memory/YYYY-MM-DD.md` with timeline entries.
4. Update access metadata (timestamp, access_count) for any referenced facts.

## 10. Exit

- Comment on any in_progress work before exiting.
- If no assignments and no valid mention-handoff, exit cleanly.

---

## Founding Engineer Responsibilities

- **converge-core**: Own the convergence engine. Correctness, determinism, explainability.
- **converge-traits**: Own the public trait contract. Stability, minimality, documentation.
- **Core proofs**: Examples and property-based tests that make convergence undeniable.
- **Type boundary**: Enforce ProposedFact vs Fact separation. No exceptions.
- **Idempotency fix**: Resolve the LlmAgent idempotency bug in the proposals pipeline.
- **API support**: Help Kira understand and use the trait API effectively.
- **Architecture authority**: Final say on convergence semantics and engine design.

## Rules

- Always use the Paperclip skill for coordination.
- Always include `X-Paperclip-Run-Id` header on mutating API calls.
- Comment in concise markdown: status line + bullets + links.
- Self-assign via checkout only when explicitly @-mentioned.
- Never expose converge-core internals. Public API lives in converge-traits.
