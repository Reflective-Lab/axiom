# Design Spec: Convergence Loop Diagram

**Used in:** Homepage, documentation, presentations, blog posts, pitch decks
**Owner:** Rio Castellan (design)
**Priority:** High (foundational visual asset — used everywhere)

---

## Concept

The convergence loop is Converge's core abstraction. It's how autonomous agents reach shared truth without central control. This diagram must make that concept intuitive in under 5 seconds.

**The loop:** `Propose → Validate → Accept/Reject → Converge`

This is not a flowchart. It's a cycle. The visual must feel like rotation, like things coming together — not a linear pipeline.

---

## Diagram Structure

### The Cycle (4 Stages)

```
                    ┌───────────┐
                    │  PROPOSE  │
                    │           │
                    └─────┬─────┘
                          │
                          ▼
        ┌─────────┐       ●       ┌──────────┐
        │ CONVERGE│◄──────┤──────►│ VALIDATE │
        │         │       │       │          │
        └─────────┘       │       └──────────┘
                          │
                          ▼
                    ┌───────────┐
                    │  ACCEPT / │
                    │  REJECT   │
                    └───────────┘
```

### Visual Interpretation

The diagram is a circle, not a rectangle grid. Four stage nodes sit at compass points around a central convergence point (●).

```
              PROPOSE
                │
                ▼
    CONVERGE ── ● ── VALIDATE
                │
                ▼
          ACCEPT/REJECT
```

Arrows flow clockwise: Propose (top) → Validate (right) → Accept/Reject (bottom) → Converge (left) → back to Propose.

The center dot (●) represents shared truth — the convergence point. It's the same center dot from the logo. This visual connection is intentional.

---

## Layout & Dimensions

### SVG Viewport

| Property | Value |
|----------|-------|
| Viewport | `viewBox="0 0 400 400"` |
| Center point | `(200, 200)` |
| Orbit radius | 120px (center of each stage node to center point) |
| Stage node | 100 × 48px rounded rectangle |
| Center dot | 12px diameter circle |
| Arrow path | Curved (cubic bezier), follows the circle |

### Stage Positions (clockwise from top)

| Stage | Position | Angle |
|-------|----------|-------|
| Propose | top center (200, 80) | 0° |
| Validate | right center (320, 200) | 90° |
| Accept/Reject | bottom center (200, 320) | 180° |
| Converge | left center (80, 200) | 270° |

---

## Color Assignments

Each stage has a distinct brand color assignment:

| Stage | Color | Token | Rationale |
|-------|-------|-------|-----------|
| Propose | Mustard | `var(--mustard)` | Proposal = speculative, attention-requiring |
| Validate | Steel | `var(--steel)` | Validation = systematic, data-driven |
| Accept/Reject | Wine | `var(--wine)` | Decision = consequential, boundary-defining |
| Converge | Pine | `var(--accent)` | Convergence = success, truth established |
| Center dot | Ink | `var(--ink)` | Shared truth = permanent, authoritative |

### Stage Node Styles

```css
/* All stage nodes */
.stageNode {
  rx: 4;                          /* 4px border-radius */
  stroke-width: 1.5;
  fill: var(--paper);
}

/* Per stage */
.stagePropose  { stroke: var(--mustard); }
.stageValidate { stroke: var(--steel); }
.stageDecide   { stroke: var(--wine); }
.stageConverge { stroke: var(--accent); }

/* Center dot */
.centerDot {
  r: 6;
  fill: var(--ink);
}
```

### Arrow Path Styles

Arrows are curved paths following the circular orbit. Each arrow matches the color of the stage it points TO.

```css
.arrowToValidate { stroke: var(--steel); }
.arrowToDecide   { stroke: var(--wine); }
.arrowToConverge { stroke: var(--accent); }
.arrowToPropose  { stroke: var(--mustard); }
```

Arrow stroke: 1.5px, with arrowhead marker (`marker-end`).

---

## Typography

| Element | Font | Size | Weight | Color |
|---------|------|------|--------|-------|
| Stage label | `var(--font-mono)` | 11px | 600 | Same as stage stroke color |
| Sub-label (optional) | `var(--font-sans)` | 9px | 400 | `var(--ink-muted)` |
| Center label "shared truth" | `var(--font-mono)` | 9px | 400 | `var(--ink-muted)` |

Stage labels sit inside the rounded rectangles, centered.

Optional sub-labels beneath each stage (not inside the box):
- Propose: "Agent submits a ProposedFact"
- Validate: "Validators check contracts"
- Accept/Reject: "Pass → Fact. Fail → Rejected."
- Converge: "Shared truth updated"

These sub-labels are shown in the detailed/documentation variant but hidden in the compact variant.

---

## Variants

### 1. Compact (icon-size)

- Used in: Cards, navigation, small contexts
- Size: 80×80px or smaller
- Show: 4 colored dots at compass points + center dot + curved arrow suggesting rotation
- No text labels
- Recognizable at a glance as "the convergence loop"

### 2. Standard (section hero)

- Used in: Homepage sections, landing pages, blog headers
- Size: 400×400px
- Show: Full diagram with stage labels, arrows, center dot
- No sub-labels
- Clean, impactful

### 3. Detailed (documentation)

- Used in: Technical docs, architecture pages, presentations
- Size: 400×480px (extra height for sub-labels)
- Show: Full diagram + sub-labels explaining each stage
- May include additional annotations for agent context flow

---

## Animation (Standard and Detailed variants)

### On Scroll Into View

1. **Center dot** appears first (fade in, 0.2s)
2. **Stage nodes** appear one at a time clockwise (fade + translate from center outward, 0.3s each, staggered 0.15s)
3. **Arrow paths** draw in clockwise (`stroke-dashoffset` animation, 0.4s each, staggered 0.1s after their origin node)
4. **Labels** fade in last (0.2s, after their node appears)

Total animation: ~1.5s

### Continuous (Optional)

After the entrance animation, a subtle continuous hint:
- Center dot pulses gently (opacity 0.8 → 1.0, 2s cycle)
- Nothing else moves — the loop is stable, not frenetic

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  /* No entrance animation — show everything immediately */
  /* No continuous pulse */
}
```

---

## Accessibility

- SVG has `role="img"`
- `aria-label="Convergence loop diagram: Propose leads to Validate, which leads to Accept or Reject, which leads to Converge, which cycles back to Propose. Shared truth is established at the center."`
- All text is real `<text>` elements (not paths)
- Colors are not the only differentiator — each stage also has its label

---

## Usage in Different Contexts

| Context | Variant | Notes |
|---------|---------|-------|
| converge.zone homepage | Standard | Section hero for "How it works" |
| `/for/operations` landing | Standard | Simplified version, business language sub-labels |
| Technical docs (Core page) | Detailed | Full annotations |
| Blog post headers | Standard | Static, no animation |
| Pitch deck slides | Standard | Dark background variant (invert paper → dark bg) |
| Card/nav icons | Compact | Recognizable at small size |
| Favicon association | Compact | Echoes the logo's convergence theme |

---

## Relationship to Logo

The convergence loop diagram intentionally echoes the logo:
- **Logo:** 8 lines converging to a center dot (static, identity)
- **Loop diagram:** 4 stages orbiting a center dot (dynamic, conceptual)

Both share the center dot as the symbol of shared truth. The logo is the brand mark. The loop diagram is the concept visualization. They're related but distinct.

---

## Implementation Notes for Jules

- Build as a React component: `<ConvergenceLoop variant="standard" animate={true} />`
- SVG inline (not external file) for CSS variable access
- Use `IntersectionObserver` for scroll-triggered animation
- Export both a component and a static SVG for non-React contexts (docs, presentations)
- Place in `src/app/components/diagrams/ConvergenceLoop.tsx`
