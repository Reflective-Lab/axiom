# Design Spec: Public Product Roadmap Page

**Route:** `/roadmap`
**Owner:** Rio Castellan (design), Blake Harmon (content curation), Ren Akiyama (technical accuracy)
**Implements:** Jules Carrera
**Priority:** Medium (P1)
**Related issue:** "Public product roadmap — let prospects see what's coming"

---

## Purpose

Prospects need to see momentum. A public roadmap signals active development, clear direction, and accountability. This is not the internal ROADMAP.md — it's a curated, simplified view that builds confidence without over-committing.

---

## Content Strategy

**Show:** Milestone names, broad timing (month, not exact dates), status indicators, what each milestone unlocks for customers.

**Don't show:** Internal owner names, specific engineering tasks, crate-level details, celebration plans, open questions.

---

## Layout

### Desktop (> 1024px)

```
[Header -- standard site header]

[Page Title Section]
  "Product Roadmap" (h1, --font-mono, --text-3xl, left-aligned)
  "Where we are and where we're headed." (subtitle, --text-lg, --ink-secondary)

[Status Legend -- inline, right of subtitle or below]
  ● Complete  ● In Progress  ○ Planned

[Timeline -- vertical, left-aligned]
  ┌─────────────────────────────────────────┐
  │ ● Milestone 1: Foundation Proven        │
  │   March 2026                            │
  │   Core convergence engine with proof    │
  │   examples and property-based tests.    │
  ├─────────────────────────────────────────┤
  │ ● Milestone 2: Website Business-Ready   │
  │   March 2026                            │
  │   Pricing, business landing page,       │
  │   interactive demos, security page.     │
  ├─────────────────────────────────────────┤
  │ ◐ Milestone 3: First Design Partner     │
  │   April 2026                            │
  │   First company running Converge on     │
  │   real workflows.                       │
  ├─────────────────────────────────────────┤
  │ ...                                     │
  └─────────────────────────────────────────┘

[CTA Section]
  "Want to shape what's next?" + pilot CTA

[Footer -- standard site footer]
```

### Mobile (< 640px)

- Timeline stacks naturally (already vertical)
- Cards full-width
- Status legend wraps if needed

---

## Milestone Card Design

Each milestone is a horizontal card with a status indicator on the left edge.

### Card Structure

```
┌──────────────────────────────────────────────┐
│ ●  MILESTONE NAME               STATUS BADGE │
│    Month YYYY                                │
│                                              │
│    Customer-facing description. What this    │
│    unlocks for people using Converge.        │
│    2-3 lines max.                            │
│                                              │
│    Key deliverables:                         │
│    ✓ Deliverable 1                           │
│    ✓ Deliverable 2                           │
│    ○ Deliverable 3                           │
└──────────────────────────────────────────────┘
```

### Card Styling

| Property | Value |
|----------|-------|
| Background | `--paper` |
| Border | 1px solid `--rule-light` |
| Border-left | 3px solid (status-dependent) |
| Padding | `--space-6` |
| Gap between cards | `--space-4` |
| Max width | `--max-width` (720px) |

### Status Colors

| Status | Border-left | Badge |
|--------|-------------|-------|
| Complete | `--pine` (3px solid) | "Complete" in `--pine` bg, `--paper` text |
| In Progress | `--mustard` (3px solid) | "In Progress" in `--mustard` bg, `--paper` text |
| Planned | `--rule` (3px solid) | "Planned" in `--surface` bg, `--ink-muted` text |

### Typography

| Element | Font | Size | Weight | Color |
|---------|------|------|--------|-------|
| Milestone name | `--font-mono` | `--text-md` | 600 | `--ink` |
| Date | `--font-mono` | `--text-xs` | 400 | `--ink-muted` |
| Description | `--font-sans` | `--text-base` | 400 | `--ink-secondary` |
| Deliverable | `--font-sans` | `--text-sm` | 400 | `--ink` |
| Deliverable check | — | — | — | `--pine` (done) or `--ink-muted` (pending) |

---

## Milestones to Display (curated from internal ROADMAP.md)

| # | Public Name | Month | Status | Customer Description |
|---|-------------|-------|--------|---------------------|
| 1 | Foundation Proven | Mar 2026 | Complete | Core convergence engine verified with property-based tests. |
| 2 | Website Business-Ready | Mar 2026 | In Progress | Pricing, business landing page, interactive demos live. |
| 3 | First Design Partner | Apr 2026 | Planned | First company running Converge on real workflows. |
| 4 | Pilot Runtime Ready | Apr 2026 | Planned | End-to-end convergence with HITL gates, audit trails, and telemetry. |
| 5 | Provider Integrations | Apr-May 2026 | Planned | Anthropic, OpenAI, and custom LLM provider support. |
| 6 | Domain Packs | Q2 2026 | Planned | Pre-built workflow packs for Lead-to-Cash, approvals, and more. |

**Note:** Milestones 6-9 from internal roadmap are collapsed into broader themes. Don't show "First Paid Contract" publicly — that's internal business metric.

---

## Status Legend

Inline row below the subtitle:

```
● Complete   ◐ In Progress   ○ Planned
```

- `●` = filled circle in `--pine`
- `◐` = filled circle in `--mustard`
- `○` = outline circle in `--ink-muted`
- Font: `--font-mono`, `--text-xs`, uppercase, `--ink-muted` for labels
- Gap: `--space-6` between items

---

## Timeline Connector

A thin vertical line connecting milestone cards on the left side:

- Width: 2px
- Color: `--rule-light`
- Runs from first card to last card
- Status dots sit on this line (8px diameter circles, colored per status)
- Creates visual continuity without being decorative

---

## CTA Section

Below the timeline:

```
┌──────────────────────────────────────────────┐
│  Want to shape what's next?                  │
│                                              │
│  Design partners help us prioritize.         │
│  Join the pilot program and your workflows   │
│  drive the roadmap.                          │
│                                              │
│  [Start Free Pilot]                          │
└──────────────────────────────────────────────┘
```

- Background: `--surface`
- Full-width bleed
- Centered text
- `--font-mono` heading, `--font-sans` body
- Tertiary button (accent bg)

---

## Interactions

- No expand/collapse — all milestones visible at once (short page, ~6 items)
- Hover on milestone card: subtle `--surface-hover` background
- No animations on load

---

## SEO / Meta

```html
<title>Product Roadmap - Converge</title>
<meta name="description" content="See what's shipping at Converge. Foundation, pilot runtime, provider integrations, and domain packs — all on the timeline.">
```

---

## Implementation Notes

- Create `src/app/pages/Roadmap.tsx` and `Roadmap.module.css`
- Add lazy-loaded route at `/roadmap` in `main.tsx`
- Add "Roadmap" link to footer navigation (not primary header nav)
- Milestone data as a typed constant array in the component — easy to update
- Status can be derived from a simple enum: `'complete' | 'in_progress' | 'planned'`
- Timeline connector: CSS pseudo-element on the card container
