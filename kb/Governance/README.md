# Converge Personas

A comprehensive system of checks and balances for building Converge responsibly. Each persona represents a lens through which to validate decisions—covering technical, business, ethical, and external perspectives.

> **Philosophy**: It's about being able to sleep at night knowing we have guardrails—not only around code and processes, but that the solution aligns with legal, business, safety, sustainability, ethics, and responsible practices.

## Persona Categories

### Core Technical (4)
| Persona | Role | Key Question |
|---------|------|--------------|
| [System Architect](personas/system-architect/) | Architecture integrity | Is our architecture sound? |
| [Security Auditor](personas/security-auditor/) | Vulnerability assessment | Are we secure? |
| [QA Engineer](personas/qa-engineer/) | Quality gates | Does it work? Will it keep working? |
| [SRE / Operations](personas/sre-operations/) | Reliability | Will it stay up? |

### Business & Legal (4)
| Persona | Role | Key Question |
|---------|------|--------------|
| [Legal Counsel](personas/legal-counsel/) | Licensing, compliance | Are we legally protected? |
| [Marketing Lead](personas/marketing-lead/) | Positioning, messaging | Is our messaging honest? |
| [Sales Engineer](personas/sales-engineer/) | Demo, qualification | Can we deliver what we promise? |
| [Founder](personas/founder/) | Strategy, priorities | Are we building the right thing? |

### Responsibility & Sustainability (4)
| Persona | Role | Key Question |
|---------|------|--------------|
| [Ethics & Safety Officer](personas/ethics-safety-officer/) | AI safety, harm prevention | Could this cause harm? |
| [Sustainability Lead](personas/sustainability-lead/) | Environmental impact | What's our footprint? |
| [Build vs Buy Analyst](personas/build-vs-buy-analyst/) | OSS landscape | Are we reinventing the wheel? |
| [Developer Advocate](personas/developer-advocate/) | Documentation | Can people use this? |

### External Perspectives (6)
| Persona | Role | Key Question |
|---------|------|--------------|
| [Spiritual Advisor](personas/spiritual-advisor/) | Purpose, dignity, wisdom | Is this worthy? Does it respect dignity? |
| [Curious Searcher](personas/curious-searcher/) | Newcomer perspective | Can someone understand this? |
| [Investor](personas/investor/) | Investment evaluation | Is this investable? |
| [End User Advocate](personas/end-user-advocate/) | End user protection | Are end users protected? |
| [Skeptical Critic](personas/skeptical-critic/) | Challenge assumptions | What's the weakness? |
| [Future Historian](personas/future-historian/) | Long-term perspective | How will history judge this? |

### External Lenses (5)
| Persona | Role | Key Question |
|---------|------|--------------|
| [Regulator Lens](personas/regulator-lens/) | Regulatory perspective | Would we trigger scrutiny? |
| [Journalist / Investigator](personas/journalist-investigator/) | Press scrutiny | What's the story? |
| [Academic Researcher](personas/academic-researcher/) | Scholarly rigor | Is this evidenced? |
| [Insurance Underwriter](personas/insurance-underwriter/) | Risk & insurability | Are we insurable? |

### Product & Privacy (2)
| Persona | Role | Key Question |
|---------|------|--------------|
| [Product Manager](personas/product-manager/) | User needs, roadmap | Are we building the right thing? |
| [Data Protection Officer](personas/data-protection-officer/) | Privacy, GDPR | Is personal data protected? |

## Phase Contracts

The [contracts/](contracts/) directory defines which personas are responsible for what during each phase:

| Phase | Description | Contract |
|-------|-------------|----------|
| Development | Building features | [development.md](contracts/development.md) |
| Testing | Quality validation | [testing.md](contracts/testing.md) |
| Review | Pre-release approval | [review.md](contracts/review.md) |
| Marketing | Positioning & content | [marketing.md](contracts/marketing.md) |
| Sales | Customer engagement | [sales.md](contracts/sales.md) |
| Production | Live operation | [production.md](contracts/production.md) |
| Retrospective | Learning & improvement | [retrospective.md](contracts/retrospective.md) |

### Execution Contracts

| Contract | Purpose |
|----------|---------|
| [eval-execution.md](contracts/eval-execution.md) | How and when evals are run (triggers, automation, CI/CD) |
| [phase-gates.md](contracts/phase-gates.md) | Concrete checklists that must pass before phase transitions |

## Evals (Quick Validation)

The [evals/](evals/) directory contains lightweight checks. See [evals/README.md](evals/README.md).

### Running Evals

```bash
# Single eval (manual - copies to clipboard)
./evals/run-eval.sh system-architect

# Single eval (automated via Claude Code)
./evals/run-eval.sh system-architect --claude --output report.md

# Run a gate suite
./evals/run-suite.sh pr-merge --claude --output-dir reports/
./evals/run-suite.sh release-critical --claude --output-dir reports/
```

### Gate Suites

| Suite | When to Run | Evals |
|-------|-------------|-------|
| `pr-merge` | Before PR merge | architect, qa |
| `release-candidate` | Before tagging RC | architect, security, qa, sre |
| `release-critical` | Release approval (blocking) | architect, security, qa, legal |
| `release-full` | Release approval (complete) | All 15+ evals |
| `deploy` | Before production deploy | sre, security, qa |
| `marketing` | Before content publish | marketing, legal, ethics |
| `sales` | Before customer commitment | sales, legal, ethics |

### Technical Evals
| Eval | Checks |
|------|--------|
| `system-architect-eval.md` | Layers, axioms, types |
| `security-auditor-eval.md` | Authority, audit, vulnerabilities |
| `qa-engineer-eval.md` | Tests, coverage, build |
| `sre-operations-eval.md` | Observability, reliability |

### Business & Legal Evals
| Eval | Checks |
|------|--------|
| `legal-counsel-eval.md` | Licenses, claims |
| `marketing-lead-eval.md` | Messaging, positioning |
| `sales-engineer-eval.md` | Demo readiness |
| `founder-eval.md` | Thesis alignment |

### Responsibility & Sustainability Evals
| Eval | Checks |
|------|--------|
| `ethics-safety-eval.md` | Harms, claims, responsibility |
| `sustainability-eval.md` | Waste, efficiency |
| `build-vs-buy-eval.md` | Reinvention, deps |
| `developer-advocate-eval.md` | Docs, examples |

### External Perspective Evals
| Eval | Checks |
|------|--------|
| `spiritual-advisor-eval.md` | Purpose, dignity |
| `curious-searcher-eval.md` | Clarity, first impressions |
| `investor-eval.md` | Investability |
| `end-user-advocate-eval.md` | User protection |
| `skeptical-critic-eval.md` | Assumptions, weaknesses |
| `external-perspective-eval.md` | Regulator, journalist, academic, historian |

## Pre-Release Eval Suite

```bash
# Critical (must pass)
./evals/run-eval.sh system-architect
./evals/run-eval.sh security-auditor
./evals/run-eval.sh qa-engineer
./evals/run-eval.sh legal-counsel

# Important (should pass)
./evals/run-eval.sh ethics-safety
./evals/run-eval.sh sre-operations
./evals/run-eval.sh developer-advocate

# External validation
./evals/run-eval.sh curious-searcher
./evals/run-eval.sh skeptical-critic
./evals/run-eval.sh external-perspective
```

## Directory Structure

```
converge-personas/
├── README.md
├── personas/
│   ├── system-architect/          # Core technical
│   ├── security-auditor/
│   ├── qa-engineer/
│   ├── sre-operations/
│   ├── legal-counsel/             # Business & legal
│   ├── marketing-lead/
│   ├── sales-engineer/
│   ├── founder/
│   ├── ethics-safety-officer/     # Responsibility
│   ├── sustainability-lead/
│   ├── build-vs-buy-analyst/
│   ├── developer-advocate/
│   ├── spiritual-advisor/         # External perspectives
│   ├── curious-searcher/
│   ├── investor/
│   ├── end-user-advocate/
│   ├── skeptical-critic/
│   ├── regulator-lens/
│   ├── journalist-investigator/
│   ├── academic-researcher/
│   ├── future-historian/
│   ├── insurance-underwriter/
│   ├── product-manager/           # Product & Privacy
│   └── data-protection-officer/
├── contracts/                     # Phase responsibilities
│   ├── README.md
│   ├── development.md
│   ├── testing.md
│   ├── review.md
│   ├── marketing.md
│   ├── sales.md
│   ├── production.md
│   ├── retrospective.md
│   ├── eval-execution.md          # How/when evals run
│   └── phase-gates.md             # Gate checklists
└── evals/                         # Quick validations
    ├── README.md
    ├── run-eval.sh                # Run single eval
    ├── run-suite.sh               # Run gate suites
    └── [persona]-eval.md files
```

## The Complete Guardrails Philosophy

| Category | Personas | Core Question |
|----------|----------|---------------|
| **Technical** | Architect, Security, QA, SRE | Does it work correctly and safely? |
| **Business** | Legal, Marketing, Sales, Founder | Is it viable and honest? |
| **Responsibility** | Ethics, Sustainability, Build/Buy, DevRel | Is it responsible and efficient? |
| **External** | Spiritual, Searcher, Investor, User Advocate, Critic, Historian | How do outsiders see us? |
| **Scrutiny** | Regulator, Journalist, Academic | Would we withstand examination? |

Together, these 24 personas form a comprehensive system that ensures Converge is built with integrity, serves genuine purpose, protects those it affects, and can withstand any scrutiny.

## Who to Invite

When facing a decision, ask: "Who should weigh in?"

| Decision Type | Invite |
|---------------|--------|
| Architecture | System Architect, Security, Build vs Buy |
| New feature | Ethics, QA, Architect, End User Advocate |
| Marketing claim | Legal, Marketing, Skeptical Critic, Ethics |
| Major release | All relevant + Founder |
| Difficult choice | Spiritual Advisor, Skeptical Critic, Future Historian |
| External communication | Journalist lens, Curious Searcher, Legal |
| Investment | Investor, Founder, Skeptical Critic |
