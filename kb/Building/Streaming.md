---
tags: [building]
---
# Streaming Callbacks

Implement `StreamingCallback` to watch convergence in real time.

```rust
struct UiCallback;

impl StreamingCallback for UiCallback {
    fn on_cycle_start(&self, cycle: u32) {
        println!("--- Cycle {cycle} ---");
    }

    fn on_fact(&self, cycle: u32, fact: &Fact) {
        println!("  [cycle {cycle}] new fact: {}", fact.id);
    }

    fn on_cycle_end(&self, cycle: u32, facts_added: usize) {
        println!("  cycle {cycle} complete: {facts_added} facts added");
    }
}
```

The callback fires as the engine runs, not after. Wire it into a Tauri command layer to push updates to a Svelte frontend, or into any other UI framework.

See also: [[Philosophy/Convergence Explained]], [[Concepts/Agents]]
