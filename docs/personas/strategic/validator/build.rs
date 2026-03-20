use serde_json::json;
use std::fs;
use std::path::Path;

fn main() {
    // Configure cargo rebuild triggers
    println!("cargo:rerun-if-changed=../../TEAM.md");
    println!("cargo:rerun-if-changed=../../GATES.md");
    println!("cargo:rerun-if-changed=build.rs");

    // Read TEAM.md and GATES.md
    let team_md = fs::read_to_string("../../TEAM.md")
        .expect("Failed to read TEAM.md");
    let gates_md = fs::read_to_string("../../GATES.md")
        .expect("Failed to read GATES.md");

    // Parse personas from TEAM.md
    let personas = parse_team_md(&team_md);

    // Parse gates from GATES.md
    let gates = parse_gates_md(&gates_md);

    // Generate Cedar entities
    let mut entities = Vec::new();

    // Add Team entities (Core and Extended)
    entities.push(json!({
        "uid": {"type": "ConvergePersonas::Team", "id": "Core"},
        "attrs": {"team_type": "Core"},
        "parents": []
    }));
    entities.push(json!({
        "uid": {"type": "ConvergePersonas::Team", "id": "Extended"},
        "attrs": {"team_type": "Extended"},
        "parents": []
    }));

    // Add User entities with parent Team references
    for persona in personas {
        entities.push(json!({
            "uid": {"type": "ConvergePersonas::User", "id": persona.persona_id},
            "attrs": {
                "persona_id": persona.persona_id,
                "authority_tier": persona.authority_tier,
            },
            "parents": [
                {"type": "ConvergePersonas::Team", "id": persona.team}
            ]
        }));
    }

    // Add Gate entities
    for gate in gates {
        // Convert persona IDs to User entity UIDs for elevated_approvers
        let elevated_approvers: Vec<serde_json::Value> = gate.elevated_blocking_evals
            .iter()
            .map(|persona_id| json!({"type": "ConvergePersonas::User", "id": persona_id}))
            .collect();

        entities.push(json!({
            "uid": {"type": "ConvergePersonas::Gate", "id": gate.gate_id},
            "attrs": {
                "gate_id": gate.gate_id,
                "risk_class": gate.risk_class,
                "elevated_approvers": elevated_approvers,
            },
            "parents": []
        }));
    }

    // Write entities.json
    let entities_json = serde_json::to_string_pretty(&entities)
        .expect("Failed to serialize entities");

    let output_path = Path::new("cedar/entities.json");
    fs::create_dir_all(output_path.parent().unwrap())
        .expect("Failed to create cedar directory");
    fs::write(output_path, entities_json)
        .expect("Failed to write entities.json");

    println!("Generated {} Cedar entities", entities.len());
}

#[derive(Debug)]
struct Persona {
    persona_id: String,
    team: String,
    authority_tier: String,
}

#[derive(Debug)]
struct Gate {
    gate_id: String,
    risk_class: String,
    elevated_blocking_evals: Vec<String>,  // Persona IDs with -eval suffix stripped
}

fn parse_team_md(content: &str) -> Vec<Persona> {
    let mut personas = Vec::new();
    let mut in_roster_table = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect table start
        if trimmed.starts_with("| persona_id |") {
            in_roster_table = true;
            continue;
        }

        // Skip separator row
        if trimmed.starts_with("|---") {
            continue;
        }

        // Parse table rows
        if in_roster_table && trimmed.starts_with('|') {
            let parts: Vec<&str> = trimmed
                .split('|')
                .map(|s| s.trim())
                .collect();

            // Valid row has at least 6 columns (empty, persona_id, persona_name, team, authority_tier, tier_rationale, empty)
            if parts.len() >= 6 && !parts[1].is_empty() && parts[1] != "persona_id" {
                personas.push(Persona {
                    persona_id: parts[1].to_string(),
                    team: parts[3].to_string(),
                    authority_tier: parts[4].to_string(),
                });
            }
        }

        // Stop when we reach end of table (empty line or non-table line)
        if in_roster_table && !trimmed.starts_with('|') && !trimmed.is_empty() {
            break;
        }
    }

    if personas.is_empty() {
        panic!("No personas found in TEAM.md");
    }

    personas
}

fn parse_gates_md(content: &str) -> Vec<Gate> {
    let mut gates = Vec::new();
    let mut in_policy_table = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect Policy Table (has 12 columns including elevated_blocking_evals)
        // Distinguish from Gate Summary by checking for core_required column
        if trimmed.contains("| gate_id |") && trimmed.contains("| core_required |") {
            in_policy_table = true;
            continue;
        }

        // Skip separator row
        if trimmed.starts_with("|---") {
            continue;
        }

        // Parse table rows
        if in_policy_table && trimmed.starts_with('|') {
            let parts: Vec<&str> = trimmed
                .split('|')
                .map(|s| s.trim())
                .collect();

            // Valid row has 13 parts (empty at start/end + 11 data columns)
            // Columns: gate_id, gate_name, type, promotion_target, risk_class,
            //          core_required, required_eval_packs, elevated_blocking_evals,
            //          escalation_allowed, override_policy, evidence_required, stop_rule
            if parts.len() >= 13 && !parts[1].is_empty() && parts[1] != "gate_id" {
                // Column 8 (index 8 in parts) is elevated_blocking_evals
                let elevated_raw = parts[8];
                let elevated_blocking_evals: Vec<String> = if elevated_raw == "—" || elevated_raw.is_empty() {
                    Vec::new()
                } else {
                    elevated_raw
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            // Strip -eval suffix: "sre-operations-eval" -> "sre-operations"
                            s.strip_suffix("-eval").unwrap_or(s).to_string()
                        })
                        .collect()
                };

                gates.push(Gate {
                    gate_id: parts[1].to_string(),
                    risk_class: parts[5].to_string(),
                    elevated_blocking_evals,
                });
            }
        }

        // Stop when we reach end of table
        if in_policy_table && !trimmed.starts_with('|') && !trimmed.is_empty() {
            break;
        }
    }

    if gates.is_empty() {
        panic!("No gates found in GATES.md Policy Table");
    }

    gates
}
