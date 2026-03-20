// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Integration tests for converge-traits.

use super::*;

// ── Mock backends ─────────────────────────────────────────────────────

struct MockLlmBackend;

impl Backend for MockLlmBackend {
    fn name(&self) -> &str {
        "mock-llm"
    }
    fn kind(&self) -> BackendKind {
        BackendKind::Llm
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::TextGeneration,
            Capability::Reasoning,
            Capability::CodeGeneration,
        ]
    }
    fn supports_replay(&self) -> bool {
        false
    }
}

struct MockPolicyBackend;

impl Backend for MockPolicyBackend {
    fn name(&self) -> &str {
        "mock-cedar"
    }
    fn kind(&self) -> BackendKind {
        BackendKind::Policy
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::AccessControl,
            Capability::ComplianceCheck,
            Capability::AuditTrail,
        ]
    }
    fn supports_replay(&self) -> bool {
        true
    }
    fn requires_network(&self) -> bool {
        false
    }
}

struct MockOptimizerBackend;

impl Backend for MockOptimizerBackend {
    fn name(&self) -> &str {
        "mock-cpsat"
    }
    fn kind(&self) -> BackendKind {
        BackendKind::Optimization
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::ConstraintSolving,
            Capability::ResourceAllocation,
            Capability::Scheduling,
        ]
    }
    fn supports_replay(&self) -> bool {
        true
    }
    fn requires_network(&self) -> bool {
        false
    }
}

struct MockAnalyticsBackend;

impl Backend for MockAnalyticsBackend {
    fn name(&self) -> &str {
        "mock-burn"
    }
    fn kind(&self) -> BackendKind {
        BackendKind::Analytics
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Embedding,
            Capability::Classification,
            Capability::AnomalyDetection,
        ]
    }
}

struct MockLocalLlmBackend;

impl Backend for MockLocalLlmBackend {
    fn name(&self) -> &str {
        "mock-local-llama"
    }
    fn kind(&self) -> BackendKind {
        BackendKind::Llm
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::TextGeneration,
            Capability::Offline,
            Capability::Replay,
        ]
    }
    fn supports_replay(&self) -> bool {
        true
    }
    fn requires_network(&self) -> bool {
        false
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[test]
fn all_backend_kinds_are_first_class() {
    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(MockLlmBackend),
        Box::new(MockPolicyBackend),
        Box::new(MockOptimizerBackend),
        Box::new(MockAnalyticsBackend),
        Box::new(MockLocalLlmBackend),
    ];

    for backend in &backends {
        assert!(!backend.name().is_empty());
        assert!(!backend.capabilities().is_empty());
        let _prov = backend.provenance("test-123");
    }
}

#[test]
fn capability_matching() {
    let llm = MockLlmBackend;
    assert!(llm.has_capability(Capability::TextGeneration));
    assert!(llm.has_capability(Capability::Reasoning));
    assert!(!llm.has_capability(Capability::AccessControl));

    let policy = MockPolicyBackend;
    assert!(policy.has_capability(Capability::AccessControl));
    assert!(!policy.has_capability(Capability::TextGeneration));
}

#[test]
fn replay_and_network_properties() {
    let cloud_llm = MockLlmBackend;
    assert!(!cloud_llm.supports_replay());
    assert!(cloud_llm.requires_network());

    let local_llm = MockLocalLlmBackend;
    assert!(local_llm.supports_replay());
    assert!(!local_llm.requires_network());

    let policy = MockPolicyBackend;
    assert!(policy.supports_replay());
    assert!(!policy.requires_network());
}

#[test]
fn backend_kind_display() {
    assert_eq!(BackendKind::Llm.to_string(), "llm");
    assert_eq!(BackendKind::Policy.to_string(), "policy");
    assert_eq!(BackendKind::Optimization.to_string(), "optimization");
    assert_eq!(BackendKind::Analytics.to_string(), "analytics");
    assert_eq!(BackendKind::Search.to_string(), "search");
    assert_eq!(BackendKind::Storage.to_string(), "storage");
    assert_eq!(
        BackendKind::Other("custom".into()).to_string(),
        "other:custom"
    );
}

#[test]
fn backend_kind_serialization() {
    let json = serde_json::to_string(&BackendKind::Llm).unwrap();
    assert_eq!(json, "\"Llm\"");

    let parsed: BackendKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, BackendKind::Llm);

    let other = BackendKind::Other("bioinformatics".into());
    let json = serde_json::to_string(&other).unwrap();
    let parsed: BackendKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, other);
}

#[test]
fn error_retryable_classification() {
    assert!(BackendError::timeout("too slow").is_retryable());
    assert!(BackendError::rate_limit("429").is_retryable());
    assert!(BackendError::network("connection reset").is_retryable());
    assert!(BackendError::unavailable("service down").is_retryable());

    assert!(!BackendError::auth("bad key").is_retryable());
    assert!(!BackendError::invalid_request("missing field").is_retryable());
    assert!(!BackendError::parse("bad json").is_retryable());
    assert!(!BackendError::unsupported(Capability::Reasoning).is_retryable());
}

#[test]
fn error_serialization() {
    let err = BackendError::timeout("exceeded 5000ms");
    let json = serde_json::to_string(&err).unwrap();
    let parsed: BackendError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.kind, BackendErrorKind::Timeout);
    assert!(parsed.retryable);
}

#[test]
fn provenance_format() {
    let llm = MockLlmBackend;
    assert_eq!(llm.provenance("req-42"), "mock-llm:req-42");

    let policy = MockPolicyBackend;
    assert_eq!(policy.provenance("eval-7"), "mock-cedar:eval-7");
}

#[test]
fn capability_serialization() {
    let cap = Capability::TextGeneration;
    let json = serde_json::to_string(&cap).unwrap();
    assert_eq!(json, "\"TextGeneration\"");

    let other = Capability::Other("quantum-annealing".into());
    let json = serde_json::to_string(&other).unwrap();
    let parsed: Capability = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, other);
}

#[test]
fn heterogeneous_backend_collection() {
    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(MockLlmBackend),
        Box::new(MockPolicyBackend),
        Box::new(MockOptimizerBackend),
        Box::new(MockAnalyticsBackend),
    ];

    // Find all backends that support replay
    let replayable: Vec<_> = backends
        .iter()
        .filter(|b| b.supports_replay())
        .map(|b| b.name())
        .collect();
    assert_eq!(replayable, vec!["mock-cedar", "mock-cpsat"]);

    // Find all backends that can run offline
    let offline: Vec<_> = backends
        .iter()
        .filter(|b| !b.requires_network())
        .map(|b| b.name())
        .collect();
    assert_eq!(offline, vec!["mock-cedar", "mock-cpsat"]);

    // Find backend with specific capability
    let reasoning: Vec<_> = backends
        .iter()
        .filter(|b| b.has_capability(Capability::Reasoning))
        .map(|b| b.name())
        .collect();
    assert_eq!(reasoning, vec!["mock-llm"]);
}
