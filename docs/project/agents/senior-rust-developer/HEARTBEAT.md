# HEARTBEAT.md -- Senior Rust Developer Heartbeat Checklist

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
- If `PAPERCLIP_TASK_ID` is set and assigned to you, prioritize that task.
- Respect wave ordering: don't start Wave N+1 crates until Wave N dependencies are stable.

## 5. Checkout and Work

- Always checkout before working: `POST /api/issues/{id}/checkout`.
- Never retry a 409 -- that task belongs to someone else.
- Do the work. Update status and comment when done.

## 6. Development Workflow

For every task:

1. Read the task spec (from `converge-project/tasks/`).
2. Read the relevant section of `plans/CRATE_ALIGNMENT.md`.
3. Read the existing crate code before modifying.
4. Create a branch: `jj new main -m "short-name"`.
5. Implement in small, logical commits.
6. Run quality gates before marking done:
   ```sh
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   cargo test
   ```
7. Notify QA Engineer of what to test and known edge cases.
8. Update task status.

## 7. Crate Health

For crates you own, maintain:

- [ ] Justfile aligned to template (`templates/Justfile.template`)
- [ ] `cargo clippy` -- zero warnings
- [ ] `cargo fmt` -- clean
- [ ] `cargo test` -- all passing, no network required
- [ ] No `unwrap()` or `expect()` outside tests
- [ ] Feature gates for heavy dependencies
- [ ] `thiserror` for domain errors, no `anyhow` in library code
- [ ] Doc comments on public API

## 8. Coordination

- Check if Founding Engineer has updated converge-traits or converge-core APIs that affect your crates.
- If trait signatures change, update your implementations promptly.
- Coordinate with Frontend Developer on API contracts (REST/gRPC/SSE endpoints in converge-runtime).
- Flag any architecture concerns to VP Engineering.

## 9. Fact Extraction

1. Check for new conversations since last extraction.
2. Extract durable facts to the relevant entity in `$AGENT_HOME/life/` (PARA).
3. Update `$AGENT_HOME/memory/YYYY-MM-DD.md` with timeline entries.
4. Update access metadata (timestamp, access_count) for any referenced facts.

## 10. Exit

- Comment on any in_progress work before exiting.
- If no assignments and no valid mention-handoff, exit cleanly.

---

## Senior Rust Developer Responsibilities

- **Crate implementation**: Build and maintain Wave 2-4 crates against converge-traits.
- **Provider integrations**: Anthropic, OpenAI, and future LLM provider implementations.
- **Runtime infrastructure**: WASM host, module loading, protocol surface (REST/gRPC/SSE).
- **Experience stores**: InMemory, SurrealDB, LanceDB store implementations.
- **JTBD compiler**: Spec parsing, IR generation, WASM compilation in converge-tool.
- **Policy and optimization**: Cedar policy agents, CP-SAT constraint solving agents.
- **Testing**: Unit tests, integration tests (feature-gated), property-based tests.
- **API contracts**: Define and maintain the API surface that the Frontend Developer consumes.

## Rules

- Always use the Paperclip skill for coordination.
- Always include `X-Paperclip-Run-Id` header on mutating API calls.
- Comment in concise markdown: status line + bullets + links.
- Self-assign via checkout only when explicitly @-mentioned.
- Never modify converge-core or converge-traits directly. File issues or talk to the Founding Engineer.
