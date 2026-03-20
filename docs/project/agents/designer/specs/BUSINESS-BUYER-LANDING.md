# Design Spec: Business-Buyer Landing Page

**Route:** `/for/operations` (or `/business` -- confirm with Blake)
**Owner:** Rio Castellan (design), Blake Harmon (copy), Jules Carrera (build)
**Priority:** Critical (P0)
**Related issue:** "Business-buyer landing page on converge.zone"

---

## Purpose

The current homepage speaks to developers. This page speaks to ops leaders, RevOps managers, and business buyers who care about outcomes, not architecture. The narrative: "Your current automation is broken. Here's what fixed looks like. Here's how to start."

This is a second entry point to converge.zone -- a separate page, not a replacement for the homepage.

---

## Narrative Flow

The page tells a story in 6 sections:

1. **Hero** -- The pain ("Never lose a lead. Never miss a follow-up.")
2. **The Problem** -- What broken looks like (visual: chaotic workflow)
3. **The Solution** -- What Converge does (visual: converged workflow)
4. **Social Proof** -- Who uses it / pilot results
5. **Pricing** -- Quick tier summary + link to /pricing
6. **CTA** -- Start a pilot

---

## Section-by-Section Design

### 1. Hero

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│   Never lose a lead.                                 │
│   Never miss a follow-up.          (h1, --font-mono, │
│                                     --text-4xl)      │
│                                                      │
│   Converge is the trust layer       (--font-sans,    │
│   for your revenue operations.      --text-lg,       │
│                                     --ink-secondary)  │
│                                                      │
│   [Start Free Pilot]  [See How It Works]             │
│   (tertiary btn)      (secondary btn)                │
│                                                      │
│   padding: --space-20 top/bottom                     │
└──────────────────────────────────────────────────────┘
```

**Design notes:**
- Left-aligned text, not centered (business pages = authoritative, not ambient)
- No logo animation on this page. Clean, instant load.
- Hero occupies ~70vh on desktop

### 2. The Problem

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│  What broken looks like              (section label,  │
│                                      --font-mono,    │
│                                      --text-xs,      │
│                                      --ink-muted,    │
│                                      uppercase)      │
│                                                      │
│  [3-column pain point cards]                         │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Lost     │  │ Manual   │  │ No       │          │
│  │ leads    │  │ handoffs │  │ audit    │          │
│  │          │  │          │  │ trail    │          │
│  │ "43% of  │  │ "Teams   │  │ "When    │          │
│  │  leads..." │ │  spend..." │ │  things..." │      │
│  └──────────┘  └──────────┘  └──────────┘          │
│                                                      │
│  Background: --surface                               │
│  padding: --space-16 top/bottom                      │
└──────────────────────────────────────────────────────┘
```

**Pain point cards:**
- Background: `--paper`
- Border-left: 3px solid `--wine` (pain = wine color)
- Icon: simple line icon (optional, monochrome `--ink-muted`)
- Title: `--font-mono`, `--text-md`, `--ink`
- Body: `--font-sans`, `--text-base`, `--ink-secondary`
- Stat if available: `--font-mono`, `--text-2xl`, `--wine`

### 3. The Solution

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│  What Converge does                  (section label)  │
│                                                      │
│  [Before/After visual -- 2 columns]                  │
│                                                      │
│  BEFORE              AFTER                           │
│  ┌────────────┐     ┌────────────┐                  │
│  │ Fragmented │     │ Converged  │                  │
│  │ workflow   │     │ workflow   │                  │
│  │ diagram    │     │ diagram    │                  │
│  │ (chaotic,  │     │ (clean,    │                  │
│  │  wine/muted│     │  pine/     │                  │
│  │  tones)    │     │  accent)   │                  │
│  └────────────┘     └────────────┘                  │
│                                                      │
│  [3 value proposition cards]                         │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Every    │  │ Agents   │  │ Full     │          │
│  │ step     │  │ that     │  │ audit    │          │
│  │ tracked  │  │ follow   │  │ trail    │          │
│  │          │  │ rules    │  │          │          │
│  └──────────┘  └──────────┘  └──────────┘          │
│                                                      │
│  padding: --space-16 top/bottom                      │
└──────────────────────────────────────────────────────┘
```

**Value prop cards:**
- Same card structure as pain points, but:
- Border-left: 3px solid `--accent` (solution = pine green)
- Stat color: `--accent` instead of `--wine`

**Before/After diagrams:**
- Before: Muted palette (`--ink-muted`, `--rule`, dashed lines, `--wine` for problems)
- After: Active palette (`--accent`, `--pine`, solid lines, `--ink` for clarity)
- Same layout/structure -- the visual delta IS the story

### 4. Social Proof

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│  Who runs on Converge               (section label)   │
│                                                      │
│  [Testimonial / pilot result cards -- 2 column]      │
│                                                      │
│  "Lead response time dropped from   Company logo     │
│   4 hours to 12 minutes."          (if available)    │
│   -- Role, Company                                   │
│                                                      │
│  Background: --surface                               │
│  padding: --space-16 top/bottom                      │
└──────────────────────────────────────────────────────┘
```

**If no testimonials yet:** Use a placeholder section:
- "Join our pilot program" with a brief description
- Metrics framework placeholders: "We measure: lead response time, manual steps eliminated, cycle time reduction"
- Style: `--font-mono` for metric names, `--ink-muted`

### 5. Pricing Summary

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│  Simple pricing                     (section label)   │
│                                                      │
│  [Compact 3-tier summary -- single row]              │
│                                                      │
│  Free Pilot → Growth ($X/mo) → Enterprise (Custom)   │
│                                                      │
│  [See full pricing →]               (text link)       │
│                                                      │
│  padding: --space-12 top/bottom                      │
└──────────────────────────────────────────────────────┘
```

- Simplified version of the pricing page cards
- Each tier: name + price + one-line description
- Link to `/pricing` for full details

### 6. Final CTA

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│  Ready to stop losing leads?        (h2, --font-mono,│
│                                      --text-2xl)     │
│                                                      │
│  Start a free pilot. See results    (--font-sans,    │
│  in your first week.                --text-lg)       │
│                                                      │
│  [Start Free Pilot]                 (tertiary btn,   │
│                                      large)          │
│                                                      │
│  Background: --surface                               │
│  padding: --space-20 top/bottom                      │
│  text-align: center                                  │
└──────────────────────────────────────────────────────┘
```

---

## Design Principles for This Page

1. **Outcome-led, not feature-led.** "Never lose a lead" beats "multi-agent convergence runtime."
2. **Before/After is the key visual pattern.** Every section implies transformation.
3. **Left-aligned hero.** Business pages are authoritative. Center alignment is ambient/developer-oriented.
4. **Wine = pain, Pine = solution.** The color shift tells the story even if you don't read the words.
5. **No jargon above the fold.** "Agents," "convergence," "context" stay below the fold or get translated into business language.
6. **Scannable in 10 seconds.** Section labels, bold stats, clear CTAs.

---

## Responsive

- **Mobile (< 640px):** All grids stack to single column. Hero text drops to `--text-3xl`. Pain/value cards full-width. Before/After stacks vertically.
- **Tablet (640-1024px):** Pain/value cards 2-column. Before/After stays side-by-side.
- **Desktop (> 1024px):** Full 3-column cards. Wide Before/After. Content max-width `--max-width`.

---

## SEO / Meta

```html
<title>Converge for Operations - Never Lose a Lead</title>
<meta name="description" content="Converge is the trust layer for revenue operations. Automate lead-to-cash workflows with full audit trails and guaranteed follow-up.">
```

---

## Implementation Notes

- Create `src/app/pages/ForOperations.tsx` and `ForOperations.module.css`
- Add lazy-loaded route at `/for/operations` in `main.tsx`
- Add "For Operations" or "For Business" link to header nav (secondary nav area)
- Before/After diagrams: SVG inline or React components, not images (for accessibility + animation)
- Consider linking from the main homepage hero as a secondary CTA: "For business buyers →"
