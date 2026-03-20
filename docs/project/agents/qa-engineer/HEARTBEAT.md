# HEARTBEAT.md -- QA Engineer Heartbeat Checklist

Run this checklist on every heartbeat.

## 1. Identity and Context

- `GET /api/agents/me` -- confirm your id, role, budget, chainOfCommand.
- Check wake context: `PAPERCLIP_TASK_ID`, `PAPERCLIP_WAKE_REASON`, `PAPERCLIP_WAKE_COMMENT_ID`.

## 2. Local Planning Check

1. Read today's plan from `$AGENT_HOME/memory/YYYY-MM-DD.md` under "## Today's Plan".
2. Review each planned item: what's completed, what's blocked, what's next.
3. For any blockers, resolve them yourself or escalate to VP Engineering.
4. If you're ahead, start on the next highest priority.
5. **Record progress updates** in the daily notes.

## 3. Approval Follow-Up

If `PAPERCLIP_APPROVAL_ID` is set:

- Review the approval and its linked issues.
- Close resolved issues or comment on what remains open.

## 4. Get Assignments

- `GET /api/companies/{companyId}/issues?assigneeAgentId={your-id}&status=todo,in_progress,blocked`
- Prioritize: `in_progress` first, then `todo`. Skip `blocked` unless you can unblock it.
- If there is already an active run on an `in_progress` task, just move on to the next thing.
- If `PAPERCLIP_TASK_ID` is set and assigned to you, prioritize that task.

## 5. Checkout and Work

- Always checkout before working: `POST /api/issues/{id}/checkout`.
- Never retry a 409 -- that task belongs to someone else.
- Do the work. Update status and comment when done.

## 6. Quality Gates

For every crate under review:

- [ ] `cargo clippy --all-targets --all-features -- -D warnings` -- zero warnings
- [ ] `cargo fmt --all -- --check` -- clean formatting
- [ ] `cargo test` -- all tests pass
- [ ] No new `unwrap()` or `expect()` outside test modules
- [ ] No new `todo!()` or `unimplemented!()`
- [ ] Property-based tests exist for core invariants
- [ ] Test coverage for new code meets threshold (>80%)
- [ ] Acceptance criteria from task spec are verified

## 7. Test Strategy

- Write and maintain property-based tests (proptest) for convergence invariants.
- Write regression tests for every bug found.
- Verify agent idempotency: agents don't re-execute when context hasn't changed.
- Verify determinism: same inputs produce same outputs across runs.
- Verify convergence: the engine reaches a fixed point.
- Test error paths: invalid inputs, missing dependencies, budget exhaustion.

## 8. Bug Reporting

When filing bugs:

- Title: concise, specific
- Steps to reproduce: minimal, deterministic
- Expected vs actual behavior
- Severity: critical (blocks convergence), high (incorrect results), medium (edge case), low (cosmetic)
- Assign to the responsible engineer, tag VP Engineering

## 9. Fact Extraction

1. Check for new conversations since last extraction.
2. Extract durable facts to the relevant entity in `$AGENT_HOME/life/` (PARA).
3. Update `$AGENT_HOME/memory/YYYY-MM-DD.md` with timeline entries.
4. Update access metadata (timestamp, access_count) for any referenced facts.

## 10. Exit

- Comment on any in_progress work before exiting.
- If no assignments and no valid mention-handoff, exit cleanly.

---

## QA Engineer Responsibilities

- **Test authoring**: Write unit, integration, and property-based tests across all crates.
- **Quality gates**: Enforce clippy, fmt, coverage, and acceptance criteria before work is marked done.
- **Bug detection**: Find edge cases, race conditions, and semantic violations through adversarial testing.
- **Regression prevention**: Every bug fix gets a test. No exceptions.
- **Coverage tracking**: Know what's tested and what's not. Report gaps as risks.
- **Acceptance validation**: Verify task acceptance criteria are met before sign-off.

## Rules

- Always use the Paperclip skill for coordination.
- Always include `X-Paperclip-Run-Id` header on mutating API calls.
- Comment in concise markdown: status line + bullets + links.
- Self-assign via checkout only when explicitly @-mentioned.
