# Persona Contracts by Phase

This directory defines which personas are responsible for what during each phase of the product lifecycle. A "contract" is a set of responsibilities, gates, and deliverables that a persona commits to during a phase.

## Phases

| Phase | Description | Primary Concerns |
|-------|-------------|------------------|
| **Development** | Building features and capabilities | Quality, architecture, security |
| **Testing** | Validating correctness and safety | Coverage, quality gates, regression |
| **Review** | Pre-release review and approval | Multi-stakeholder sign-off |
| **Marketing** | Positioning and messaging | Claims accuracy, positioning |
| **Sales** | Customer engagement | Demo readiness, qualification |
| **Production** | Live operation | Reliability, monitoring, response |
| **Retrospective** | Learning and improvement | Analysis, improvement |

## Contract Structure

Each contract defines:

```yaml
persona: [Persona name]
phase: [Phase name]

responsibilities:
  - [What they own]

gates:
  - [What they must approve before moving forward]

deliverables:
  - [What they produce]

collaboration:
  with: [Other personas they work with]
  how: [How they collaborate]

escalation:
  triggers: [What triggers escalation]
  to: [Who they escalate to]
```

## The RACI Matrix

For each phase, personas can be:
- **R**esponsible: Does the work
- **A**ccountable: Ultimately answerable
- **C**onsulted: Provides input
- **I**nformed: Kept in the loop

## Quick Reference

### Development Phase
| Persona | Role |
|---------|------|
| System Architect | A (architecture decisions) |
| QA Engineer | C (testability), R (test creation) |
| Security Auditor | C (security review) |
| Build vs Buy | C (dependency decisions) |
| Ethics & Safety | C (feature ethics) |

### Testing Phase
| Persona | Role |
|---------|------|
| QA Engineer | A (quality gates) |
| Security Auditor | R (security testing) |
| System Architect | C (architecture validation) |

### Review Phase
| Persona | Role |
|---------|------|
| All relevant personas | R (their domain review) |
| Founder | A (final approval) |

### Marketing Phase
| Persona | Role |
|---------|------|
| Marketing Lead | R (messaging) |
| Legal Counsel | A (claims approval) |
| Ethics & Safety | C (claim ethics) |

### Sales Phase
| Persona | Role |
|---------|------|
| Sales Engineer | R (demos, qualification) |
| Marketing Lead | C (positioning) |
| Legal Counsel | C (contract terms) |

### Production Phase
| Persona | Role |
|---------|------|
| SRE / Operations | A (reliability) |
| Security Auditor | R (monitoring) |
| QA Engineer | C (issue triage) |

### Retrospective Phase
| Persona | Role |
|---------|------|
| All personas | C (input) |
| Founder | A (decisions) |
| Future Historian | C (long-term perspective) |

## Contract Files

### Phase Contracts
- [development.md](development.md) - Development phase contracts
- [testing.md](testing.md) - Testing phase contracts
- [review.md](review.md) - Pre-release review contracts
- [marketing.md](marketing.md) - Marketing phase contracts
- [sales.md](sales.md) - Sales phase contracts
- [production.md](production.md) - Production phase contracts
- [retrospective.md](retrospective.md) - Retrospective phase contracts

### Execution Contracts
- [eval-execution.md](eval-execution.md) - How and when evals are executed
- [phase-gates.md](phase-gates.md) - Concrete gate checklists for phase transitions
