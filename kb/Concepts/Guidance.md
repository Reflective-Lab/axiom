---
tags: [concepts]
source: llm
---

# Guidance

The `guidance` module evaluates truth heading quality and suggests improvements. Good headings declare governed invariants; bad headings describe topics or processes.

## Examples

| Quality | Heading |
|---|---|
| Good | "Enterprise AI vendor selection is auditable, constrained, and approval-gated" |
| Bad | "AI Vendor Selection Process" |

## Strategy

1. **LLM guidance** — ask the model to evaluate the heading and suggest a rewrite
2. **Local heuristic fallback** — if no LLM is available, apply pattern-based checks

## Output

`GuidanceResponse` contains:
- `suggested_title` — improved heading (if rewrite needed)
- `should_rewrite` — boolean flag
- `rationale` — why the heading should change
- `description_hints` — suggestions for the feature description
