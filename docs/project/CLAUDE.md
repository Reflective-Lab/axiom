# Converge Project — Claude Code Configuration

## Paperclip Control Plane

This project is managed by a Paperclip instance running locally.

```
PAPERCLIP_API_URL=http://localhost:3100
PAPERCLIP_COMPANY_ID=ad34cffc-6c98-47c9-b629-2aed6f694149
```

The server runs in `local_trusted` mode — no API key is required for board-level operations.
Use the `/paperclip` skill for all coordination tasks (creating issues, managing agents, updating status, etc.).

### Agent Roster

| Name | Title | ID | Reports To |
|------|-------|----|------------|
| Morgan Vale | CEO | `25bff990-da10-4541-afb2-f5701b676564` | Kenneth (Board) |
| Nadia Reeves | Product Manager | `d81b761b-7cbb-490c-93fb-f5615ceee761` | Morgan Vale |
| Priya Chandran | Finance & Operations | `9876bdb0-e398-4478-85f5-65ab4d12824b` | Morgan Vale |
| Ren Akiyama | VP of Engineering | `689ba070-8e6a-451e-bb5d-cc0aea414dba` | Morgan Vale |
| Blake Harmon | VP of Marketing & Sales | `44d608a5-2d0b-422c-b7f0-c201e874c12a` | Morgan Vale |
| Eli Marsh | Founding Engineer | `a32158d4-396f-400c-8b25-60b9c959652d` | Ren Akiyama |
| Kira Novak | Senior Rust Developer | `2a46e45a-03f5-4a70-80ec-78d41cd5f8e5` | Ren Akiyama |
| Jules Carrera | Frontend Developer | `25214502-dbca-4f89-8ba7-1ab486707eca` | Ren Akiyama |
| Sam Okafor | QA Engineer | `ca1c54c6-7372-4b5f-a346-1282e9c38089` | Ren Akiyama |
| Dex Tanaka | DevOps Release Engineer | `825c119c-c656-4dd8-89b8-9d4be5dce5d5` | Ren Akiyama |
| Ava Petrov | Security Engineer | `9739f2d3-04ce-4043-b46a-ff8a75dc0e84` | Ren Akiyama |
| Leo Marin | Solutions Engineer | `43926e85-1ab4-40d9-9e37-707e7ff4e249` | Ren Akiyama |
| Rio Castellan | Designer | `0254c0f8-2280-467f-a40d-cf2d85f61494` | Blake Harmon |
| Caroline Ashford | Editor-in-Chief (external) | `368313a2-a6c2-4ccd-8261-a62717a52908` | Blake Harmon |
| Alice Mercer | Systems Pragmatist (external) | `4ae2c6ca-638a-4896-b269-c7d880f76b72` | Caroline Ashford |
| Bob Calder | Builder-Experimentalist (external) | `afa5f8f5-006e-400c-81a8-263153897c70` | Caroline Ashford |

### Strategic Structure

**Company Goal:** Build Converge — the semantic governance platform

**Engineering (5 waves, 15 projects, 10 tasks):**

| Wave | Projects | Status |
|------|----------|--------|
| Wave 1: Foundation | converge-core, converge-traits, converge-business | Active |
| Wave 2: Instantiation | converge-provider, converge-llm, converge-analytics, converge-policy, converge-optimization | Planned |
| Wave 3: Tooling | converge-tool, converge-domain, converge-experience | Backlog |
| Wave 4: Infrastructure | converge-runtime | Backlog |
| Wave 5: Experience | converge-remote, converge-application, converge-personas | Backlog |

**Go-to-Market (5 projects, 9 tasks) — "Land 3-4 paying design partners":**

| Priority | Project | Key Tasks |
|----------|---------|-----------|
| P0 | Pricing & Packaging | Draft 2-3 tiers, publish pricing page |
| P0 | Pilot Program & Metrics | Instrument pilots from day 1, build conversion funnel |
| P0 | converge.zone — Business Buyer | Business-buyer landing page, outcome-led messaging |
| P1 | Security & Compliance | Security one-pager, SOC 2 Type I kickoff |
| P1-P2 | Support & Onboarding Model | Instrument support touchpoints, define cost model |

### Quick API Examples

```bash
# List all agents
curl -s http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/agents | python3 -m json.tool

# List all issues (with filters)
curl -s "http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues?status=todo,in_progress,blocked" | python3 -m json.tool

# List all projects
curl -s http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/projects | python3 -m json.tool

# List all goals
curl -s http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/goals | python3 -m json.tool

# Create an issue
curl -s -X POST http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues \
  -H 'Content-Type: application/json' \
  -d '{"title": "Task title", "description": "Details", "priority": "high", "status": "todo", "assigneeAgentId": "<agent-id>", "projectId": "<project-id>", "goalId": "<goal-id>"}'

# Update an issue
curl -s -X PATCH http://localhost:3100/api/issues/<issue-id> \
  -H 'Content-Type: application/json' \
  -d '{"status": "done", "comment": "Completed."}'

# Create a project
curl -s -X POST http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/projects \
  -H 'Content-Type: application/json' \
  -d '{"name": "Project Name", "description": "Description", "status": "planned"}'

# Create a goal
curl -s -X POST http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/goals \
  -H 'Content-Type: application/json' \
  -d '{"title": "Goal Title", "description": "Description", "level": "team", "status": "planned", "parentId": "<parent-goal-id>"}'

# Add a comment to an issue
curl -s -X POST http://localhost:3100/api/issues/<issue-id>/comments \
  -H 'Content-Type: application/json' \
  -d '{"body": "Comment text here."}'
```

### Enums Reference

- **Project status:** backlog, planned, in_progress, completed, cancelled
- **Goal level:** company, team, agent, task
- **Goal status:** planned, active, achieved, cancelled
- **Issue status:** backlog, todo, in_progress, in_review, done, blocked, cancelled
- **Issue priority:** critical, high, medium, low

### Agent Definitions

Each agent's instructions are in `agents/<role>/AGENTS.md`. When adding a new agent:
1. Create `agents/<role>/AGENTS.md` with their instructions
2. Register via the API: `POST /api/companies/{companyId}/agents`
3. Or extend the seed script at `/Users/kpernyer/tool/paperclip/packages/db/src/seed-converge-agents.ts`

### Seed Scripts (in paperclip repo)

- `packages/db/src/seed-converge-agents.ts` — Registers all 10 agents
- `packages/db/src/seed-converge-full.ts` — Creates goals, projects, and issues for the full strategic structure

## Project Structure

- `agents/` — Agent instruction files (AGENTS.md, SOUL.md, TOOLS.md per agent)
- `plans/` — Strategic plans (CRATE_ALIGNMENT.md)
- `tasks/` — Task specifications (001-010)
- `milestones/` — Project milestones
- `scripts/` — Automation scripts (hire-agents.sh)
- `templates/` — Justfile template
