# Design Spec: ProposedFact → Fact Pipeline Diagram

**Used in:** Security page, documentation, /mechanics, pitch decks
**Owner:** Rio Castellan (design)
**Priority:** High (core security concept — the most important trust boundary in Converge)

---

## Concept

This diagram answers: "What happens to data before Converge trusts it?"

Every piece of information in Converge starts as a `ProposedFact` — untrusted, unverified. It must pass through invariant validation before it becomes a `Fact` that can influence convergence outcomes. This is the single most important security boundary in the platform.

The visual must make this boundary feel **real and non-negotiable**. Not a suggestion. A wall.

---

## Diagram Structure

### Layout: Horizontal Pipeline (Left to Right)

```
  LLM Response    ProposedFact     Invariant       Accept/Reject     Fact
  (untrusted)                      Validation                      (trusted)
      │               │               │                │              │
      ▼               ▼               ▼                ▼              ▼
  ┌────────┐     ┌──────────┐   ┌──────────────┐   ┌────────┐   ┌────────┐
  │        │────▶│          │──▶│              │──▶│   ✓    │──▶│        │
  │  LLM   │     │ Proposed │   │  VALIDATION  │   │ Accept │   │  Fact  │
  │        │     │   Fact   │   │    GATE      │   │        │   │        │
  └────────┘     └──────────┘   │              │   ├────────┤   └────────┘
                                │  • Type check │   │   ✗    │
                                │  • Invariants │   │ Reject │──▶ ∅ (discarded)
                                │  • Policy     │   │        │
                                │  • Budget     │   └────────┘
                                └──────────────┘
```

Data flows left to right: untrusted → proposed → validated → accepted/rejected → trusted (or discarded).

---

## Color Coding

The pipeline uses a deliberate color shift to show the trust transition:

| Stage | Color | Meaning |
|-------|-------|---------|
| LLM Response | Wine (`#722f37`) | Untrusted, external, potentially dangerous |
| ProposedFact | Mustard (`#d4a017`) | Pending, under evaluation |
| Validation Gate | Steel (`#3b82f6`) | Neutral, mechanical, rule-based |
| Reject | Wine (`#722f37`) | Failed validation, discarded |
| Accept → Fact | Pine (`#2d5a3d`) | Trusted, verified, part of convergence |

This mirrors the brand color semantics: Wine = problem/danger, Pine = solution/trust.

---

## Stage Node Design

Each stage is a rectangle with a top color bar:

```
┌──────────────────────────┐
█ COLOR BAR (4px)          █  ← Stage color, full width
│                          │
│  STAGE NAME              │  ← var(--font-mono), var(--text-sm), uppercase, weight 600
│  Description text        │  ← var(--font-sans), var(--text-xs), var(--ink-muted)
│                          │
└──────────────────────────┘
```

| Property | Value |
|----------|-------|
| Padding | `var(--space-4)` |
| Border | `1px solid var(--rule-light)` |
| Border-top | `4px solid {stage-color}` |
| Background | `var(--paper)` |
| Border-radius | `0 0 4px 4px` |
| Min-width | `120px` |

---

## The Validation Gate

The center validation gate is the visual anchor. It's larger than other nodes and has a distinct treatment:

```css
.validationGate {
  padding: var(--space-5);
  border: 2px dashed var(--brand-steel);
  background: var(--surface);
  border-radius: 4px;
  min-width: 160px;
  position: relative;
}

.validationGate::before {
  content: "TRUST BOUNDARY";
  position: absolute;
  top: calc(-1 * var(--space-3));
  left: var(--space-4);
  font-family: var(--font-mono);
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--brand-steel);
  background: var(--surface);
  padding: 0 var(--space-2);
}
```

The dashed border communicates "boundary" — not solid like a container, but a checkpoint that data must pass through.

Validation checks listed inside:
- Type check
- Invariant validation
- Policy evaluation
- Budget verification

Each check rendered as a monospace bullet:

```css
.checkItem {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--ink-secondary);
  line-height: 1.8;
}
```

---

## Connection Arrows

Arrows between stages show data flow:

| Segment | Style | Color |
|---------|-------|-------|
| LLM → ProposedFact | Solid, 1.5px | Wine (muted) |
| ProposedFact → Validation | Solid, 1.5px | Mustard |
| Validation → Accept | Solid, 2px | Pine |
| Validation → Reject | Dashed, 1.5px | Wine |
| Accept → Fact | Solid, 2px | Pine |
| Reject → Discard | Dashed, 1px | Wine (muted) |

Arrow labels (optional, Detailed variant only):
- Between LLM and ProposedFact: "wrap as ProposedFact"
- Between Validation and Accept: "all checks pass"
- Between Validation and Reject: "invariant violated"

```css
.arrow {
  stroke-width: 1.5;
  fill: none;
  marker-end: url(#arrowhead);
}

.arrowTrusted {
  stroke: var(--accent);
  stroke-width: 2;
}

.arrowRejected {
  stroke: var(--brand-wine);
  stroke-dasharray: 6 4;
}
```

---

## Variants

### Compact (for inline use)

- 300×80px
- 5 circles connected by arrows (no boxes, no labels)
- Color-coded: Wine → Mustard → Steel → Pine or Wine
- Used in: cards, sidebar references, security page inline

### Standard (for documentation)

- 700×200px
- Full boxes with labels and descriptions
- Validation gate with check list
- Accept/Reject fork
- Used in: security page, documentation, /mechanics

### Detailed (for presentations)

- 900×300px
- Full boxes with labels, descriptions, and example data
- Validation gate with expanded check list
- Arrow labels
- Example: "LLM says: 'price is $50'" → ProposedFact → checks → Fact or Rejected
- Annotation: "This boundary cannot be bypassed. No code path converts ProposedFact → Fact without validation."
- Used in: pitch decks, security deep-dives

---

## Animation (Optional)

For web rendering:

1. **Entrance:** Pipeline stages appear left-to-right with 0.2s stagger
2. **Data flow:** A colored dot travels the pipeline from LLM to Fact (or forks to Reject)
   - Dot changes color as it passes through stages (wine → mustard → pine)
   - On reject path, dot turns wine and fades out
3. **Validation gate:** Subtle shimmer on the dashed border when data passes through

Trigger: scroll-triggered. Respects `prefers-reduced-motion`.

---

## SVG Implementation

Inline SVG, same conventions as other diagrams:

```html
<svg
  role="img"
  aria-label="Pipeline diagram showing how untrusted LLM output becomes a verified Fact through invariant validation in Converge"
  viewBox="0 0 700 200"
  class="pipelineDiagram"
>
  <!-- stages, arrows, validation gate -->
</svg>
```

- All colors via CSS custom properties
- Responsive: `width: 100%`, `max-width: 700px`
- `role="img"` + `aria-label` for accessibility

---

## Relationship to Other Diagrams

This is the third of four foundational diagrams:

1. **Convergence Loop:** The mechanism (how convergence works)
2. **Agent Collaboration:** The actors (who participates)
3. **This Diagram:** The data flow (how trust is established)
4. **Before/After Workflows:** The business impact (why it matters)

This diagram is especially important for the security page and for enterprise buyers who need to understand the LLM trust boundary.

---

## Implementation Notes for Jules

- **Component:** `src/app/components/diagrams/PipelineDiagram.tsx`
- SVG with CSS modules
- Accept props for variant (`compact | standard | detailed`)
- The validation gate's `::before` pseudo-element can't be done in SVG — use a `<text>` element positioned above the gate rectangle
- Consider making the reject path visually distinct (dashed + faded) so the "happy path" (accept → fact) reads clearly at a glance
- Animation gated behind `prefers-reduced-motion` media query
