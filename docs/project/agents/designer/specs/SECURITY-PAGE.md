# Design Spec: Security Page

**Route:** `/security`
**Owner:** Rio Castellan (design), Ava Petrov (content), Jules Carrera (build)
**Priority:** P1
**Content source:** `agents/security-engineer/deliverables/security-one-pager-content.md`

---

## Purpose

Enterprise buyers and design partners need to see that Converge takes security seriously before they'll share CRM data, workflow details, or commit to a pilot. This page makes the security posture visible and credible. It's not a compliance document — it's a trust signal.

---

## Visual Strategy

The security page should feel **authoritative and calm**. No alarmist language, no lock icons scattered everywhere. The architecture speaks for itself. The visual treatment should communicate: "we thought about this before you asked."

- Pine (#2d5a3d) for positive security properties (trust boundaries, encryption, access controls)
- Monospace typography for technical details (creates engineering credibility)
- Clean section breaks with generous whitespace
- No decorative elements — every visual element earns its place

---

## Page Structure

### Section 1: Hero

**Layout:** Left-aligned (consistent with all business pages)
**Tagline:** `Security` (mono, uppercase, accent)
**Headline:** `Security is architecture, not a feature`
**Subhead:** "Converge enforces trust boundaries at every layer. LLM outputs are never trusted. Agent identity is verified. Every decision is traceable."

```css
.header {
  text-align: left;
  padding: var(--space-12) 0;
  border-bottom: 1px solid var(--rule-light);
}
```

---

### Section 2: Architecture Security

**Section title:** `Architecture Security`
**Layout:** Left-aligned prose with three subsection cards

Three trust boundary cards, stacked vertically with `var(--space-6)` gap:

**Card 1: LLM Trust Boundary**
- Icon indicator: `//` (mono, pine accent)
- Title: "LLM outputs are untrusted"
- Body: ProposedFact → validation → Fact explanation
- Visual cue: Left border `3px solid var(--accent)` instead of full border

**Card 2: Agent Identity**
- Title: "Agent identity is verified"
- Body: Verified identity, tagged contributions, no forgery

**Card 3: Policy Enforcement**
- Title: "Policy is a first-class agent"
- Body: Cedar policies, convergence loop integration

**Card CSS:**
```css
.trustCard {
  padding: var(--space-5) var(--space-6);
  border-left: 3px solid var(--accent);
  background: var(--paper);
  margin-bottom: var(--space-4);
}

.trustCardTitle {
  font-family: var(--font-mono);
  font-size: var(--text-md);
  font-weight: 600;
  margin-bottom: var(--space-2);
}

.trustCardBody {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  color: var(--ink-secondary);
  line-height: 1.7;
}
```

---

### Section 3: WASM Sandboxing

**Section title:** `WASM Sandboxing`
**Layout:** Left-aligned prose + 4-item checklist

Checklist items styled as a simple list with pine checkmarks:

```css
.checkItem {
  display: flex;
  align-items: baseline;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
  font-family: var(--font-sans);
  font-size: var(--text-base);
  color: var(--ink-secondary);
}

.checkMark {
  font-family: var(--font-mono);
  color: var(--accent);
  flex-shrink: 0;
}
```

Items: Isolated modules, Signature verification, Resource limits, No filesystem/network access.

---

### Section 4: Data Handling

**Section title:** `Data Handling`
**Layout:** Three subsections (Data in Transit, Data at Rest, Data Classification)

Each subsection: `<h3>` title + bullet list. Simple prose, no cards.

**Data Classification** rendered as a definition list:
```css
.defList dt {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--ink);
  margin-bottom: var(--space-1);
}

.defList dd {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  color: var(--ink-secondary);
  margin-bottom: var(--space-4);
  margin-left: 0;
}
```

---

### Section 5: Access Controls

**Section title:** `Access Controls`
**Layout:** Three subsections (Authentication, Authorization, Audit Trail)

Same pattern as Data Handling — subsection titles + bullet lists. Keep it scannable.

---

### Section 6: Incident Response

**Section title:** `Incident Response`
**Layout:** Two parts

**Part 1: Monitoring** — bullet list
**Part 2: Response Process** — ordered list with 5 steps, each as a compact card:

```css
.step {
  display: flex;
  align-items: baseline;
  gap: var(--space-3);
  padding: var(--space-3) 0;
  border-bottom: 1px solid var(--rule-light);
}

.stepNumber {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--accent);
  flex-shrink: 0;
  width: 24px;
}

.stepTitle {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 600;
}

.stepDesc {
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  color: var(--ink-secondary);
  margin-left: var(--space-1);
}
```

Steps: 1. Detection, 2. Triage (1 hour), 3. Containment, 4. Remediation, 5. Disclosure (72 hours)

---

### Section 7: Supply Chain Security

**Section title:** `Supply Chain Security`
**Layout:** Simple bullet list, same styling as Data Handling

---

### Section 8: Compliance Roadmap

**Section title:** `Compliance Roadmap`
**Layout:** Simple table

| Milestone | Target | Status |
|-----------|--------|--------|

Status rendered as a mono badge:
- "In progress" → pine text
- "Planned" → ink-muted text

```css
.statusBadge {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.statusActive {
  color: var(--accent);
}

.statusPlanned {
  color: var(--ink-muted);
}
```

---

### Section 9: Contact

**Section title:** `Questions?`
**Layout:** Short prose + email link

Email: `security@converge.zone` styled as a mono link in accent color.

---

## Responsive Behavior

| Breakpoint | Change |
|------------|--------|
| > 768px | Full layout, generous whitespace |
| < 768px | All content single-column, trust cards full-width |
| < 640px | Title drops to `var(--text-2xl)`, section padding reduces to `var(--space-8)` |

---

## Navigation

- Add `/security` to the footer links (under "Company" or similar group)
- Do NOT add to header nav — this is a reference page, not a primary path
- Route registered in `main.tsx` with lazy loading

---

## Accessibility

- All section headings follow correct `<h1>` → `<h2>` → `<h3>` hierarchy
- Checklist items use semantic `<ul>` with `role="list"`
- Table uses `<thead>` and `<th>` with `scope="col"`
- Email link has descriptive text
- All text meets WCAG AA contrast ratios

---

## Implementation Notes for Jules

- **Component:** `src/app/pages/Security.tsx` + `Security.module.css`
- Content is long — consider extracting section data as const arrays (like About.tsx)
- Left-border trust cards are new — could be reusable for other feature highlight patterns
- Lazy-load in `main.tsx`
- SEO: `<title>Security - Converge</title>`, `<meta name="description" content="How Converge secures AI agent workflows. Trust boundaries, WASM sandboxing, encryption, and compliance roadmap.">`
