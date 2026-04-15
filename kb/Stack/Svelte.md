---
tags: [stack]
source: mixed
---
# Svelte

The UI layer for Converge desktop applications. Runs inside a Tauri webview.

## Role

Svelte handles presentation only:
- Rendering convergence progress via [[Building/Streaming|streaming callbacks]]
- Displaying facts, proposals, and decisions
- Presenting [[Concepts/HITL Gates|human-in-the-loop]] review panels
- File picking for local input sources (Gherkin, truth-spec JSON)

Svelte does not run agents, evaluate criteria, or make decisions. That's the Rust layer's job.

## Stack

- SvelteKit for the application framework
- Bun for package management
- Vite for bundling

## Local Input Model

Desktop apps accept local input formats:
- Gherkin `.feature` files — parsed by `converge-axiom`
- Truth-spec `.truths.json` files — structured job definitions

Both are normalized in the Rust layer before execution.

See also: [[Stack/Tauri]], [[Stack/Rust]]
