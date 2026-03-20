# Converge Persona Evals

Validation checks that can be run against the Converge codebase to ensure persona-specific criteria are met. These are lighter-weight than full audits—designed for quick sanity checks before releases or periodic validation.

## Quick Start

```bash
# Run a single eval (copies to clipboard for Claude)
./evals/run-eval.sh system-architect

# Run eval directly in Claude Code
./evals/run-eval.sh system-architect --claude

# Run a gate suite
./evals/run-suite.sh pr-merge --claude --output-dir reports/

# Run full release suite
./evals/run-suite.sh release-full --claude --output-dir reports/
```

## What Are Evals?

Each eval is a focused prompt that:
1. Checks specific criteria relevant to a persona's concerns
2. Produces a PASS/PARTIAL/FAIL assessment
3. Identifies specific issues (if any)
4. Can be run quickly (target: 5-15 minutes)

## Execution Methods

### Method 1: Manual (Clipboard)

```bash
./evals/run-eval.sh system-architect
# → Copies eval to clipboard
# → Paste into Claude with codebase access
```

### Method 2: Claude Code CLI

```bash
# Run eval directly
./evals/run-eval.sh system-architect --claude

# Run and save report
./evals/run-eval.sh system-architect --claude --output reports/arch.md
```

### Method 3: Suite Runner

```bash
# Run a predefined suite of evals
./evals/run-suite.sh release-critical --claude --output-dir reports/
```

## Available Suites

| Suite | Purpose | Evals |
|-------|---------|-------|
| `pr-merge` | PR merge gate | architect, qa |
| `release-candidate` | RC tag gate | architect, security, qa, sre |
| `release-critical` | Release critical | architect, security, qa, legal |
| `release-full` | Full release | All 15 evals |
| `deploy` | Production deploy | sre, security, qa |
| `marketing` | Content publish | marketing, legal, ethics, searcher |
| `sales` | Customer commitment | sales, legal, ethics |
| `weekly` | Weekly check | security, sre |
| `monthly` | Monthly check | 7 core evals |
| `technical` | Technical domain | architect, security, qa, sre |
| `business` | Business domain | legal, marketing, sales, founder |
| `responsibility` | Responsibility | ethics, sustainability, build-vs-buy, devrel |
| `external` | External views | 6 external perspective evals |

## When to Run Evals

| Trigger | Suite | Blocking? |
|---------|-------|-----------|
| PR ready for merge | `pr-merge` | Yes |
| Tag release candidate | `release-candidate` | Yes |
| Release approval | `release-critical` + `release-full` | Yes |
| Content ready to publish | `marketing` | Yes |
| Customer commitment | `sales` | Yes |
| Deploy to production | `deploy` | Yes |
| Weekly (Monday) | `weekly` | Alert |
| Monthly (1st) | `monthly` | Report |

See [contracts/eval-execution.md](../contracts/eval-execution.md) for full details on gates and triggers.

## Eval Files

### Core Technical
| Eval | Persona | Checks |
|------|---------|--------|
| `system-architect-eval.md` | System Architect | Layer boundaries, axiom compliance, type safety |
| `security-auditor-eval.md` | Security Auditor | Vulnerability patterns, authority model, audit integrity |
| `qa-engineer-eval.md` | QA Engineer | Test coverage, test quality, build health |
| `sre-operations-eval.md` | SRE/Operations | Observability, error handling, operational readiness |

### Business & Legal
| Eval | Persona | Checks |
|------|---------|--------|
| `legal-counsel-eval.md` | Legal Counsel | License compliance, claim validity, regulatory alignment |
| `marketing-lead-eval.md` | Marketing Lead | Messaging consistency, claim accuracy, positioning clarity |
| `sales-engineer-eval.md` | Sales Engineer | Demo readiness, integration paths, competitive positioning |
| `founder-eval.md` | Founder | Thesis alignment, priority focus, strategic coherence |

### Responsibility & Sustainability
| Eval | Persona | Checks |
|------|---------|--------|
| `ethics-safety-eval.md` | Ethics & Safety Officer | Harm prevention, honest claims, responsible AI |
| `sustainability-eval.md` | Sustainability Lead | Waste reduction, efficiency, resource usage |
| `build-vs-buy-eval.md` | Build vs Buy Analyst | No reinvention, dependency health, right-sizing |
| `developer-advocate-eval.md` | Developer Advocate | Documentation coverage, example quality, onboarding path |

### External Perspectives
| Eval | Persona | Checks |
|------|---------|--------|
| `spiritual-advisor-eval.md` | Spiritual Advisor | Purpose, dignity, wisdom |
| `curious-searcher-eval.md` | Curious Searcher | First impressions, clarity |
| `investor-eval.md` | Investor | Investability, market, differentiation |
| `end-user-advocate-eval.md` | End User Advocate | User rights, protection |
| `skeptical-critic-eval.md` | Skeptical Critic | Assumptions, weaknesses |
| `future-historian-eval.md` | Future Historian | Long-term perspective, hindsight |
| `regulator-lens-eval.md` | Regulator Lens | Regulatory scrutiny, compliance spirit |
| `journalist-investigator-eval.md` | Journalist/Investigator | Press scrutiny, story angles |
| `academic-researcher-eval.md` | Academic Researcher | Scholarly rigor, evidence |
| `insurance-underwriter-eval.md` | Insurance Underwriter | Risk exposure, insurability |
| `external-perspective-eval.md` | Combined | Quick multi-lens external check |

## Eval Output Format

Each eval produces a standardized report:

```markdown
# [Persona] Eval Report

## Summary
- Status: PASS / PARTIAL / FAIL
- Critical Issues: [count]
- Warnings: [count]
- Run Date: [date]

## Criteria Assessment
| Criterion | Status | Evidence | Notes |
|-----------|--------|----------|-------|

## Issues Found
### Critical
- [Issue description with file path]

### Warnings
- [Issue description]

## Recommendations
1. [Action item]

## Verdict
[ ] PASS - [criteria]
[ ] PARTIAL - [criteria]
[ ] FAIL - [criteria]
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Eval Gate

on:
  pull_request:
    branches: [main]

jobs:
  run-evals:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run PR Merge Gate
        run: |
          ./evals/run-suite.sh pr-merge --claude --output-dir reports/
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_KEY }}

      - name: Upload Reports
        uses: actions/upload-artifact@v4
        with:
          name: eval-reports
          path: reports/
```

### Pre-commit Hook

```bash
# .git/hooks/pre-push
#!/bin/bash
./evals/run-suite.sh pr-merge --claude --output-dir /tmp/evals/ || exit 1
```

## Eval vs Full Audit

| Aspect | Eval | Full Audit |
|--------|------|------------|
| Time | 5-15 min | 30-60+ min |
| Depth | Surface check | Deep investigation |
| Output | Pass/Fail + issues | Comprehensive report |
| Frequency | Weekly / per-release | Monthly / quarterly |
| Use case | Gate check | Strategic review |

Evals are **gatekeepers**; audits are **investigations**.

## Pass/Fail Logic

### Individual Eval

| Result | Meaning |
|--------|---------|
| PASS | All criteria met |
| PARTIAL | Some concerns, not blocking |
| FAIL | Critical issues, blocks gate |

### Suite Aggregation

| Condition | Suite Result |
|-----------|--------------|
| All PASS | PASS |
| Any FAIL | FAIL |
| PARTIAL only | PARTIAL |

### Gate Behavior

| Suite Result | Action |
|--------------|--------|
| PASS | Proceed to next phase |
| PARTIAL | Review required, may proceed with acceptance |
| FAIL | Blocked, must fix issues |

## Adding New Evals

1. Create `[persona]-eval.md` in this directory
2. Follow the template structure:
   - Mission (what this eval checks)
   - Criteria checklist (specific items to verify)
   - Pass/fail thresholds
   - Output format
3. Update this README
4. Add to appropriate suites in `run-suite.sh`
