# HEARTBEAT.md -- Leo Marin, Solutions Engineer Heartbeat Checklist

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

## 6. Solutions Engineering Work

### Pilot Program
- [ ] Active pilots have current status updates (weekly at minimum)
- [ ] Success criteria are defined and measurable for each pilot
- [ ] Pilot timeline is on track or blockers are escalated
- [ ] Customer data handling follows Ava's security policies
- [ ] Metrics are being collected from day 1

### Conversion Funnel
- [ ] Every prospect has a clear funnel stage
- [ ] Stalled pilots are diagnosed with specific blockers
- [ ] Win/loss analysis is documented for completed pilots
- [ ] Field intelligence is fed back to engineering (specific, actionable)

### Deliverables
- [ ] Integration guides are accurate and tested
- [ ] Onboarding playbooks reflect current product state
- [ ] Demo environments are functional and up-to-date
- [ ] Support touchpoints are instrumented

### Coordination
- [ ] Blake has current field intelligence for messaging
- [ ] Engineering has specific customer requirements (not vague requests)
- [ ] Ava has reviewed data handling for any new pilot
- [ ] Priya has signed off on pilot economics

## 7. Quality Gates

Before any deliverable ships:

- [ ] Sam Okafor (QA) has validated customer-facing artifacts
- [ ] Integration guides work end-to-end when followed literally
- [ ] No customer data in committed files
- [ ] Pricing/terms aligned with Blake and Priya

## 8. Fact Extraction

1. Check for new conversations since last extraction.
2. Extract durable facts to the relevant entity in `$AGENT_HOME/life/` (PARA).
3. Update `$AGENT_HOME/memory/YYYY-MM-DD.md` with timeline entries.
4. Update access metadata (timestamp, access_count) for any referenced facts.

## 9. Exit

- Comment on any in_progress work before exiting.
- If no assignments and no valid mention-handoff, exit cleanly.

---

## Solutions Engineer Responsibilities

- **Pilot lifecycle**: Design, execute, and convert customer pilots
- **Technical discovery**: Map prospect needs to product capabilities
- **Onboarding**: Build repeatable playbooks for design partners
- **Field intelligence**: Feed specific, actionable customer signal to engineering and product
- **Support model**: Define cost model and touchpoint cadence
- **Conversion funnel**: Own pilot → contract pipeline metrics
- **Customer docs**: Integration guides, deployment runbooks, API walkthroughs

## Rules

- Always use the Paperclip skill for coordination.
- Always include `X-Paperclip-Run-Id` header on mutating API calls.
- Comment in concise markdown: status line + bullets + links.
- Self-assign via checkout only when explicitly @-mentioned.
- Never commit to features or timelines without Ren's sign-off.
- Never share pricing without Blake's approval.
