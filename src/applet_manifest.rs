//! Applet manifest validation.
//!
//! Applet manifests are app-neutral JTBD contracts. They are not UI state and
//! not executable code. Axiom owns this typed shape so Helm, apps, Arena, and
//! Atelier can validate the same intent boundary before projecting it into
//! Truth Packages, operator surfaces, WASM artifacts, or app-specific contracts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current applet manifest version.
pub const APPLET_MANIFEST_VERSION: &str = "intent-codec-applet.v1";

/// JSON Schema artifact for [`APPLET_MANIFEST_VERSION`].
pub const APPLET_MANIFEST_JSON_SCHEMA: &str =
    include_str!("../schema/intent-codec-applet.v1.schema.json");

/// TypeScript declarations for [`APPLET_MANIFEST_VERSION`].
pub const APPLET_MANIFEST_TYPESCRIPT_DECLARATIONS: &str =
    include_str!("../schema/intent-codec-applet.v1.d.ts");

/// Machine-readable Intent Codec applet manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppletManifest {
    pub manifest_version: String,
    pub job_name: String,
    pub primary_job_key: String,
    pub status: AppletStatus,
    #[serde(default)]
    pub source_schema: Option<String>,
    #[serde(default)]
    pub human_readable: Option<String>,
    pub trigger: String,
    pub current_workaround: String,
    #[serde(default)]
    pub source_evidence: BTreeMap<String, String>,
    pub functional_need: FunctionalNeed,
    pub emotional_need: EmotionalNeed,
    pub relational_need: RelationalNeed,
    pub failure_modes: Vec<String>,
    pub authority: AuthorityEnvelope,
    pub evidence_contract: EvidenceContract,
    pub runtime_needs: Vec<String>,
    pub commercial_needs: Vec<String>,
    pub projection: AppletProjection,
    pub non_goals: Vec<String>,
    #[serde(default)]
    pub layer_mapping: BTreeMap<String, String>,
}

impl AppletManifest {
    /// Validate required applet semantics after JSON parsing.
    ///
    /// Serde enforces field presence, unknown-field rejection, and enum values.
    /// This method enforces business-shape invariants that JSON shape alone
    /// cannot express well.
    ///
    /// # Errors
    ///
    /// Returns [`AppletManifestError::Invalid`] with the first invalid field.
    pub fn validate(&self) -> Result<(), AppletManifestError> {
        if self.manifest_version != APPLET_MANIFEST_VERSION {
            return Err(invalid(
                "manifest_version",
                format!("expected {APPLET_MANIFEST_VERSION}"),
            ));
        }

        ensure_non_blank("job_name", &self.job_name)?;
        ensure_non_blank("primary_job_key", &self.primary_job_key)?;
        ensure_non_blank("trigger", &self.trigger)?;
        ensure_non_blank("current_workaround", &self.current_workaround)?;
        ensure_optional_non_blank("source_schema", self.source_schema.as_deref())?;
        ensure_optional_non_blank("human_readable", self.human_readable.as_deref())?;
        ensure_string_map_entries("source_evidence", &self.source_evidence)?;

        self.functional_need.validate()?;
        self.emotional_need.validate()?;
        self.relational_need.validate()?;
        self.authority.validate()?;
        self.evidence_contract.validate()?;
        self.projection.validate()?;

        ensure_string_list("failure_modes", &self.failure_modes)?;
        ensure_string_list("runtime_needs", &self.runtime_needs)?;
        ensure_string_list("commercial_needs", &self.commercial_needs)?;
        ensure_string_list("non_goals", &self.non_goals)?;
        ensure_string_map("layer_mapping", &self.layer_mapping)?;

        Ok(())
    }
}

/// Applet lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppletStatus {
    Proposed,
    CodeBacked,
    Executable,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalNeed {
    pub outcome: String,
    pub inputs: Vec<String>,
    pub output: String,
    pub constraints: Vec<String>,
    pub success_signal: String,
}

impl FunctionalNeed {
    fn validate(&self) -> Result<(), AppletManifestError> {
        ensure_non_blank("functional_need.outcome", &self.outcome)?;
        ensure_string_list("functional_need.inputs", &self.inputs)?;
        ensure_non_blank("functional_need.output", &self.output)?;
        ensure_string_list("functional_need.constraints", &self.constraints)?;
        ensure_non_blank("functional_need.success_signal", &self.success_signal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmotionalNeed {
    pub operator_anxiety: String,
    pub desired_confidence: String,
    pub tolerance: String,
}

impl EmotionalNeed {
    fn validate(&self) -> Result<(), AppletManifestError> {
        ensure_non_blank("emotional_need.operator_anxiety", &self.operator_anxiety)?;
        ensure_non_blank(
            "emotional_need.desired_confidence",
            &self.desired_confidence,
        )?;
        ensure_non_blank("emotional_need.tolerance", &self.tolerance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalNeed {
    pub dependent_parties: Vec<String>,
    pub trust_obligation: String,
    pub handoff_created: String,
}

impl RelationalNeed {
    fn validate(&self) -> Result<(), AppletManifestError> {
        ensure_string_list("relational_need.dependent_parties", &self.dependent_parties)?;
        ensure_non_blank("relational_need.trust_obligation", &self.trust_obligation)?;
        ensure_non_blank("relational_need.handoff_created", &self.handoff_created)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityEnvelope {
    pub requester: String,
    pub approvers: Vec<String>,
    pub allowed_actions: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub approval_points: Vec<String>,
    pub reversibility: Reversibility,
    pub expiry: String,
    pub audit_visibility: Vec<String>,
}

impl AuthorityEnvelope {
    fn validate(&self) -> Result<(), AppletManifestError> {
        ensure_non_blank("authority.requester", &self.requester)?;
        ensure_string_list("authority.approvers", &self.approvers)?;
        ensure_string_list("authority.allowed_actions", &self.allowed_actions)?;
        ensure_string_list("authority.forbidden_actions", &self.forbidden_actions)?;
        ensure_string_list("authority.approval_points", &self.approval_points)?;
        ensure_non_blank("authority.expiry", &self.expiry)?;
        ensure_string_list("authority.audit_visibility", &self.audit_visibility)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Reversible,
    PartiallyReversible,
    Irreversible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceContract {
    pub required_sources: Vec<EvidenceSource>,
    #[serde(default)]
    pub disallowed_sources: Vec<String>,
    pub confidence_floor: String,
    pub conflict_policy: ConflictPolicy,
    #[serde(default)]
    pub sensitive_fields: Vec<String>,
}

impl EvidenceContract {
    fn validate(&self) -> Result<(), AppletManifestError> {
        if self.required_sources.is_empty() {
            return Err(invalid(
                "evidence_contract.required_sources",
                "must not be empty",
            ));
        }
        for (idx, source) in self.required_sources.iter().enumerate() {
            source.validate(idx)?;
        }
        if !self
            .required_sources
            .iter()
            .any(|source| source.authority == EvidenceAuthority::Primary)
        {
            return Err(invalid(
                "evidence_contract.required_sources",
                "must include at least one primary source",
            ));
        }
        ensure_string_list(
            "evidence_contract.disallowed_sources",
            &self.disallowed_sources,
        )?;
        ensure_non_blank("evidence_contract.confidence_floor", &self.confidence_floor)?;
        ensure_string_list("evidence_contract.sensitive_fields", &self.sensitive_fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSource {
    pub source: String,
    pub freshness: String,
    pub authority: EvidenceAuthority,
}

impl EvidenceSource {
    fn validate(&self, idx: usize) -> Result<(), AppletManifestError> {
        ensure_non_blank(
            &format!("evidence_contract.required_sources[{idx}].source"),
            &self.source,
        )?;
        ensure_non_blank(
            &format!("evidence_contract.required_sources[{idx}].freshness"),
            &self.freshness,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAuthority {
    Primary,
    Corroborating,
    Advisory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Stop,
    AskOperator,
    PreferPrimary,
    RunAdversarialReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppletProjection {
    pub operator_view: String,
    #[serde(default)]
    pub customer_or_partner_view: Option<String>,
}

impl AppletProjection {
    fn validate(&self) -> Result<(), AppletManifestError> {
        ensure_non_blank("projection.operator_view", &self.operator_view)?;
        if let Some(value) = self.customer_or_partner_view.as_ref() {
            ensure_non_blank("projection.customer_or_partner_view", value)?;
        }
        Ok(())
    }
}

/// Applet manifest parse/validation errors.
#[derive(Debug, thiserror::Error)]
pub enum AppletManifestError {
    #[error("applet manifest JSON did not parse: {0}")]
    Json(#[from] serde_json::Error),
    #[error("applet manifest field '{field}' invalid: {message}")]
    Invalid { field: String, message: String },
}

/// Parse and validate an applet manifest JSON document.
///
/// # Errors
///
/// Returns JSON parse errors or semantic validation errors.
pub fn parse_applet_manifest_json(source: &str) -> Result<AppletManifest, AppletManifestError> {
    let manifest: AppletManifest = serde_json::from_str(source)?;
    manifest.validate()?;
    Ok(manifest)
}

/// Parse and validate an applet manifest from a JSON value.
///
/// # Errors
///
/// Returns JSON shape errors or semantic validation errors.
pub fn parse_applet_manifest_value(
    value: serde_json::Value,
) -> Result<AppletManifest, AppletManifestError> {
    let manifest: AppletManifest = serde_json::from_value(value)?;
    manifest.validate()?;
    Ok(manifest)
}

/// Parse the Axiom-owned applet manifest JSON Schema artifact.
///
/// # Errors
///
/// Returns a JSON parse error if the packaged schema artifact is malformed.
pub fn applet_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(APPLET_MANIFEST_JSON_SCHEMA)
}

fn ensure_non_blank(field: &str, value: &str) -> Result<(), AppletManifestError> {
    if value.trim().is_empty() {
        return Err(invalid(field, "must not be blank"));
    }
    Ok(())
}

fn ensure_optional_non_blank(field: &str, value: Option<&str>) -> Result<(), AppletManifestError> {
    if let Some(value) = value {
        ensure_non_blank(field, value)?;
    }
    Ok(())
}

fn ensure_string_list(field: &str, values: &[String]) -> Result<(), AppletManifestError> {
    if values.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    for (idx, value) in values.iter().enumerate() {
        ensure_non_blank(&format!("{field}[{idx}]"), value)?;
    }
    Ok(())
}

fn ensure_string_map(
    field: &str,
    values: &BTreeMap<String, String>,
) -> Result<(), AppletManifestError> {
    if values.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    ensure_string_map_entries(field, values)
}

fn ensure_string_map_entries(
    field: &str,
    values: &BTreeMap<String, String>,
) -> Result<(), AppletManifestError> {
    for (key, value) in values {
        ensure_non_blank(&format!("{field} key"), key)?;
        ensure_non_blank(&format!("{field}.{key}"), value)?;
    }
    Ok(())
}

fn invalid(field: impl Into<String>, message: impl Into<String>) -> AppletManifestError {
    AppletManifestError::Invalid {
        field: field.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"{
      "manifest_version": "intent-codec-applet.v1",
      "job_name": "Run governed inquiry",
      "primary_job_key": "quorum-adaptive-inquiry",
      "status": "code-backed",
      "trigger": "adaptive_inquiry_requested",
      "current_workaround": "Facilitator manually reconciles workshop signals.",
      "functional_need": {
        "outcome": "Turn live uncertainty into an evidence-backed process receipt.",
        "inputs": ["core_question", "participants"],
        "output": "Contracted inquiry and receipt.",
        "constraints": ["signals require consent"],
        "success_signal": "receipt cites evidence and dissent"
      },
      "emotional_need": {
        "operator_anxiety": "Hidden dissent may be erased.",
        "desired_confidence": "The run did not manufacture consensus.",
        "tolerance": "Prefer deferral over false certainty."
      },
      "relational_need": {
        "dependent_parties": ["facilitator", "participants"],
        "trust_obligation": "Explain how signals shaped the result.",
        "handoff_created": "Open research task when evidence is missing."
      },
      "failure_modes": ["suppress minority hypothesis"],
      "authority": {
        "requester": "facilitator",
        "approvers": ["facilitator"],
        "allowed_actions": ["open contracted inquiry"],
        "forbidden_actions": ["steer toward predetermined conclusion"],
        "approval_points": ["contract creation"],
        "reversibility": "partially_reversible",
        "expiry": "contract time budget",
        "audit_visibility": ["facilitator"]
      },
      "evidence_contract": {
        "required_sources": [
          {
            "source": "participant signal",
            "freshness": "during run",
            "authority": "primary"
          }
        ],
        "disallowed_sources": ["uncited summary"],
        "confidence_floor": "contract evidence topology",
        "conflict_policy": "run_adversarial_review",
        "sensitive_fields": ["participant_id"]
      },
      "runtime_needs": ["auth claim"],
      "commercial_needs": ["entitlement outside applet"],
      "projection": {
        "operator_view": "contract, events, outcome, and receipt",
        "customer_or_partner_view": "support-safe status"
      },
      "non_goals": ["generic survey builder"],
      "layer_mapping": {
        "axiom": "typed manifest",
        "helm": "operator review"
      }
    }"#;

    #[test]
    fn parses_valid_applet_manifest() {
        let manifest = parse_applet_manifest_json(VALID_MANIFEST).expect("manifest parses");
        assert_eq!(manifest.manifest_version, APPLET_MANIFEST_VERSION);
        assert_eq!(manifest.primary_job_key, "quorum-adaptive-inquiry");
        assert_eq!(manifest.status, AppletStatus::CodeBacked);
        assert_eq!(
            manifest.evidence_contract.required_sources[0].authority,
            EvidenceAuthority::Primary
        );
    }

    #[test]
    fn rejects_manifest_without_primary_evidence() {
        let mut manifest: AppletManifest =
            serde_json::from_str(VALID_MANIFEST).expect("manifest parses structurally");
        manifest.evidence_contract.required_sources[0].authority = EvidenceAuthority::Advisory;

        let err = manifest
            .validate()
            .expect_err("primary evidence should be required");
        assert!(err.to_string().contains("primary source"));
    }

    #[test]
    fn rejects_blank_jtbd_lanes() {
        let mut manifest: AppletManifest =
            serde_json::from_str(VALID_MANIFEST).expect("manifest parses structurally");
        manifest.emotional_need.operator_anxiety.clear();

        let err = manifest
            .validate()
            .expect_err("emotional lane should be required");
        assert!(err.to_string().contains("operator_anxiety"));
    }

    #[test]
    fn schema_artifact_tracks_manifest_version_and_required_boundaries() {
        let schema = applet_manifest_json_schema().expect("schema artifact parses");
        assert_eq!(
            schema["properties"]["manifest_version"]["const"],
            APPLET_MANIFEST_VERSION
        );

        let required = schema["required"]
            .as_array()
            .expect("schema has top-level required fields");
        for field in [
            "functional_need",
            "emotional_need",
            "relational_need",
            "authority",
            "evidence_contract",
            "projection",
            "layer_mapping",
        ] {
            assert!(
                required.iter().any(|value| value == field),
                "schema should require {field}"
            );
        }

        assert_eq!(
            schema["$defs"]["evidence_contract"]["properties"]["required_sources"]["contains"]["properties"]
                ["authority"]["const"],
            "primary"
        );
    }

    #[test]
    fn typescript_declarations_track_manifest_version() {
        assert!(
            APPLET_MANIFEST_TYPESCRIPT_DECLARATIONS.contains("export interface AppletManifest")
        );
        assert!(APPLET_MANIFEST_TYPESCRIPT_DECLARATIONS.contains(APPLET_MANIFEST_VERSION));
        assert!(APPLET_MANIFEST_TYPESCRIPT_DECLARATIONS.contains("layer_mapping"));
    }
}
