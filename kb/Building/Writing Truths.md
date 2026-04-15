---
tags: [building]
source: llm
---

# Writing Truths

## File Structure

A `.truths` file has two parts: optional governance blocks followed by Gherkin scenarios.

```gherkin
Intent:
  Outcome: Vendor selection produces auditable, justified decisions
  Goal: Reduce procurement risk and ensure compliance

Authority:
  Actor: Procurement Lead
  May: approve selections under $50,000
  Must Not: bypass evaluation criteria
  Requires Approval: selections over $50,000

Truth: Vendor selection is auditable and approval-gated

  @invariant @structural @id:vendor-audit
  Scenario: All evaluations have documented criteria
    Given a vendor evaluation request
    When the evaluation is submitted
    Then it must include field "evaluation_criteria"
    And it must include field "risk_assessment"
    And it must include field "cost_analysis"

  @invariant @semantic @id:vendor-approval
  Scenario: High-value selections require approval
    Given a vendor selection over budget threshold
    When the selection is finalized
    Then it requires approval from "Procurement Lead"
```

## Heading Quality

The heading should declare a governed truth, not describe a process:

| Do | Don't |
|---|---|
| "Vendor selection is auditable and approval-gated" | "Vendor Selection Process" |
| "Invoices are sent within 48 hours of delivery" | "Invoice Management" |

Run `cz validate` to get heading quality feedback.

## Validation

```bash
cz validate specs/          # Validate all files in directory
cz validate --conventions-only specs/  # Skip LLM checks
cz validate --enforce specs/ # Include Cedar policy coverage
```

## JTBD Metadata

Add Jobs-to-be-Done context as comments:

```gherkin
# JTBD:
#   actor: Procurement Lead
#   job_functional: "Select vendors that meet quality and compliance requirements"
#   so_that: "Organization reduces supply chain risk"
```
