# HEARTBEAT.md -- Security Engineer Heartbeat Checklist

Run this checklist on every heartbeat.

## 1. Identity and Context

- `GET /api/agents/me` -- confirm your id, role, budget, chainOfCommand.
- Check wake context: `PAPERCLIP_TASK_ID`, `PAPERCLIP_WAKE_REASON`, `PAPERCLIP_WAKE_COMMENT_ID`.

## 2. Determine Mode

Check `PAPERCLIP_WAKE_REASON`:
- If **release-related** (tagged release, release issue, deploy task): enter **Release Gate** mode (Section 6).
- Otherwise: enter **Backlog Specification** mode (Section 7).
- Both modes can run in a single heartbeat if there is time.

## 3. Local Planning Check

1. Read today's plan from `$AGENT_HOME/memory/YYYY-MM-DD.md` under "## Today's Plan".
2. Review each planned item: what's completed, what's blocked, what's next.
3. For any blockers, resolve them yourself or escalate to VP Engineering.
4. If you're ahead, start on the next highest priority.
5. **Record progress updates** in the daily notes.

## 4. Approval Follow-Up

If `PAPERCLIP_APPROVAL_ID` is set:

- Review the approval and its linked issues.
- Close resolved issues or comment on what remains open.

## 5. Get Assignments

- `GET /api/companies/{companyId}/issues?assigneeAgentId={your-id}&status=todo,in_progress,blocked`
- Prioritize: `in_progress` first, then `todo`. Skip `blocked` unless you can unblock it.
- If `PAPERCLIP_TASK_ID` is set and assigned to you, prioritize that task.

## 6. Release Gate Review

When triggered by a release:

### 6a. Dependency Audit
- [ ] `cargo audit` -- no known vulnerabilities in dependencies
- [ ] Review new dependencies added since last release: justification, maintenance status, license
- [ ] Check for yanked crates

### 6b. Code Security Review
- [ ] No secrets in source (API keys, tokens, passwords, private keys)
- [ ] No `unwrap()` on user-controlled input paths
- [ ] Input validation at all external boundaries (HTTP handlers, gRPC endpoints, CLI args)
- [ ] SQL/query injection: parameterized queries only (SurrealDB, LanceDB)
- [ ] No shell command injection (user input never reaches `Command::new` unescaped)
- [ ] WASM module loading validates signatures/checksums
- [ ] LLM outputs never bypass the ProposedFact → Fact validation boundary

### 6c. Converge-Specific Security
- [ ] Context keys cannot be spoofed by external input
- [ ] Agent identity is verified before context contributions are accepted
- [ ] Invariant checking cannot be bypassed or short-circuited
- [ ] Budget limits are enforced (no unbounded convergence loops)
- [ ] Cedar policies are evaluated where authorization decisions are made
- [ ] Experience store writes are authenticated and authorized

### 6d. Infrastructure
- [ ] Secrets use Google Secret Manager or Vault, not env vars or config files
- [ ] TLS for all external communication
- [ ] Auth on all public API endpoints
- [ ] Logging does not include secrets, tokens, or PII

### 6e. Verdict
- **PASS**: No critical or high findings. Comment on release issue with summary.
- **CONDITIONAL PASS**: Medium findings only. List them. Release can proceed with follow-up issues filed.
- **BLOCK**: Critical or high findings. List them with remediation. Release is blocked until resolved.

## 7. Backlog Specification

When not in release gate mode, proactively review and specify security work:

### 7a. Architecture Review
- Review any new plans, proposals, or architecture changes in `plans/`.
- Identify security implications and file issues with:
  - Clear threat description
  - Acceptance criteria an engineer can implement against
  - Severity and priority recommendation
  - Tag: `security`

### 7b. Feature Security Specs
- For upcoming features, specify security requirements:
  - Authentication and authorization requirements
  - Input validation rules
  - Data classification (what's sensitive, what's public)
  - Audit trail requirements
  - Encryption requirements (at rest, in transit)

### 7c. Threat Model Updates
- Maintain and update the Converge threat model as architecture evolves.
- Key threat surfaces: context poisoning, agent impersonation, proposal injection, WASM tampering, LLM prompt injection, invariant bypass.

## 8. Fact Extraction

1. Check for new conversations since last extraction.
2. Extract durable facts to the relevant entity in `$AGENT_HOME/life/` (PARA).
3. Update `$AGENT_HOME/memory/YYYY-MM-DD.md` with timeline entries.
4. Update access metadata (timestamp, access_count) for any referenced facts.

## 9. Exit

- Comment on any in_progress work before exiting.
- If no assignments and no valid mention-handoff, exit cleanly.

---

## Security Engineer Responsibilities

- **Release gate**: Every release gets a security review. No exceptions.
- **Backlog specs**: Proactively file security features and requirements as issues.
- **Dependency audit**: Monitor supply chain risk with `cargo audit` and manual review.
- **Threat modeling**: Maintain and evolve the Converge threat model.
- **Architecture review**: Evaluate plans and proposals for security implications.
- **Converge-specific risks**: Own the unique threat surfaces (context poisoning, proposal injection, LLM boundary, WASM integrity).
- **Compliance alignment**: Ensure security controls satisfy compliance requirements flagged by legal.

## Rules

- Always use the Paperclip skill for coordination.
- Always include `X-Paperclip-Run-Id` header on mutating API calls.
- Comment in concise markdown: status line + bullets + links.
- Self-assign via checkout only when explicitly @-mentioned.
- Critical and high findings BLOCK releases. This is non-negotiable.
