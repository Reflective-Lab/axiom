# Design Spec: About Us / Team Page

**Route:** `/about`
**Owner:** Rio Castellan (design), Jules Carrera (build), Blake Harmon (copy approval), Caroline Ashford (editorial bios)
**Priority:** High
**Related issue:** `1a5980fd-3e5b-4441-a5fd-e990151e9967`

---

## Purpose

Converge is a team, not a product. Before design partners sign, they want to know who they're working with. This page makes the team visible and credible: real people, real structure, real accountability.

This page also serves a second purpose — demonstrating that Converge practices what it preaches. The org structure itself is a convergence system: clear roles, explicit reporting lines, defined ownership.

---

## Relationship to /editorial

The existing `/editorial` page covers the three external editorial voices (Caroline, Alice, Bob) with deep persona detail. The About Us page references them but does **not** duplicate that content. Instead, it links to `/editorial` for the full editorial team story.

---

## Page Structure

### Section 1: Hero

**Layout:** Left-aligned (consistent with business pages)
**Tagline:** `About Converge` (mono, uppercase, accent)
**Headline:** `The team building trust into automation`
**Subhead:** 1-2 sentences — "Converge is built by engineers, designers, and operators who believe AI should be governed, not hoped. Meet the team." (Blake approves final copy.)

No hero image. The team grid IS the visual.

**CSS pattern:**
```css
.header {
  text-align: left;           /* Left-aligned, not centered */
  padding: var(--space-12) 0;
  border-bottom: 1px solid var(--rule-light);
}
```

---

### Section 2: Leadership

**Section title:** `Leadership`
**Layout:** 3-column grid (`repeat(3, 1fr)`, collapses to 1-col on mobile)

Three cards:

| Name | Title | One-liner |
|------|-------|-----------|
| Morgan Vale | CEO | Sets direction, owns strategy and fundraising |
| Ren Akiyama | VP of Engineering | Owns the platform, the architecture, the ship schedule |
| Blake Harmon | VP of Marketing & Sales | Owns the story, the funnel, the first 3-4 design partners |

**Card design:** Compact variant (no portrait illustrations yet — use initials avatar).

```
┌─────────────────────────────┐
│  [MV]  Morgan Vale          │
│        CEO                  │
│                             │
│  Sets direction, owns       │
│  strategy and fundraising.  │
└─────────────────────────────┘
```

**Initials avatar:**
- 48×48px circle
- Background: `var(--surface)`
- Border: `1px solid var(--rule-light)`
- Text: `var(--font-mono)`, `var(--text-sm)`, `var(--ink-secondary)`, centered
- No photos, no illustrations — consistent, egalitarian, low-maintenance

---

### Section 3: Engineering

**Section title:** `Engineering`
**Subtitle:** `Reports to Ren Akiyama, VP of Engineering`
**Layout:** 3×2 grid (`repeat(auto-fit, minmax(260px, 1fr))`)

Six cards:

| Name | Title | One-liner |
|------|-------|-----------|
| Eli Marsh | Founding Engineer | Core convergence loop, proof of correctness |
| Kira Novak | Senior Rust Developer | Trait system, type safety, crate architecture |
| Jules Carrera | Frontend Developer | converge.zone, application UI, interactive demos |
| Sam Okafor | QA Engineer | Test coverage, integration testing, quality gates |
| Dex Tanaka | DevOps Release Engineer | CI/CD, release automation, infrastructure |
| Ava Petrov | Security Engineer | Threat modeling, access controls, compliance |

Same card design as Leadership. Initials avatar + name + title + one-liner.

---

### Section 4: Marketing & Design

**Section title:** `Marketing & Design`
**Subtitle:** `Reports to Blake Harmon, VP of Marketing & Sales`
**Layout:** Single card (just me), left-aligned, not centered

| Name | Title | One-liner |
|------|-------|-----------|
| Rio Castellan | Designer | Brand identity, design system, visual storytelling |

---

### Section 5: Editorial Team (External)

**Section title:** `Editorial Team`
**Subtitle:** `External voices. Reports to Blake Harmon via Caroline Ashford.`
**Layout:** 3-column grid

Three cards, more compact than the `/editorial` page:

| Name | Title | One-liner |
|------|-------|-----------|
| Caroline Ashford | Editor-in-Chief & Producer | Owns the blog voice, enforces claim integrity |
| Alice Mercer | Systems Pragmatist | Architecture, determinism, production constraints |
| Bob Calder | Builder-Experimentalist | Demos, experiments, benchmarks, fast iteration |

**Link:** At the bottom of this section, a text link: `Read the full editorial team story →` linking to `/editorial`.

---

### Section 6: How We Work

**Section title:** `How we work`
**Layout:** Left-aligned prose block, max-width 42rem

3-4 short paragraphs about how the team operates:
- Small team, high autonomy, explicit ownership
- Every role has a clear scope — no ambiguity about who owns what
- We use Converge's own governance model internally (dogfooding)
- Open to design partners who want to shape the platform with us

**CTA button:** `Start a pilot conversation →` linking to contact/pilot flow.

---

## Card Component Spec

The team member card is the primary reusable component.

```
┌─────────────────────────────────┐
│  [AV]  Name                     │
│        Title                    │
│                                 │
│  One-liner description text     │
│  that may wrap to two lines.    │
└─────────────────────────────────┘
```

### Dimensions & Spacing

| Property | Value |
|----------|-------|
| Card padding | `var(--space-5)` (1.25rem) |
| Card border | `1px solid var(--rule-light)` |
| Card border-radius | `4px` |
| Card background | `var(--paper)` |
| Card gap (internal) | `var(--space-3)` |
| Avatar size | 48×48px |
| Avatar border-radius | `50%` (circle) |
| Avatar background | `var(--surface)` |
| Avatar text | `var(--font-mono)`, `var(--text-sm)`, `var(--ink-secondary)` |
| Name | `var(--font-mono)`, `var(--text-md)`, weight 600 |
| Title | `var(--font-mono)`, `var(--text-xs)`, uppercase, `letter-spacing: 0.08em`, `var(--ink-muted)` |
| One-liner | `var(--font-sans)`, `var(--text-sm)`, `var(--ink-secondary)`, `line-height: 1.6` |

### Card Header Layout

```css
.cardHeader {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
```

### Avatar

```css
.avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--surface);
  border: 1px solid var(--rule-light);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--ink-secondary);
  flex-shrink: 0;
}
```

---

## Org Structure Visualization

Between the Hero and the first team section, include a simple org chart.

**Layout:** Horizontal tree, text-only, no boxes or connectors — just indented text with thin rules.

```
Morgan Vale, CEO
├── Ren Akiyama, VP of Engineering
│   ├── Eli Marsh — Founding Engineer
│   ├── Kira Novak — Senior Rust Developer
│   ├── Jules Carrera — Frontend Developer
│   ├── Sam Okafor — QA Engineer
│   ├── Dex Tanaka — DevOps Release Engineer
│   └── Ava Petrov — Security Engineer
└── Blake Harmon, VP of Marketing & Sales
    ├── Rio Castellan — Designer
    └── Caroline Ashford — Editor-in-Chief (external)
        ├── Alice Mercer — Systems Pragmatist
        └── Bob Calder — Builder-Experimentalist
```

**Rendering:** Use `<pre>` with `var(--font-mono)`, `var(--text-sm)`, `var(--ink-secondary)`. Wrap in a bordered container:

```css
.orgChart {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--ink-secondary);
  line-height: 1.8;
  padding: var(--space-5);
  border: 1px solid var(--rule-light);
  border-radius: 4px;
  background: var(--paper);
  overflow-x: auto;
  white-space: pre;
}
```

---

## Responsive Behavior

| Breakpoint | Change |
|------------|--------|
| > 768px | 3-column grid for leadership + editorial, 3×2 for engineering |
| 640–768px | 2-column grid |
| < 640px | Single column, all sections stack. Org chart scrolls horizontally. Title drops to `var(--text-2xl)`. Section padding reduces to `var(--space-8)`. |

---

## Navigation

- Add `/about` to the header nav (right side, before auth)
- Add `/about` to the footer links
- Route registered in `main.tsx` with lazy loading

---

## Accessibility

- All initials avatars are `aria-hidden="true"` (decorative)
- Card names are `<h3>` within each section's `<h2>` hierarchy
- Org chart `<pre>` has `role="img"` and `aria-label="Converge organization chart showing team reporting structure"`
- All text meets WCAG AA contrast ratios (already guaranteed by token system)
- Link to `/editorial` has descriptive text, not "click here"

---

## What This Page Is NOT

- Not a careers page (we're not hiring externally)
- Not a deep bio page (that's `/editorial` for the editorial team)
- Not a photo gallery (no portraits — initials avatars keep it egalitarian and easy to maintain)
- Not marketing copy — it's factual, structured, and brief

---

## Implementation Notes for Jules

- **Component:** `src/app/pages/About.tsx` + `About.module.css`
- **Extract** the team member card as a reusable component if it'll be used elsewhere
- Follow the EditorialTeam.tsx patterns for section structure
- The org chart is a static `<pre>` block — no JS needed
- Lazy-load in `main.tsx`: `const About = lazy(() => import('./app/pages/About').then(m => ({ default: m.About })));`
- Add route: `<Route path="about" element={<Suspense fallback={<Loader />}><About /></Suspense>} />`
