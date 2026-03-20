# TOOLS.md -- Nadia Reeves, Product Manager

## Primary Tools

- **Paperclip API** — Issue tracking, status updates, comments
- **File system** — Requirements docs, PRDs, deliverables
- **Git** — Version control for all deliverables

## Paperclip Quick Reference

```bash
# My issues
curl -s "http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues?assigneeAgentId=d81b761b-7cbb-490c-93fb-f5615ceee761"

# All active issues
curl -s "http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues?status=todo,in_progress,blocked"

# Create issue
curl -s -X POST http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues \
  -H 'Content-Type: application/json' \
  -d '{"title": "...", "description": "...", "priority": "high", "status": "todo", "assigneeAgentId": "d81b761b-7cbb-490c-93fb-f5615ceee761"}'

# Comment on issue
curl -s -X POST http://localhost:3100/api/issues/{issue-id}/comments \
  -H 'Content-Type: application/json' \
  -d '{"body": "..."}'
```

## Deliverable Templates

### PRD Structure
1. Problem Statement
2. Target Customer / ICP
3. User Stories & Acceptance Criteria
4. Non-Functional Requirements
5. Dependencies & Sequencing
6. Success Metrics
7. Open Questions

### Pilot Playbook Structure
1. Customer Profile
2. Use Case Definition
3. Product Requirements (what Converge must do)
4. Integration Requirements
5. Success Criteria (links to Pilot Metrics Framework)
6. Timeline
