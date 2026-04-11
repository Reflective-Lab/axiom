---
tags: [concepts]
---
# Experience and Recall

Converge tracks experience events during runs and lets agents query past experience to inform current decisions.

## Tracking Experience

```rust
let observer = Arc::new(|event: &ExperienceEvent| {
    log::info!("experience: {:?}", event);
});

TypesRunHooks {
    criterion_evaluator: Some(Arc::new(evaluator)),
    event_observer: Some(observer),
}
```

## Recall System

The `converge_core::recall` module lets agents query past decisions:

| Type | Purpose |
|---|---|
| `RecallQuery` | Searches for relevant past decisions |
| `RecallCandidate` | Scores results by relevance |
| `RecallPolicy` | Controls what can be recalled and by whom |

An agent can recall what similar situations looked like in past runs and use that as a baseline — while still going through the governance gate for its own proposals.

See also: [[Concepts/Governed Artifacts]], [[Concepts/Context and Facts]]
