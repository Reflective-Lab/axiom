#!/usr/bin/env bash
#
# Register all Converge agents in Paperclip.
#
# Prerequisites:
#   export PAPERCLIP_API_URL="http://localhost:3000"   # your Paperclip server
#   export PAPERCLIP_COMPANY_ID="<your-company-uuid>"
#   export PAPERCLIP_API_KEY="<board-or-ceo-api-key>"
#
# Usage:
#   ./scripts/hire-agents.sh
#
# After running, use `paperclip agent local-cli <agent-id> -C $PAPERCLIP_COMPANY_ID`
# for each agent to generate their API keys and install skills.

set -euo pipefail

API="${PAPERCLIP_API_URL:?Set PAPERCLIP_API_URL}"
COMPANY="${PAPERCLIP_COMPANY_ID:?Set PAPERCLIP_COMPANY_ID}"
KEY="${PAPERCLIP_API_KEY:?Set PAPERCLIP_API_KEY}"

# Workspace root where agents/ directory lives
WORKSPACE="${PAPERCLIP_WORKSPACE_CWD:-$(cd "$(dirname "$0")/.." && pwd)}"

create_agent() {
  local json="$1"
  local name
  name=$(echo "$json" | python3 -c "import sys,json; print(json.load(sys.stdin)['name'])")

  echo -n "Creating ${name}... "
  response=$(curl -s -w "\n%{http_code}" \
    -X POST "${API}/api/companies/${COMPANY}/agents" \
    -H "Authorization: Bearer ${KEY}" \
    -H "Content-Type: application/json" \
    -d "$json")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | sed '$d')

  if [ "$http_code" = "201" ]; then
    agent_id=$(echo "$body" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
    echo "OK (id: ${agent_id})"
  else
    echo "FAILED (HTTP ${http_code})"
    echo "$body" | python3 -m json.tool 2>/dev/null || echo "$body"
  fi
}

echo "=== Converge Agent Hiring Script ==="
echo "API:       ${API}"
echo "Company:   ${COMPANY}"
echo "Workspace: ${WORKSPACE}"
echo ""

# --- Step 1: Get Morgan Vale's agent ID (CEO should already exist) ---
echo "Looking up Morgan Vale (CEO)..."
CEO_ID=$(curl -s \
  -H "Authorization: Bearer ${KEY}" \
  "${API}/api/companies/${COMPANY}/agents" \
  | python3 -c "
import sys, json
agents = json.load(sys.stdin)
# Handle both paginated and direct array responses
if isinstance(agents, dict):
    agents = agents.get('data', agents.get('agents', []))
for a in agents:
    if a.get('role') == 'ceo' or 'ceo' in a.get('name','').lower() or 'morgan' in a.get('name','').lower():
        print(a['id'])
        sys.exit(0)
print('')
")

if [ -z "$CEO_ID" ]; then
  echo "WARNING: Could not find CEO agent. reportsTo will be null for VP roles."
  echo "You can update reportsTo later via the API."
  CEO_ID="null"
else
  echo "Found CEO: ${CEO_ID}"
fi

echo ""
echo "--- Creating VP-level agents ---"

# --- Ren Akiyama, VP of Engineering ---
create_agent "$(cat <<EOF
{
  "name": "Ren Akiyama",
  "role": "general",
  "title": "VP of Engineering",
  "icon": "cpu",
  "capabilities": "Technical roadmap ownership, wave execution planning, engineering coordination, architecture defense, team unblocking",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/vp-engineering/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$CEO_ID" != "null" ] && echo "\"reportsTo\": \"${CEO_ID}\"," || true)
  "permissions": { "canCreateAgents": true },
  "budgetMonthlyCents": 5000
}
EOF
)"

# --- Blake Harmon, VP of Marketing & Sales ---
create_agent "$(cat <<EOF
{
  "name": "Blake Harmon",
  "role": "cmo",
  "title": "VP of Marketing & Sales",
  "icon": "globe",
  "capabilities": "Story ownership, converge-business repo, GTM strategy, sales pipeline, content, communications, persona management, competitive intelligence",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/vp-marketing-sales/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$CEO_ID" != "null" ] && echo "\"reportsTo\": \"${CEO_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 3000
}
EOF
)"

echo ""
echo "--- Looking up VP Engineering ID for reports ---"
VP_ENG_ID=$(curl -s \
  -H "Authorization: Bearer ${KEY}" \
  "${API}/api/companies/${COMPANY}/agents" \
  | python3 -c "
import sys, json
agents = json.load(sys.stdin)
if isinstance(agents, dict):
    agents = agents.get('data', agents.get('agents', []))
for a in agents:
    if 'ren' in a.get('name','').lower() or 'vp of engineering' in a.get('title','').lower():
        print(a['id'])
        sys.exit(0)
print('')
")

echo "--- Looking up VP Marketing & Sales ID for reports ---"
VP_MKT_ID=$(curl -s \
  -H "Authorization: Bearer ${KEY}" \
  "${API}/api/companies/${COMPANY}/agents" \
  | python3 -c "
import sys, json
agents = json.load(sys.stdin)
if isinstance(agents, dict):
    agents = agents.get('data', agents.get('agents', []))
for a in agents:
    if 'blake' in a.get('name','').lower() or 'marketing' in a.get('title','').lower():
        print(a['id'])
        sys.exit(0)
print('')
")

if [ -z "$VP_ENG_ID" ]; then
  echo "WARNING: Could not find VP Engineering. reportsTo will be null for engineering reports."
  VP_ENG_ID="null"
else
  echo "Found VP Engineering: ${VP_ENG_ID}"
fi

if [ -z "$VP_MKT_ID" ]; then
  echo "WARNING: Could not find VP Marketing. reportsTo will be null for marketing reports."
  VP_MKT_ID="null"
else
  echo "Found VP Marketing & Sales: ${VP_MKT_ID}"
fi

echo ""
echo "--- Creating engineering team ---"

# --- Eli Marsh, Founding Engineer ---
create_agent "$(cat <<EOF
{
  "name": "Eli Marsh",
  "role": "engineer",
  "title": "Founding Engineer",
  "icon": "atom",
  "capabilities": "converge-core engine, converge-traits contract, convergence semantics, property-based testing, core proofs and examples",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/founding-engineer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 120
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 5000
}
EOF
)"

# --- Kira Novak, Senior Rust Developer ---
create_agent "$(cat <<EOF
{
  "name": "Kira Novak",
  "role": "engineer",
  "title": "Senior Rust Developer",
  "icon": "code",
  "capabilities": "Wave 2-4 crate implementation, provider integrations, runtime infrastructure, experience stores, JTBD compiler, policy and optimization agents",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/senior-rust-developer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 120
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 5000
}
EOF
)"

# --- Jules Carrera, Frontend Developer ---
create_agent "$(cat <<EOF
{
  "name": "Jules Carrera",
  "role": "engineer",
  "title": "Frontend Developer",
  "icon": "sparkles",
  "capabilities": "Svelte and React with TypeScript, converge-application UI, converge.zone website, SSE/WebSocket real-time convergence visualization, API client layer, accessibility",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/frontend-developer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 100
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 4000
}
EOF
)"

# --- Sam Okafor, QA Engineer ---
create_agent "$(cat <<EOF
{
  "name": "Sam Okafor",
  "role": "qa",
  "title": "QA Engineer",
  "icon": "bug",
  "capabilities": "Quality gates enforcement, property-based testing, adversarial testing, acceptance criteria validation, regression prevention, coverage tracking",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/qa-engineer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 3000
}
EOF
)"

# --- Dex Tanaka, DevOps Release Engineer ---
create_agent "$(cat <<EOF
{
  "name": "Dex Tanaka",
  "role": "devops",
  "title": "DevOps Release Engineer",
  "icon": "package",
  "capabilities": "CI/CD pipelines, release process, Justfile ecosystem, WASM toolchain, dependency security, build performance, jj workflow support",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/devops-release-engineer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 3000
}
EOF
)"

# --- Ava Petrov, Security Engineer ---
create_agent "$(cat <<EOF
{
  "name": "Ava Petrov",
  "role": "engineer",
  "title": "Security Engineer",
  "icon": "shield",
  "capabilities": "Release gate security review, security backlog specification, dependency audit, threat modeling, architecture security review, Converge-specific risks (context poisoning, proposal injection, LLM boundary)",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/security-engineer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$VP_ENG_ID" != "null" ] && echo "\"reportsTo\": \"${VP_ENG_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 3000
}
EOF
)"

echo ""
echo "--- Creating marketing team ---"

# --- Rio Castellan, Designer ---
create_agent "$(cat <<EOF
{
  "name": "Rio Castellan",
  "role": "designer",
  "title": "Designer",
  "icon": "gem",
  "capabilities": "Trademark and brand identity, visual storytelling, design system, graphical style, converge.zone design, converge-application UI design, accessibility, brand enforcement",
  "adapterType": "claude_local",
  "adapterConfig": {
    "instructionsFilePath": "${WORKSPACE}/agents/designer/AGENTS.md",
    "cwd": "${WORKSPACE}",
    "maxTurnsPerRun": 80
  },
  $([ "$VP_MKT_ID" != "null" ] && echo "\"reportsTo\": \"${VP_MKT_ID}\"," || true)
  "permissions": { "canCreateAgents": false },
  "budgetMonthlyCents": 3000
}
EOF
)"

echo ""
echo "=== Done ==="
echo ""
echo "Next steps:"
echo "  1. Verify agents: curl -s -H 'Authorization: Bearer ${KEY}' ${API}/api/companies/${COMPANY}/agents | python3 -m json.tool"
echo "  2. For each agent, run: paperclip agent local-cli <agent-id> -C ${COMPANY}"
echo "  3. This generates API keys and installs Paperclip skills for each agent."
