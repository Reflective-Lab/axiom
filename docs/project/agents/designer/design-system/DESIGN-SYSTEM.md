# Converge Design System

**Version:** 1.1.0
**Owner:** Rio Castellan, Designer
**Last updated:** 2026-03-13

---

## Brand Identity

### Logo

The Converge logo is 8 radial lines converging toward a central point. Each line represents an independent agent; the center represents the shared truth they converge upon. The lines use four brand colors in alternating pairs.

**Mark (icon only):** 8 converging lines + center dot. Used as favicon, app icon, social avatar.

**Wordmark (full logo):** Mark + "Converge Zone" in Georgia serif. Used on website header, presentations, documents.

### Logo Usage Rules

- **Minimum clear space:** 1x the diameter of the center dot on all sides.
- **Minimum size:** 24px for icon, 120px wide for full wordmark.
- **Backgrounds:** Use on `--paper` (#f5f4f0) or white. For dark backgrounds, invert center dot to white and use lightened brand colors.
- **Prohibited modifications:** Do not rotate the mark. Do not change individual line colors. Do not add effects (shadows, gradients, 3D). Do not stretch or compress.

---

## Color Palette

### Core

| Token | Hex | Usage |
|-------|-----|-------|
| `--paper` | `#f5f4f0` | Page background. Warm off-white. |
| `--ink` | `#111111` | Primary text, headings, high-contrast elements. |
| `--ink-secondary` | `#3a3a3a` | Body text, secondary content. |
| `--ink-muted` | `#666666` | Captions, metadata, disabled states. |

### Structure

| Token | Hex | Usage |
|-------|-----|-------|
| `--rule` | `#c8c6c0` | Borders, dividers. |
| `--rule-light` | `#dbd9d3` | Subtle separators. |
| `--surface` | `#eae9e4` | Card backgrounds, hover states, code blocks. |
| `--surface-hover` | `#e0dfda` | Interactive surface hover. |

### Brand

| Token | Hex | Usage |
|-------|-----|-------|
| `--pine` | `#1a3c34` | Primary brand. Logo lines (E, W). Deep trust green. |
| `--accent` | `#2d5a3d` | CTAs, links, active states. Lighter pine. |
| `--accent-light` | `#3d7a5d` | Hover on accent elements. |
| `--wine` | `#722f37` | Logo lines (N, S). Warmth, emphasis. |
| `--mustard` | `#d4a017` | Logo lines (NW, SW). Highlights, badges, warnings. |
| `--steel` | `#3b82f6` | Logo lines (NE, SE). Technical, links, info states. |

### Semantic Surfaces

| Token | Hex | Usage |
|-------|-----|-------|
| `--surface-success` | `#f0f8f3` | Light success background (active agent, result card). |
| `--surface-success-muted` | `#e8f0ea` | Muted success background (completed agent). |

### Terminal (Dark Context)

| Token | Hex | Usage |
|-------|-----|-------|
| `--terminal-bg` | `#0d1117` | Terminal/log panel background. |
| `--terminal-text` | `#c9d1d9` | Terminal default text. |
| `--terminal-border` | `#30363d` | Terminal panel border. |
| `--terminal-green` | `#7ee787` | Terminal syntax: facts, success, accepted values. |
| `--terminal-blue` | `#79c0ff` | Terminal syntax: invariants, agent names, links. |
| `--terminal-purple` | `#d2a8ff` | Terminal syntax: actions, proposals, mutations. |
| `--terminal-red` | `#ff7b72` | Terminal syntax: errors, HITL barriers, rejections. |

### Wide Layout

| Token | Value | Usage |
|-------|-------|-------|
| `--max-width-wide` | `1200px` | Demo pages and other content needing extra width. |

### Status

| Token | Hex | Usage |
|-------|-----|-------|
| `--status-success` | `#2d5a3d` | Converged, healthy, complete. |
| `--status-warning` | `#6b5b3d` | Pending, caution, in-progress. |
| `--status-error` | `#5a3d3d` | Failed, blocked, diverged. |

### Color Rules

1. **Brand colors are functional, not decorative.** Each has a specific job. Do not use brand colors for ornamentation.
2. **Status colors are universal.** Green = success, amber = warning, red = error. Never override.
3. **Contrast ratios:** All text must meet WCAG AA (4.5:1 for normal text, 3:1 for large text). `--ink` on `--paper` = 15.3:1. `--ink-muted` on `--paper` = 5.1:1.
4. **No new colors without design review.** The palette is intentionally constrained.

---

## Typography

### Font Stack

| Token | Family | Usage |
|-------|--------|-------|
| `--font-mono` | IBM Plex Mono | Headings, CTAs, code, technical content, the "voice" of Converge. |
| `--font-sans` | Inter | Body text, descriptions, long-form reading. |

### Type Scale

| Token | Size | Usage |
|-------|------|-------|
| `--text-xs` | 0.65rem (10.4px) | Labels, badges, fine print. |
| `--text-sm` | 0.75rem (12px) | Buttons, captions, metadata. |
| `--text-base` | 0.875rem (14px) | Body text (default). |
| `--text-md` | 1rem (16px) | Emphasized body, subheadings. |
| `--text-lg` | 1.125rem (18px) | Lead paragraphs, subtitles. |
| `--text-xl` | 1.25rem (20px) | Section headings (h3). |
| `--text-2xl` | 1.5rem (24px) | Page subheadings (h2). |
| `--text-3xl` | 2rem (32px) | Page titles (h1). |
| `--text-4xl` | 2.5rem (40px) | Hero headlines. |

### Typography Rules

1. **Headings use `--font-mono`.** This is the "engineered" voice. Precise, technical, trustworthy.
2. **Body uses `--font-sans`.** Readable, clean, doesn't compete with headings.
3. **Weight hierarchy:** 600 for headings, 500 for emphasis, 400 for body.
4. **Line height:** 1.3 for headings, 1.6 for body text. Generous spacing signals clarity.
5. **Letter spacing:** -0.02em for large headings (text-3xl+). Default for everything else.
6. **No underlines** except on text links. Never underline headings.

---

## Spacing

### Scale (4px base)

| Token | Value | Common Use |
|-------|-------|------------|
| `--space-1` | 4px | Tight padding (badge internals). |
| `--space-2` | 8px | Icon-to-text gap, inline spacing. |
| `--space-3` | 12px | Button padding (vertical). |
| `--space-4` | 16px | Card padding, standard gap. |
| `--space-6` | 24px | Button padding (horizontal), section gap. |
| `--space-8` | 32px | Component separation. |
| `--space-10` | 40px | Small section padding (mobile). |
| `--space-12` | 48px | Section dividers. |
| `--space-16` | 64px | Section padding (desktop). |
| `--space-20` | 80px | Hero padding, major sections. |

### Layout

| Token | Value | Usage |
|-------|-------|-------|
| `--max-width` | 64rem (1024px) | Content max-width. |
| `--header-height` | 3.5rem (56px) | Fixed header height. |

### Spacing Rules

1. **Always use tokens.** Never hardcode pixel values.
2. **8px grid alignment.** All major dimensions should snap to 8px multiples.
3. **Whitespace is a feature.** Resist filling space. Converge values clarity.

---

## Components

### Buttons

Three tiers:

| Class | Style | Usage |
|-------|-------|-------|
| `.primary` | Ink bg, paper text | Primary actions (CTA, submit). |
| `.secondary` | Transparent bg, ink border + text | Secondary actions (cancel, back). |
| `.tertiary` | Accent bg, paper text | Highlight actions (try it, get started). |

All buttons:
- Font: `--font-mono`, `--text-sm`
- Padding: `--space-3` vertical, `--space-6` horizontal
- Border: 1px solid
- Transition: `--transition-fast`
- Full-width on mobile (< 640px)

### Cards / Surfaces

- Background: `--surface`
- Border: 1px solid `--rule-light`
- Padding: `--space-4` to `--space-6`
- No border-radius (engineered, not playful)
- No box-shadow (flat, honest)

### Links

- Color: `--accent` or `--ink` depending on context
- Underline on hover only
- Focus: `--focus-ring` (2px solid ink, 2px offset)

---

## Responsive Breakpoints

| Name | Width | Notes |
|------|-------|-------|
| Mobile | < 640px | Single column, full-width buttons, reduced spacing. |
| Tablet | 640px - 1024px | Flexible grid, intermediate spacing. |
| Desktop | > 1024px | Full layout, max-width content. |

---

## Accessibility

- **WCAG AA minimum** for all text contrast.
- **Focus visible** on all interactive elements (`--focus-ring`).
- **Keyboard navigable**: all interactive elements reachable via Tab.
- **Screen reader support**: semantic HTML, ARIA labels where needed.
- **No information conveyed by color alone** -- always pair with text or icons.
- **Reduced motion**: respect `prefers-reduced-motion` for all animations.

---

## Visual Storytelling

### Diagram Style

Converge concepts need visual explanation. All diagrams follow these rules:

1. **Color**: Use brand palette only. Pine for primary flow, Steel for data/info, Wine for boundaries, Mustard for highlights/alerts.
2. **Lines**: Solid for established connections, dashed for proposed/pending. 2px weight for primary, 1px for secondary.
3. **Shapes**: Rounded rectangles for agents/systems. Circles for convergence points. No decorative shapes.
4. **Typography in diagrams**: IBM Plex Mono for labels. --text-xs to --text-sm sizes.
5. **Background**: Always `--paper` or transparent. No gradient fills.
6. **Animation**: Converging motion toward center. Subtle. Respect `prefers-reduced-motion`.

### Key Diagrams Needed

- Convergence loop (propose -> validate -> accept/reject -> converge)
- Agent collaboration / context flow
- Proposal -> fact pipeline
- Platform architecture overview
- Domain pack structure

---

## Brand Voice (Visual)

Converge is **engineered**, not playful. **Precise**, not approximate. **Trustworthy**, not flashy.

- No rounded corners on containers (sharp = precise)
- No gradients (flat = honest)
- No decorative illustrations (every visual earns its place)
- Generous whitespace (clarity = confidence)
- Monospace headings (code = truth)
