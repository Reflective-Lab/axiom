# Production Onboarding Checklist — Pilot to Paid Transition

> Use this checklist when a design partner converts from pilot to production.
> This is NOT the pilot onboarding playbook — see `agents/solutions-engineer/deliverables/design-partner-onboarding-playbook.md` for pilot setup.

**Customer:** [Company Name]
**Pilot end date:** [Date]
**Production tier:** [Professional / Enterprise]
**Contract signed:** [Date]
**Target go-live:** [Date — typically 5 business days after contract]
**Owner:** Leo Marin
**Engineering contact:** [Assigned engineer, if applicable]

---

## Phase 1: Commercial & Legal (Day 0-2)

- [ ] Production contract signed (MSA + Order Form)
- [ ] Payment method on file (Stripe billing set up)
- [ ] SLA tier confirmed: [Starter / Professional / Enterprise]
- [ ] Support channel established: [email / Slack Connect / dedicated channel]
- [ ] Customer success handoff meeting (Leo → named account owner, if applicable)

## Phase 2: Environment Migration (Day 1-3)

### Workspace Setup

- [ ] Production workspace created (separate from pilot workspace)
- [ ] Customer team accounts provisioned: [list users]
- [ ] Role-based access configured per tier
- [ ] Domain packs activated: [list packs]
- [ ] Run allowance configured per contract: [X runs/month]
- [ ] Overage billing enabled (if applicable): [$0.02/run]

### Data Migration

- [ ] Decision: migrate pilot data or start fresh? **[Migrate / Fresh Start]**
- [ ] If migrating: pilot workflow configurations exported
- [ ] If migrating: pilot integrations verified in production environment
- [ ] If fresh start: pilot data disposal scheduled per agreement §6.4

### Infrastructure

- [ ] Production tenant isolated (per Ava's access control policy)
- [ ] Encryption at rest confirmed for production environment
- [ ] TLS 1.2+ verified for all endpoints
- [ ] Backup and recovery configured per tier SLA
- [ ] Monitoring and alerting enabled for customer workspace

## Phase 3: Integration Cutover (Day 2-4)

- [ ] Integration credentials rotated (new production keys, pilot keys revoked)
- [ ] OAuth tokens re-authorized in production environment
- [ ] Webhook endpoints updated to production URLs
- [ ] Integration health check: all connected systems responding
- [ ] End-to-end test run completed successfully in production
- [ ] Customer's IT team notified of production endpoints (if firewall/allowlist needed)

## Phase 4: Validation (Day 3-5)

- [ ] First production convergence run completed successfully
- [ ] Customer confirms output matches expected behavior
- [ ] HITL gates functioning correctly in production
- [ ] Monitoring confirms metrics collection is active
- [ ] Support channel tested (customer sends test request, Leo responds)
- [ ] SLA clock starts: [Date]

## Phase 5: Pilot Cleanup (Day 5-10)

- [ ] Pilot workspace decommissioned
- [ ] Pilot-specific credentials revoked
- [ ] Pilot data disposed per agreement §6.4 (if not migrated)
- [ ] Disposal confirmation sent to customer
- [ ] Pilot agreement superseded by production contract (confirm with customer)

## Phase 6: Steady State Setup (Day 5-15)

- [ ] Regular check-in cadence established: [weekly / biweekly / monthly]
- [ ] Customer has self-serve documentation access (per SLA tier)
- [ ] Escalation path documented and shared with customer
- [ ] Usage dashboard access provided to customer (if available)
- [ ] 30-day post-production review scheduled: [Date]
- [ ] Expansion opportunities documented: [additional workflows, departments, users]

---

## Sign-Off

| Role | Name | Date | Approved |
|------|------|------|----------|
| Solutions Engineer | Leo Marin | | [ ] |
| Customer Pilot Lead | [Name] | | [ ] |
| Security (if Enterprise) | Ava Petrov | | [ ] |
| Engineering | [Name] | | [ ] |

---

## Notes

[Any customer-specific considerations, unusual requirements, or lessons learned from the pilot.]

---

*Referenced from: pilot-to-contract playbook §5.3. See also: SLA tier definitions (`agents/solutions-engineer/deliverables/sla-tier-definitions.md`).*
