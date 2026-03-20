# Design Spec: Pricing Page

**Route:** `/pricing`
**Owner:** Rio Castellan (design), Blake Harmon (copy/numbers), Morgan Vale (final pricing approval)
**Implements:** Jules Carrera
**Priority:** Critical (P0)
**Related issue:** "Pricing page -- draft 2-3 tiers and publish on converge.zone"

---

## Purpose

Make Converge look real. A pricing page signals "we're a business, not a research project." Even if the numbers change, the structure forces the right conversations with prospects. The page should be clear, scannable, and drive prospects to start a pilot.

---

## Layout

### Desktop (> 1024px)

```
[Header -- standard site header]

[Page Title Section]
  "Pricing" (h1, --font-mono, --text-3xl, centered)
  "Start with a free pilot. Scale when you're ready." (subtitle, --text-lg, --ink-secondary, centered)

[Tier Cards -- 3 columns, equal width, centered in --max-width]
  [Starter]        [Professional]    [Enterprise]

[FAQ Section -- single column, accordion]

[CTA Section]
  "Ready to start?" + primary button

[Footer -- standard site footer]
```

### Mobile (< 640px)

- Tier cards stack vertically, full-width
- FAQ accordion unchanged
- CTA section full-width

---

## Tier Card Design

Each card is a vertical block with clear hierarchy.

### Card Structure

```
┌─────────────────────────────────┐
│  TIER NAME         (--font-mono, --text-sm, uppercase, --ink-muted)
│
│  $0 / month        (--font-mono, --text-3xl, --ink)
│  or "Custom"
│
│  Brief description  (--font-sans, --text-base, --ink-secondary)
│  1-2 lines max
│
│  ─────────────────  (--rule-light, 1px)
│
│  ✓ Feature line 1   (--font-sans, --text-base)
│  ✓ Feature line 2
│  ✓ Feature line 3
│  ✓ Feature line 4
│  ...
│
│  [CTA Button]       (full-width within card)
│
└─────────────────────────────────┘
```

### Card Styling

| Property | Value |
|----------|-------|
| Background | `--paper` (default) or `--surface` (highlighted tier) |
| Border | 1px solid `--rule` |
| Highlighted border | 2px solid `--accent` |
| Padding | `--space-8` |
| Gap between cards | `--space-6` |
| Max card width | 320px each |

### Highlighted Tier

The "Professional" tier (middle) gets emphasis:
- 2px border in `--accent` instead of `--rule`
- Small "Recommended" badge above: `--font-mono`, `--text-xs`, uppercase, `--accent` bg, `--paper` text, padding `--space-1` `--space-3`
- No other visual differences. Let the border do the work.

### Tier Content (placeholder -- Blake/Morgan to finalize)

**Starter (Free)**
- $0 / month
- "Try Converge on a real workflow. No commitment."
- 1 workspace, 2 domain packs, 500 runs/month, community support
- CTA: "Start Free" (secondary button)

**Professional**
- $349 / month
- "For teams running production workflows."
- 3 workspaces, 4 domain packs, 5K runs/month, Lead-to-Cash blueprint, email support, analytics dashboard
- CTA: "Get Started" (tertiary button -- accent bg)

**Enterprise**
- Custom
- "For organizations with compliance and scale requirements."
- Unlimited everything, custom domain packs, dedicated support, SLA, SSO, audit logs
- CTA: "Contact Sales" (secondary button)

---

## Feature Comparison (Optional Section)

Below the cards, an optional expandable comparison table:

| Feature | Starter | Professional | Enterprise |
|---------|---------|--------------|------------|
| Workspaces | 1 | 3 | Unlimited |
| Domain Packs | 2 | 4 | Custom |
| Runs / month | 500 | 5,000 | Unlimited |
| ... | ... | ... | ... |

Style: standard table with `--font-mono` headers, `--text-sm`, `--rule-light` borders.

---

## FAQ Section

Accordion-style. Each item:
- Question: `--font-mono`, `--text-md`, `--ink`
- Answer: `--font-sans`, `--text-base`, `--ink-secondary`
- Toggle: "+" / "-" indicator, right-aligned
- Divider: `--rule-light` between items

Suggested questions (Blake to finalize):
1. What counts as a "run"?
2. Can I change plans?
3. What's included in a domain pack?
4. Do you offer annual billing?
5. What happens when I hit my run limit?

---

## Interactions

- Tier cards: subtle `--surface-hover` on hover (desktop only)
- CTA buttons: standard button hover states per design system
- FAQ items: smooth expand/collapse, `--transition-base`
- No animations on page load. Clean, instant rendering.

---

## SEO / Meta

```html
<title>Pricing - Converge</title>
<meta name="description" content="Simple, transparent pricing for Converge. Start with a free pilot, scale to production.">
```

---

## Implementation Notes

- Create `src/app/pages/Pricing.tsx` and `Pricing.module.css`
- Add lazy-loaded route at `/pricing` in `main.tsx`
- Add "Pricing" link to site header navigation
- Use CSS Grid for the 3-column tier layout (auto-fit, minmax)
- No external dependencies needed
