---
tags: [building]
source: llm
---

# CLI Reference — cz

`cz` is Axiom's workspace orchestrator CLI.

## Diagnostics

| Command | Purpose |
|---|---|
| `cz doctor` | Check environment: rustc, cargo, docker, git, toolchain, hooks, .env |
| `cz bootstrap` | Install deps, create .env, validate binaries |

## Governance

| Command | Purpose |
|---|---|
| `cz validate [paths]` | Validate `.truths` files |
| `cz validate --conventions-only` | Skip LLM checks |
| `cz validate --skip-business-sense` | Skip business sense check |
| `cz validate --skip-compilability` | Skip compilability check |
| `cz validate --enforce` | Include Cedar policy coverage |
| `cz digest` | Summarize open findings, pending acks, escalations |
| `cz ack <finding>` | Acknowledge a finding (creates audit artifact) |
| `cz escalate <finding>` | Escalate to target team |
| `cz assign <finding> <owner>` | Assign finding to owner |

## Development

| Command | Purpose |
|---|---|
| `cz test` | Run all tests |
| `cz fmt` | Format code |
| `cz lint` | Clippy + format check |
| `cz ci` | Full CI pipeline locally |

## Services

| Command | Purpose |
|---|---|
| `cz up` | Start Docker Compose services |
| `cz down` | Stop services |
| `cz logs` | View service logs |

## Findings Storage

All governance artifacts are stored in `.converge/`:

```
.converge/
├── findings/      JSON finding records
├── acks/          Acknowledgement artifacts
├── escalations/   Escalation records
└── assignments/   Assignment records
```
