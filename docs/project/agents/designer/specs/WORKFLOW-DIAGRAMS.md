# Design Spec: Before/After Workflow Diagrams

**Used in:** Business-buyer landing page (`/for/operations`), Section 3 — "What Converge does"
**Owner:** Rio Castellan (design), Jules Carrera (build as inline SVG/React)
**Priority:** High (blocks business-buyer page visual impact)
**Related issue:** `88b68241-76fd-430f-ad14-6313267a4381`

---

## Concept

Two side-by-side diagrams showing the same Lead-to-Cash workflow — one broken, one converged. The visual delta IS the story. Business buyers should immediately see the difference without reading any text.

**Key principle:** Same structure, different execution. Both diagrams show the same 5 stages. The "Before" is chaotic. The "After" is governed. The layout is identical so the comparison is instant.

---

## The 5 Stages (Both Diagrams)

1. **Lead captured** — A new lead enters the system
2. **Enriched & qualified** — Lead data is enriched, scored, qualified
3. **Quote generated** — Pricing quote is created and sent
4. **Contract signed** — Deal closes, contract executed
5. **Invoice & cash** — Invoice sent, payment received

---

## BEFORE Diagram: "Without Converge"

### Visual Tone
- **Palette:** `var(--wine)` for problems, `var(--ink-muted)` for de-emphasized elements, `var(--rule)` for borders
- **Lines:** Dashed, `stroke-dasharray="4 4"`, `var(--rule)` color
- **Overall feel:** Fragmented, unreliable, things falling through cracks

### Layout

```
BEFORE — Without Converge

  ┌─ ─ ─ ─ ─ ─┐     ┌─ ─ ─ ─ ─ ─┐     ┌─ ─ ─ ─ ─ ─┐
  │  Lead       │- - -│  Enriched   │- - -│  Quote      │
  │  captured   │     │  & qualified│     │  generated  │
  └─ ─ ─ ─ ─ ─┘     └─ ─ ─ ─ ─ ─┘     └─ ─ ─ ─ ─ ─┘
       │                    │                    │
       ▼                    ▼                    ▼
  [!] Manual          [!] No audit         [!] 4-hour
      entry               trail                delay
       │                                         │
       ×─ ─ ─ ─ ─ ┐                              │
                   ▼                              ▼
            ┌─ ─ ─ ─ ─ ─┐     ┌─ ─ ─ ─ ─ ─┐
            │  Contract   │- - -│  Invoice    │
            │  signed     │     │  & cash     │
            └─ ─ ─ ─ ─ ─┘     └─ ─ ─ ─ ─ ─┘
                                     │
                                     ▼
                               [!] Mismatch
```

### Elements

| Element | Style |
|---------|-------|
| Stage boxes | Dashed border (`stroke-dasharray: 4 4`), `var(--rule)` stroke, `var(--paper)` fill, rounded corners 0 (sharp — broken feels angular) |
| Stage text | `var(--font-mono)`, 12px, `var(--ink-muted)` |
| Connector lines | Dashed, `var(--rule)`, 1px stroke |
| Problem callouts [!] | `var(--wine)` text + small circle with `!`, `var(--font-mono)`, 11px |
| Problem labels | `var(--font-sans)`, 11px, `var(--ink-muted)` |
| Dropped connection (×) | `var(--wine)`, X mark where a handoff fails |
| Overall label | "BEFORE" — `var(--font-mono)`, `var(--text-xs)`, uppercase, `var(--ink-muted)`, `letter-spacing: 0.1em` |

### Problem Callouts (positioned below relevant stage)

1. **Lead captured → "Manual entry"** — Leads entered by hand, errors and delays
2. **Enriched → "No audit trail"** — No record of who enriched what or when
3. **Quote → "4-hour delay"** — Pricing requires manager approval over email
4. **Lead→Contract gap → "Dropped"** — The X mark: a lead falls through the crack between stages
5. **Invoice → "Mismatch"** — Invoice doesn't match the quote terms

---

## AFTER Diagram: "With Converge"

### Visual Tone
- **Palette:** `var(--accent)` / `var(--pine)` for success, `var(--ink)` for strong text, `var(--rule-light)` for structure
- **Lines:** Solid, `var(--accent)` color, 1.5px stroke
- **Overall feel:** Connected, governed, every step tracked

### Layout

```
AFTER — With Converge

  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
  │  Lead         │─────│  Enriched     │─────│  Quote        │
  │  captured     │     │  & qualified  │     │  generated    │
  │               │     │               │     │               │
  │  ✓ Auto-      │     │  ✓ Agent-     │     │  ✓ Policy-    │
  │    captured   │     │    enriched   │     │    checked    │
  └──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │
       │              [trace logged]        [HITL gate]
       │                                         │
       ▼                                         ▼
  ┌──────────────┐     ┌──────────────┐
  │  Contract     │─────│  Invoice      │
  │  signed       │     │  & cash       │
  │               │     │               │
  │  ✓ Terms      │     │  ✓ Auto-      │
  │    validated  │     │    reconciled │
  └──────────────┘     └──────────────┘
                              │
                              ▼
                        [✓ Full audit trail]
```

### Elements

| Element | Style |
|---------|-------|
| Stage boxes | Solid border, `var(--accent)` stroke 1.5px, `var(--paper)` fill, `border-radius: 4px` |
| Stage title | `var(--font-mono)`, 12px, `var(--ink)`, weight 600 |
| Stage detail (✓ line) | `var(--font-sans)`, 11px, `var(--accent)` for checkmark, `var(--ink-secondary)` for text |
| Connector lines | Solid, `var(--accent)`, 1.5px stroke |
| Governance annotations | `var(--font-mono)`, 10px, `var(--accent)`, enclosed in `[ ]` brackets |
| HITL gate | Special annotation: `var(--mustard)` text (human decision point) |
| Audit trail badge | Bottom: `var(--pine)` background pill, white text, "Full audit trail" |
| Overall label | "AFTER" — `var(--font-mono)`, `var(--text-xs)`, uppercase, `var(--accent)`, `letter-spacing: 0.1em` |

### Governance Annotations (positioned along connector lines)

1. **Lead→Enrichment:** `[auto-captured]` — No manual entry
2. **Enrichment→Quote:** `[trace logged]` — Every enrichment step recorded
3. **Quote→Contract:** `[HITL gate]` — Human approval for discounts > threshold (mustard color)
4. **Contract→Invoice:** `[terms validated]` — Contract terms automatically checked
5. **Final:** `[✓ Full audit trail]` — Everything traceable end-to-end

---

## SVG Implementation Guide

### Dimensions

| Property | Value |
|----------|-------|
| Viewport | `viewBox="0 0 480 320"` per diagram |
| Container width | `50%` each, side by side (stacks on mobile) |
| Stage box | 120 × 64px |
| Connector line | 1.5px stroke (After), 1px dashed (Before) |
| Text size | Stage title: 12px, Detail: 11px, Annotation: 10px |
| Padding inside boxes | 8px |

### Color Variables (use CSS custom properties in SVG)

```css
/* Before diagram */
.before-box { stroke: var(--rule); fill: var(--paper); stroke-dasharray: 4 4; }
.before-text { fill: var(--ink-muted); }
.before-problem { fill: var(--wine); }
.before-line { stroke: var(--rule); stroke-dasharray: 4 4; }

/* After diagram */
.after-box { stroke: var(--accent); fill: var(--paper); stroke-width: 1.5; }
.after-text { fill: var(--ink); }
.after-success { fill: var(--accent); }
.after-line { stroke: var(--accent); stroke-width: 1.5; }
.after-hitl { fill: var(--mustard); }
```

### Accessibility

- Each SVG has `role="img"` and `aria-label` describing the diagram
- Before: `aria-label="Diagram showing a fragmented lead-to-cash workflow without Converge: manual data entry, no audit trail, 4-hour delays, dropped leads, and invoice mismatches"`
- After: `aria-label="Diagram showing a governed lead-to-cash workflow with Converge: automatic capture, agent enrichment, policy checks, HITL approval gates, and full audit trail"`
- Stage text is real `<text>` elements, not paths (screen-reader accessible)

### Animation (Optional Enhancement)

On scroll-into-view, the "After" diagram draws in:
1. Boxes appear with a subtle fade (0.3s, staggered 0.1s per box)
2. Connector lines draw left-to-right (`stroke-dashoffset` animation)
3. Checkmarks pop in last (0.2s scale from 0→1)

Before diagram is static — no animation. The contrast between static/broken and animated/governed reinforces the narrative.

**Implementation:** Use `IntersectionObserver` to trigger. CSS-only animation preferred over JS.

---

## Container Layout

```css
.diagramContainer {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-6);
  padding: var(--space-8) 0;
}

.diagramLabel {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: var(--space-3);
}

.diagramBefore .diagramLabel {
  color: var(--ink-muted);
}

.diagramAfter .diagramLabel {
  color: var(--accent);
}

@media (max-width: 640px) {
  .diagramContainer {
    grid-template-columns: 1fr;
  }
}
```

---

## What These Diagrams Are NOT

- Not a full architecture diagram (that's for the developer audience)
- Not a flowchart with decision diamonds (too technical)
- Not decorative — every element communicates a specific point
- Not animated for fun — animation serves the Before/After narrative only
