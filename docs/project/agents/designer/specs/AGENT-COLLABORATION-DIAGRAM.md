# Design Spec: Agent Collaboration & Context Flow Diagram

**Used in:** Documentation, business-buyer landing page, pitch decks, /mechanics
**Owner:** Rio Castellan (design)
**Priority:** High (second foundational visual asset after convergence loop)

---

## Concept

The convergence loop shows the mechanism. This diagram shows the actors. Multiple agents contribute to a shared context, each with their own expertise, each reading and writing to keys they own. The context flows toward convergence through governed collaboration, not centralized control.

This diagram answers: "How do agents actually work together in Converge?"

---

## Diagram Structure

### Layout: Radial Hub-and-Spoke

The shared context is the center. Agents are arranged radially around it. Each agent has a lane connecting it to the context — representing the keys it reads and writes.

```
                      Agent A
                     (Pricing)
                        │
                        │ writes: price_quote
                        │ reads: customer_data
                        ▼
        Agent D ──── SHARED ──── Agent B
       (Approval)    CONTEXT    (Inventory)
                        ▲
                        │ writes: stock_level
                        │ reads: price_quote
                        │
                      Agent C
                    (Fulfillment)
```

### Visual Interpretation

- **Center:** Rounded rectangle labeled "Shared Context" — same center dot motif as the convergence loop. Background: `var(--surface)`, border: `2px solid var(--accent)`.
- **Agents:** Rectangular nodes arranged at cardinal + intercardinal positions (4-6 agents depending on variant).
- **Lanes:** Lines connecting each agent to the center context. Annotated with key names they read/write.
- **Direction:** Arrows on lanes indicate read (toward agent) and write (toward context).

---

## Color Coding

Each agent gets a muted brand color to distinguish it, drawn from the design system:

| Agent Role | Color | Token |
|------------|-------|-------|
| Data/Input agents | Steel (`#3b82f6`) | `var(--brand-steel)` |
| Logic/Decision agents | Pine (`#2d5a3d`) | `var(--accent)` |
| Validation/Gate agents | Mustard (`#d4a017`) | `var(--brand-mustard)` |
| Output/Action agents | Wine (`#722f37`) | `var(--brand-wine)` |

The shared context center uses `var(--accent)` border — the same pine green that signals "solution" in the brand system.

---

## Agent Node Design

```
┌──────────────────────┐
│  AGENT NAME          │  ← var(--font-mono), var(--text-sm), uppercase, weight 600
│  Role description    │  ← var(--font-sans), var(--text-xs), var(--ink-muted)
└──────────────────────┘
```

| Property | Value |
|----------|-------|
| Padding | `var(--space-3) var(--space-4)` |
| Border | `1px solid var(--rule-light)` |
| Border-left | `3px solid {agent-color}` |
| Background | `var(--paper)` |
| Border-radius | `4px` |

---

## Lane Annotations

Each lane shows the context keys that agent reads and writes:

```css
.laneLabel {
  font-family: var(--font-mono);
  font-size: 10px;
  fill: var(--ink-muted);
  letter-spacing: 0.05em;
}
```

- Write annotations: positioned near the context center, pointing inward
- Read annotations: positioned near the agent node, pointing outward
- Arrow style: `stroke-width: 1.5`, `marker-end: url(#arrowhead)`, color matches agent

---

## Context Center Node

```
┌─────────────────────────┐
│         ●               │  ← Center dot from logo
│    SHARED CONTEXT       │  ← var(--font-mono), var(--text-sm), uppercase
│                         │
│  customer_data: {...}   │  ← var(--font-mono), 10px, var(--ink-muted)
│  price_quote: {...}     │
│  stock_level: {...}     │
│  approval: pending      │
└─────────────────────────┘
```

| Property | Value |
|----------|-------|
| Padding | `var(--space-5)` |
| Border | `2px solid var(--accent)` |
| Background | `var(--surface)` |
| Border-radius | `4px` |
| Min-width | `200px` |

The key listing inside the context is optional — included in the Detailed variant, omitted in Compact.

---

## Variants

### Compact (for inline use)

- 200×200px
- 4 agents as labeled dots (no boxes)
- Center dot only, no context box
- No lane annotations
- Used in: cards, sidebar illustrations, icon-sized references

### Standard (for documentation)

- 600×400px
- 4 agents with full node boxes
- Center context box with key listing
- Lane annotations visible
- Used in: documentation pages, blog posts, /mechanics

### Detailed (for presentations)

- 900×600px
- 6 agents with full node boxes
- Center context box with key listing and convergence state indicator
- Lane annotations with read/write icons
- HITL gate indicator (mustard dashed border) on approval agent
- Governance annotation: "Policy agent validates every write"
- Used in: pitch decks, detailed architecture pages

---

## Animation (Optional)

For web rendering, subtle animation reinforces the collaboration concept:

1. **Entrance:** Agents fade in from outside, lanes draw inward toward context (0.5s stagger)
2. **Steady state:** Subtle pulse on the center dot (same as convergence loop center dot)
3. **Data flow:** Small dots travel along lanes toward/from context (slow, continuous, 3s loop)

Trigger: scroll-triggered via Intersection Observer. No autoplay.

```css
@keyframes dataFlow {
  0% { offset-distance: 0%; opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { offset-distance: 100%; opacity: 0; }
}
```

---

## SVG Implementation

Render as inline SVG with CSS custom properties. Not raster.

```html
<svg
  role="img"
  aria-label="Diagram showing multiple agents collaborating through a shared context in Converge"
  viewBox="0 0 600 400"
  class="agentCollabDiagram"
>
  <!-- agents, lanes, context center -->
</svg>
```

- All colors via CSS custom properties (theme-able)
- `role="img"` + `aria-label` for accessibility
- Responsive: `width: 100%`, `max-width: 600px` (Standard variant)

---

## Relationship to Other Diagrams

- **Convergence Loop Diagram:** Shows the mechanism (Propose → Validate → Accept/Reject → Converge)
- **This Diagram:** Shows the actors and their relationships to shared context
- **Workflow Diagrams (Before/After):** Shows a specific business workflow transformed by Converge
- **Pipeline Diagram (next):** Shows the data flow from ProposedFact → Fact

These four diagrams form a complete visual story: who (agents), how (loop), what happens to data (pipeline), and why it matters (before/after).

---

## Implementation Notes for Jules

- **Component:** `src/app/components/diagrams/AgentCollabDiagram.tsx`
- SVG with CSS modules for styling
- Accept props for variant (`compact | standard | detailed`) and agent count
- Lead-to-Cash demo agents make a good default data set (CRM, Quoting, Inventory, Approval, Invoicing, Fulfillment)
- Animation optional — gated behind `prefers-reduced-motion` media query
