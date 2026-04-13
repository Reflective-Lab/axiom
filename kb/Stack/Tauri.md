---
tags: [stack]
source: mixed
---
# Tauri

Desktop applications built on Converge use Tauri for native packaging and local-first execution.

## Architecture

```
┌─────────────────────────────────┐
│     Svelte frontend (webview)   │  UI layer
├─────────────────────────────────┤
│     Tauri command layer         │  Bridge: JS ↔ Rust
├─────────────────────────────────┤
│     Rust application layer      │  Business logic, Converge engine
└─────────────────────────────────┘
```

The Svelte frontend calls local Rust commands through Tauri — not HTTP/REST/gRPC. The Tauri command layer is the boundary between UI and engine.

## Key Points

- Tauri apps are local-first — the Converge engine runs in-process
- No server required for basic operation
- External access (LLM calls, API access) goes through `converge-provider`
- [[Building/Streaming|StreamingCallback]] pushes engine updates to the frontend in real time

## Desktop Dependencies

```toml
[dependencies]
converge-kernel = "3.0.1"
converge-provider = { version = "3.0.1", features = ["kong"] }
converge-tool = "3.0.1"
```

## Commands

```bash
just install-desktop   # Install frontend dependencies (bun)
just dev-desktop       # Run desktop app in dev mode
just package-desktop   # Build native desktop bundle
```

See also: [[Stack/Svelte]], [[Stack/Rust]]
