// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Streaming output handler for remote CLI.
//!
//! Handles the display of streaming facts from converge-runtime.

use std::io::{self, Write};

/// Output format for streaming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable format
    Human,
    /// JSON Lines format
    Json,
}

/// Streaming output handler
pub struct StreamingHandler {
    format: OutputFormat,
    fact_count: usize,
}

impl StreamingHandler {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            fact_count: 0,
        }
    }

    pub fn human() -> Self {
        Self::new(OutputFormat::Human)
    }

    pub fn json() -> Self {
        Self::new(OutputFormat::Json)
    }

    pub fn fact_count(&self) -> usize {
        self.fact_count
    }

    /// Emit a context entry
    pub fn emit_entry(&mut self, sequence: i64, entry_type: &str, entry_id: &str, content: &str) {
        self.fact_count += 1;

        match self.format {
            OutputFormat::Human => {
                println!("[seq:{sequence}] {entry_type}:{entry_id} | {content}");
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "sequence": sequence,
                    "type": entry_type,
                    "entry_id": entry_id,
                    "content": content,
                });
                if let Ok(json) = serde_json::to_string(&output) {
                    println!("{json}");
                }
            }
        }

        let _ = io::stdout().flush();
    }

    /// Emit run status change
    pub fn emit_status(&self, run_id: &str, status: &str, facts: i32, cycles: i32) {
        match self.format {
            OutputFormat::Human => {
                println!("[{run_id}] {status} | {facts} facts, {cycles} cycles");
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "run_id": run_id,
                    "type": "status",
                    "status": status,
                    "facts": facts,
                    "cycles": cycles,
                });
                if let Ok(json) = serde_json::to_string(&output) {
                    println!("{json}");
                }
            }
        }

        let _ = io::stdout().flush();
    }

    /// Emit waiting state
    pub fn emit_waiting(&self, run_id: &str, waiting_for: &[String]) {
        match self.format {
            OutputFormat::Human => {
                let waiting_list = waiting_for.join(", ");
                println!("[{run_id}] waiting for: {waiting_list}");
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "run_id": run_id,
                    "type": "waiting",
                    "waiting_for": waiting_for,
                });
                if let Ok(json) = serde_json::to_string(&output) {
                    println!("{json}");
                }
            }
        }

        let _ = io::stdout().flush();
    }

    /// Emit halt explanation
    pub fn emit_halt(&self, run_id: &str, reason: &str, truth_id: &str) {
        match self.format {
            OutputFormat::Human => {
                println!("[{run_id}] HALTED: {reason} (truth: {truth_id})");
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "run_id": run_id,
                    "type": "halted",
                    "reason": reason,
                    "truth_id": truth_id,
                });
                if let Ok(json) = serde_json::to_string(&output) {
                    println!("{json}");
                }
            }
        }

        let _ = io::stdout().flush();
    }
}

/// Format a human-readable entry line
pub fn format_human_entry(
    sequence: i64,
    entry_type: &str,
    entry_id: &str,
    content: &str,
) -> String {
    format!("[seq:{sequence}] {entry_type}:{entry_id} | {content}")
}

/// Format a JSON entry line
pub fn format_json_entry(
    sequence: i64,
    entry_type: &str,
    entry_id: &str,
    content: &str,
) -> Option<String> {
    let output = serde_json::json!({
        "sequence": sequence,
        "type": entry_type,
        "entry_id": entry_id,
        "content": content,
    });
    serde_json::to_string(&output).ok()
}

/// Format a human-readable status line
pub fn format_human_status(run_id: &str, status: &str, facts: i32, cycles: i32) -> String {
    format!("[{run_id}] {status} | {facts} facts, {cycles} cycles")
}

/// Format a JSON status line
pub fn format_json_status(run_id: &str, status: &str, facts: i32, cycles: i32) -> Option<String> {
    let output = serde_json::json!({
        "run_id": run_id,
        "type": "status",
        "status": status,
        "facts": facts,
        "cycles": cycles,
    });
    serde_json::to_string(&output).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Unit Tests - StreamingHandler Construction
    // ==========================================================================

    #[test]
    fn handler_new_human_format() {
        let handler = StreamingHandler::new(OutputFormat::Human);
        assert_eq!(handler.format, OutputFormat::Human);
        assert_eq!(handler.fact_count(), 0);
    }

    #[test]
    fn handler_new_json_format() {
        let handler = StreamingHandler::new(OutputFormat::Json);
        assert_eq!(handler.format, OutputFormat::Json);
        assert_eq!(handler.fact_count(), 0);
    }

    #[test]
    fn handler_human_convenience() {
        let handler = StreamingHandler::human();
        assert_eq!(handler.format, OutputFormat::Human);
    }

    #[test]
    fn handler_json_convenience() {
        let handler = StreamingHandler::json();
        assert_eq!(handler.format, OutputFormat::Json);
    }

    // ==========================================================================
    // Unit Tests - Fact Counting
    // ==========================================================================

    #[test]
    fn streaming_handler_counts_entries() {
        let mut handler = StreamingHandler::human();
        assert_eq!(handler.fact_count(), 0);

        handler.emit_entry(1, "fact", "test-id", "test content");
        assert_eq!(handler.fact_count(), 1);
    }

    #[test]
    fn streaming_handler_counts_multiple_entries() {
        let mut handler = StreamingHandler::human();

        for i in 0..10 {
            handler.emit_entry(i, "fact", &format!("fact-{i}"), "content");
        }

        assert_eq!(handler.fact_count(), 10);
    }

    #[test]
    fn emit_status_does_not_increment_count() {
        let mut handler = StreamingHandler::human();
        handler.emit_entry(1, "fact", "test", "content");
        handler.emit_status("run_123", "converged", 5, 3);

        assert_eq!(handler.fact_count(), 1); // Only entries count
    }

    #[test]
    fn emit_waiting_does_not_increment_count() {
        let mut handler = StreamingHandler::human();
        handler.emit_entry(1, "fact", "test", "content");
        handler.emit_waiting("run_123", &["manager".to_string()]);

        assert_eq!(handler.fact_count(), 1);
    }

    #[test]
    fn emit_halt_does_not_increment_count() {
        let mut handler = StreamingHandler::human();
        handler.emit_entry(1, "fact", "test", "content");
        handler.emit_halt("run_123", "Invariant violated", "money.period.closed");

        assert_eq!(handler.fact_count(), 1);
    }

    // ==========================================================================
    // Unit Tests - Human Format Output
    // ==========================================================================

    #[test]
    fn format_human_entry_basic() {
        let output = format_human_entry(1, "fact", "signal:growth", "Market growing");
        assert_eq!(output, "[seq:1] fact:signal:growth | Market growing");
    }

    #[test]
    fn format_human_entry_high_sequence() {
        let output = format_human_entry(999_999, "trace", "trace:abc", "Action logged");
        assert_eq!(output, "[seq:999999] trace:trace:abc | Action logged");
    }

    #[test]
    fn format_human_entry_negative_sequence() {
        let output = format_human_entry(-1, "fact", "test", "Negative seq");
        assert_eq!(output, "[seq:-1] fact:test | Negative seq");
    }

    #[test]
    fn format_human_entry_empty_content() {
        let output = format_human_entry(1, "fact", "empty", "");
        assert_eq!(output, "[seq:1] fact:empty | ");
    }

    #[test]
    fn format_human_status_converged() {
        let output = format_human_status("run_abc123", "converged", 15, 6);
        assert_eq!(output, "[run_abc123] converged | 15 facts, 6 cycles");
    }

    #[test]
    fn format_human_status_halted() {
        let output = format_human_status("run_xyz", "halted", 3, 2);
        assert_eq!(output, "[run_xyz] halted | 3 facts, 2 cycles");
    }

    // ==========================================================================
    // Unit Tests - JSON Format Output
    // ==========================================================================

    #[test]
    fn format_json_entry_basic() {
        let output = format_json_entry(1, "fact", "signal:growth", "Market growing");
        assert!(output.is_some());

        let json: serde_json::Value = serde_json::from_str(&output.unwrap()).unwrap();
        assert_eq!(json["sequence"], 1);
        assert_eq!(json["type"], "fact");
        assert_eq!(json["entry_id"], "signal:growth");
        assert_eq!(json["content"], "Market growing");
    }

    #[test]
    fn format_json_entry_special_characters() {
        let output =
            format_json_entry(1, "fact", "test", "Content with \"quotes\" and \\backslash");
        assert!(output.is_some());

        // Should be valid JSON
        let json: serde_json::Value = serde_json::from_str(&output.unwrap()).unwrap();
        assert!(json["content"].as_str().unwrap().contains("quotes"));
    }

    #[test]
    fn format_json_entry_unicode() {
        let output = format_json_entry(1, "fact", "test", "Unicode: 日本語 émojis 🎉");
        assert!(output.is_some());

        let json: serde_json::Value = serde_json::from_str(&output.unwrap()).unwrap();
        assert!(json["content"].as_str().unwrap().contains("日本語"));
    }

    #[test]
    fn format_json_status_converged() {
        let output = format_json_status("run_abc", "converged", 10, 5);
        assert!(output.is_some());

        let json: serde_json::Value = serde_json::from_str(&output.unwrap()).unwrap();
        assert_eq!(json["run_id"], "run_abc");
        assert_eq!(json["type"], "status");
        assert_eq!(json["status"], "converged");
        assert_eq!(json["facts"], 10);
        assert_eq!(json["cycles"], 5);
    }

    // ==========================================================================
    // Negative Tests - Edge Cases
    // ==========================================================================

    #[test]
    fn format_entry_with_zero_sequence() {
        let output = format_human_entry(0, "fact", "first", "First fact");
        assert!(output.contains("[seq:0]"));
    }

    #[test]
    fn format_entry_with_max_i64_sequence() {
        let output = format_human_entry(i64::MAX, "fact", "huge", "Max sequence");
        assert!(output.contains(&i64::MAX.to_string()));
    }

    #[test]
    fn format_entry_with_min_i64_sequence() {
        let output = format_human_entry(i64::MIN, "fact", "negative", "Min sequence");
        assert!(output.contains(&i64::MIN.to_string()));
    }

    #[test]
    fn format_json_entry_empty_strings() {
        let output = format_json_entry(0, "", "", "");
        assert!(output.is_some());

        let json: serde_json::Value = serde_json::from_str(&output.unwrap()).unwrap();
        assert_eq!(json["type"], "");
        assert_eq!(json["entry_id"], "");
        assert_eq!(json["content"], "");
    }

    #[test]
    fn format_status_zero_counts() {
        let output = format_human_status("run_empty", "running", 0, 0);
        assert!(output.contains("0 facts, 0 cycles"));
    }

    #[test]
    fn format_status_negative_counts() {
        // While unusual, the format function shouldn't crash
        let output = format_human_status("run_weird", "error", -1, -1);
        assert!(output.contains("-1 facts, -1 cycles"));
    }

    // ==========================================================================
    // OutputFormat Tests
    // ==========================================================================

    #[test]
    fn output_format_equality() {
        assert_eq!(OutputFormat::Human, OutputFormat::Human);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Human, OutputFormat::Json);
    }

    #[test]
    fn output_format_clone() {
        let format = OutputFormat::Human;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn output_format_debug() {
        let human = format!("{:?}", OutputFormat::Human);
        let json = format!("{:?}", OutputFormat::Json);
        assert_eq!(human, "Human");
        assert_eq!(json, "Json");
    }
}

// ==========================================================================
// Property-Based Tests
// ==========================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn human_format_contains_sequence(seq in any::<i64>()) {
            let output = format_human_entry(seq, "fact", "id", "content");
            let expected = format!("[seq:{seq}]");
            prop_assert!(output.contains(&expected));
        }

        #[test]
        fn human_format_contains_entry_type(entry_type in "[a-z]{1,10}") {
            let output = format_human_entry(1, &entry_type, "id", "content");
            prop_assert!(output.contains(&entry_type));
        }

        #[test]
        fn human_format_contains_entry_id(entry_id in "[a-z0-9:-]{1,50}") {
            let output = format_human_entry(1, "fact", &entry_id, "content");
            prop_assert!(output.contains(&entry_id));
        }

        #[test]
        fn json_format_always_valid(
            seq in any::<i64>(),
            entry_type in "[a-z]{1,10}",
            entry_id in "[a-z0-9]{1,20}",
            content in "[a-zA-Z0-9 ]{0,100}"
        ) {
            let output = format_json_entry(seq, &entry_type, &entry_id, &content);
            prop_assert!(output.is_some());

            let json: Result<serde_json::Value, _> = serde_json::from_str(&output.unwrap());
            prop_assert!(json.is_ok());
        }

        #[test]
        fn json_status_always_valid(
            run_id in "[a-z0-9_]{1,20}",
            status in "(running|converged|halted|waiting)",
            facts in 0i32..1_000_000,
            cycles in 0i32..1000
        ) {
            let output = format_json_status(&run_id, &status, facts, cycles);
            prop_assert!(output.is_some());

            let json: Result<serde_json::Value, _> = serde_json::from_str(&output.unwrap());
            prop_assert!(json.is_ok());
        }

        #[test]
        #[allow(clippy::cast_possible_wrap)]
        fn fact_count_increments_correctly(n in 1usize..100) {
            let mut handler = StreamingHandler::human();
            for i in 0..n {
                handler.emit_entry(i as i64, "fact", "id", "content");
            }
            prop_assert_eq!(handler.fact_count(), n);
        }
    }
}
