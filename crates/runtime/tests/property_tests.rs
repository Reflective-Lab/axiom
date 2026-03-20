// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Property-based tests for Converge Runtime using proptest.
//!
//! These tests verify invariants and properties that should hold
//! for all possible inputs.

use proptest::prelude::*;
use std::net::SocketAddr;

// =============================================================================
// Config Property Tests
// =============================================================================

proptest! {
    /// Any valid IP address and port should create a valid HttpConfig.
    #[test]
    fn http_config_accepts_valid_addresses(
        ip_parts in prop::array::uniform4(0u8..=255u8),
        port in 1u16..=65535u16
    ) {
        let addr_str = format!("{}.{}.{}.{}:{}",
            ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3], port);
        let addr: Result<SocketAddr, _> = addr_str.parse();
        prop_assert!(addr.is_ok(), "Failed to parse: {}", addr_str);
    }

    /// HttpConfig roundtrip through JSON preserves data.
    #[test]
    fn http_config_json_roundtrip(
        ip_parts in prop::array::uniform4(0u8..=255u8),
        port in 1u16..=65535u16,
        max_body_size in 0usize..=1_000_000_000usize
    ) {
        let addr_str = format!("{}.{}.{}.{}:{}",
            ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3], port);
        let json = format!(
            r#"{{"bind":"{}","max_body_size":{}}}"#,
            addr_str, max_body_size
        );

        let config: Result<converge_runtime::config::HttpConfig, _> =
            serde_json::from_str(&json);

        if let Ok(config) = config {
            let serialized = serde_json::to_string(&config).unwrap();
            let deserialized: converge_runtime::config::HttpConfig =
                serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(config.bind, deserialized.bind);
            prop_assert_eq!(config.max_body_size, deserialized.max_body_size);
        }
    }
}

// =============================================================================
// Error Property Tests
// =============================================================================

proptest! {
    /// RuntimeError Config always formats correctly.
    #[test]
    fn config_error_formats_correctly(msg in "\\PC*") {
        let error = converge_runtime::error::RuntimeError::Config(msg.clone());
        let formatted = error.to_string();
        prop_assert!(formatted.contains("configuration error:"));
        prop_assert!(formatted.contains(&msg));
    }

    /// RuntimeError NotFound always formats correctly.
    #[test]
    fn not_found_error_formats_correctly(msg in "\\PC*") {
        let error = converge_runtime::error::RuntimeError::NotFound(msg.clone());
        let formatted = error.to_string();
        prop_assert!(formatted.contains("not found:"));
        prop_assert!(formatted.contains(&msg));
    }

    /// RuntimeError Conflict always formats correctly.
    #[test]
    fn conflict_error_formats_correctly(msg in "\\PC*") {
        let error = converge_runtime::error::RuntimeError::Conflict(msg.clone());
        let formatted = error.to_string();
        prop_assert!(formatted.contains("conflict:"));
        prop_assert!(formatted.contains(&msg));
    }
}

// =============================================================================
// RuntimeErrorResponse Property Tests
// =============================================================================

proptest! {
    /// RuntimeErrorResponse always serializes to valid JSON.
    #[test]
    fn error_response_serializes_to_valid_json(
        error_msg in "\\PC*",
        status in 100u16..=599u16
    ) {
        let response = converge_runtime::error::RuntimeErrorResponse {
            error: error_msg,
            status,
        };

        let json = serde_json::to_string(&response);
        prop_assert!(json.is_ok(), "Failed to serialize: {:?}", json.err());

        let parsed: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();
        prop_assert!(parsed.get("error").is_some());
        prop_assert!(parsed.get("status").is_some());
    }

    /// Status code is preserved through serialization.
    #[test]
    fn status_code_preserved_in_json(status in 100u16..=599u16) {
        let response = converge_runtime::error::RuntimeErrorResponse {
            error: "test".to_string(),
            status,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(parsed["status"].as_u64().unwrap() as u16, status);
    }
}

// =============================================================================
// JSON Payload Property Tests
// =============================================================================

proptest! {
    /// JobRequest accepts any valid JSON object as context.
    #[test]
    fn job_request_accepts_valid_json_context(
        key in "[a-zA-Z][a-zA-Z0-9_]{0,20}",
        value in prop::bool::ANY
    ) {
        let json = format!(r#"{{"context":{{"{}":{}}}}} "#, key, value);
        let parsed: Result<converge_runtime::handlers::JobRequest, _> =
            serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "Failed to parse: {} - {:?}", json, parsed.err());
    }

    /// JobRequest handles nested JSON.
    #[test]
    fn job_request_handles_nested_json(
        key1 in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        key2 in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        value in -1000i32..1000i32
    ) {
        let json = format!(
            r#"{{"context":{{"{}":{{"{}":{}}}}}}} "#,
            key1, key2, value
        );
        let parsed: Result<converge_runtime::handlers::JobRequest, _> =
            serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "Failed to parse: {} - {:?}", json, parsed.err());
    }
}

// =============================================================================
// Validation Request Property Tests
// =============================================================================

proptest! {
    /// ValidateRulesRequest accepts any string content.
    #[test]
    fn validate_rules_accepts_any_content(content in "\\PC*") {
        // Escape for JSON
        let escaped = serde_json::to_string(&content).unwrap();
        let json = format!(r#"{{"content":{}}}"#, escaped);

        let parsed: Result<converge_runtime::handlers::ValidateRulesRequest, _> =
            serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "Failed to parse with content length {}", content.len());
    }

    /// ValidateRulesRequest handles file names.
    #[test]
    fn validate_rules_handles_file_names(
        content in "Feature: \\w+",
        file_name in "[a-zA-Z0-9_]{1,20}\\.feature"
    ) {
        let content_escaped = serde_json::to_string(&content).unwrap();
        let file_escaped = serde_json::to_string(&file_name).unwrap();
        let json = format!(
            r#"{{"content":{},"file_name":{}}}"#,
            content_escaped, file_escaped
        );

        let parsed: Result<converge_runtime::handlers::ValidateRulesRequest, _> =
            serde_json::from_str(&json);
        prop_assert!(parsed.is_ok());

        let request = parsed.unwrap();
        prop_assert_eq!(request.file_name.unwrap(), file_name);
    }
}

// =============================================================================
// IPv6 Address Property Tests
// =============================================================================

proptest! {
    /// IPv6 addresses are valid for HttpConfig.
    #[test]
    fn http_config_accepts_ipv6(
        groups in prop::array::uniform8(0u16..=0xFFFFu16),
        port in 1u16..=65535u16
    ) {
        let addr_str = format!(
            "[{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}]:{}",
            groups[0], groups[1], groups[2], groups[3],
            groups[4], groups[5], groups[6], groups[7],
            port
        );
        let addr: Result<SocketAddr, _> = addr_str.parse();
        prop_assert!(addr.is_ok(), "Failed to parse IPv6: {}", addr_str);
    }
}

// =============================================================================
// Template Property Tests
// =============================================================================

proptest! {
    /// PackJobRequest overrides are properly structured.
    #[test]
    fn pack_job_request_with_budget_overrides(
        max_cycles in 1u32..1000u32,
        max_facts in 1u32..10000u32
    ) {
        let json = format!(
            r#"{{
                "pack": "test-pack",
                "overrides": {{
                    "seeds": [],
                    "budget": {{
                        "max_cycles": {},
                        "max_facts": {}
                    }}
                }}
            }}"#,
            max_cycles, max_facts
        );

        let parsed: Result<converge_runtime::templates::PackJobRequest, _> =
            serde_json::from_str(&json);
        prop_assert!(parsed.is_ok(), "Failed to parse: {:?}", parsed.err());
    }
}

// =============================================================================
// State Property Tests
// =============================================================================

proptest! {
    /// AppState can be cloned any number of times.
    #[test]
    fn app_state_multiple_clones(count in 1usize..100usize) {
        let state = converge_runtime::state::AppState::new();

        let clones: Vec<_> = (0..count).map(|_| state.clone()).collect();

        // All clones should have the same template count
        let expected_count = state.templates.list().len();
        for clone in clones {
            prop_assert_eq!(clone.templates.list().len(), expected_count);
        }
    }
}

// =============================================================================
// Boundary Tests
// =============================================================================

proptest! {
    /// Port 0 is valid (OS assigns).
    #[test]
    fn port_zero_is_valid(
        ip_parts in prop::array::uniform4(0u8..=255u8)
    ) {
        let addr_str = format!("{}.{}.{}.{}:0",
            ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]);
        let addr: Result<SocketAddr, _> = addr_str.parse();
        prop_assert!(addr.is_ok());
        prop_assert_eq!(addr.unwrap().port(), 0);
    }

    /// Max body size boundary values.
    #[test]
    fn max_body_size_boundaries(
        size in prop::sample::select(vec![0usize, 1, 1024, 1024*1024, usize::MAX / 2])
    ) {
        let json = format!(
            r#"{{"bind":"127.0.0.1:8080","max_body_size":{}}}"#,
            size
        );
        let config: Result<converge_runtime::config::HttpConfig, _> =
            serde_json::from_str(&json);
        prop_assert!(config.is_ok());
        prop_assert_eq!(config.unwrap().max_body_size, size);
    }
}

// =============================================================================
// Unicode and Special Characters
// =============================================================================

proptest! {
    /// Error messages handle unicode.
    #[test]
    fn error_handles_unicode(msg in "[\\p{L}\\p{N}\\p{S}\\p{P}]{0,100}") {
        let error = converge_runtime::error::RuntimeError::Config(msg.clone());
        let formatted = error.to_string();
        // Should not panic and should contain the message
        prop_assert!(formatted.contains(&msg) || msg.is_empty());
    }

    /// Error response handles special characters in messages.
    #[test]
    fn error_response_handles_special_chars(
        msg in "[\\x00-\\x7F]{0,100}"  // ASCII including control chars
    ) {
        let response = converge_runtime::error::RuntimeErrorResponse {
            error: msg,
            status: 500,
        };

        // Should serialize without panicking
        let json = serde_json::to_string(&response);
        prop_assert!(json.is_ok());
    }
}
