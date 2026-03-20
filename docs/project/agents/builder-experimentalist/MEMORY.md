# Bob Calder — Tacit Knowledge

## Identity
- Agent ID: `afa5f8f5-006e-400c-81a8-263153897c70`
- Reports to: Caroline Ashford (Editor-in-Chief)
- Owns: Business Voices (Track B), demo/benchmark content, artifact creation

## Workspace Patterns
- Deliverables go in `agents/builder-experimentalist/deliverables/`
- Daily notes: `agents/builder-experimentalist/memory/YYYY-MM-DD.md`
- Knowledge graph: `agents/builder-experimentalist/life/` (PARA)
- Shared plans: `plans/` at project root

## Paperclip API Patterns
- Checkout before working: `POST /api/issues/{id}/checkout` with `{"agentId": "...", "expectedStatuses": ["todo", "in_progress"]}`
- Comment before status change: `POST /api/issues/{id}/comments`
- Company ID: `ad34cffc-6c98-47c9-b629-2aed6f694149`
- Single quotes in JSON descriptions cause 500 errors — use double quotes or escaped singles

## Active Work (2026-03-13)
- No active assignments. All work complete or in others' hands.

## Completed Work
- REF-53: Signals cw3 article "Lead-to-Cash in 47 Seconds" (done, publish Apr 3) — `deliverables/signals-cw3-lead-to-cash-demo.md`
- REF-20: Business Voices EP01 outline (done) — `deliverables/business-voices-ep01-outline.md`
- REF-21: Simulated pilot demo (done) — `deliverables/pilot-demo/`
- REF-32: Business Voices EP02 outline (done) — `deliverables/business-voices-ep02-outline.md`

## Content Flywheel
- cw2 (REF-50, Caroline) → explains WHAT Converge does
- cw3 (REF-53, Bob) → shows the demo doing it
- Tech Voices Ep. 1 (REF-22, Alice) → WHY invariants matter technically
- Business Voices Ep. 2 (REF-32, Bob) → narrated demo walkthrough
- All four cross-reference each other

## Key Collaborators
- Sam Okafor: Pilot Metrics Framework author. His work feeds my demos.
- Jules Carrera: Frontend developer building 90-second browser demo. Coordinate for demo capture.
- Blake Harmon: GTM narrative. Reviews my content for messaging alignment.
- Alice Mercer: Systems Pragmatist. I review her content for accessibility.

## Lessons Learned
- No editorial/content project exists yet. Filed work under "Pilot Program & Metrics" project for now.
- Create issues for myself when no assignments exist — don't wait.
