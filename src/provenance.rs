// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Axiom provenance for Truth-Package-seeded Converge proposals.

use crate::truth_package::{
    ClauseFingerprint, ClauseId, JtbdClause, JtbdClauseKind, TruthPackage, TruthPackageId,
};
use converge_pack::{ContextKey, FactPayload, ProposedFact, ProvenanceSource};
use serde::{Deserialize, Serialize};

/// Zero-sized provenance marker for Axiom-authored Truth Package proposals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AxiomTruth;

impl ProvenanceSource for AxiomTruth {
    fn as_str(&self) -> &'static str {
        "axiom_truth_package"
    }
}

/// Canonical provenance source for Axiom Truth Package facts.
pub const AXIOM_PROVENANCE: AxiomTruth = AxiomTruth;

/// Typed seed payload for facts emitted from a Truth Package clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TruthPackageSeedPayload {
    pub package_id: TruthPackageId,
    pub truth_version: String,
    pub clause_id: ClauseId,
    pub clause_kind: JtbdClauseKind,
    pub clause_fingerprint: ClauseFingerprint,
    pub clause_text: String,
}

impl TruthPackageSeedPayload {
    pub fn from_clause(package: &TruthPackage, clause: &JtbdClause) -> Self {
        Self {
            package_id: package.package_id.clone(),
            truth_version: package.truth_version.clone(),
            clause_id: clause.id.clone(),
            clause_kind: clause.kind,
            clause_fingerprint: clause.fingerprint.clone(),
            clause_text: clause.canonical_text.clone(),
        }
    }
}

impl FactPayload for TruthPackageSeedPayload {
    const FAMILY: &'static str = "axiom.truth_package_seed";
    const VERSION: u16 = 1;
}

/// Build one Axiom-stamped seed proposal for a Truth Package clause.
pub fn truth_package_seed_fact(package: &TruthPackage, clause: &JtbdClause) -> ProposedFact {
    AXIOM_PROVENANCE.proposed_fact(
        ContextKey::Seeds,
        truth_package_seed_fact_id(package, clause),
        TruthPackageSeedPayload::from_clause(package, clause),
    )
}

/// Build Axiom-stamped seed proposals for every source JTBD clause.
pub fn truth_package_seed_facts(package: &TruthPackage) -> Vec<ProposedFact> {
    package
        .source_jtbd
        .clauses
        .iter()
        .map(|clause| truth_package_seed_fact(package, clause))
        .collect()
}

fn truth_package_seed_fact_id(package: &TruthPackage, clause: &JtbdClause) -> String {
    format!(
        "axiom_truth_package:{}:{}",
        package.package_id.as_str(),
        clause.id.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truth_package::{ClauseInput, JtbdInput, decode_jtbd};

    fn vendor_package() -> TruthPackage {
        decode_jtbd(JtbdInput {
            key: "Vendor Commitment".to_string(),
            actor: "finance controller".to_string(),
            functional_job: "approve a vendor commitment".to_string(),
            so_that: "spend is traceable and policy-compliant".to_string(),
            evidence_required: vec![ClauseInput::new("vendor assessment")],
            failure_modes: vec![ClauseInput::new("bypassed approval")],
        })
        .unwrap()
    }

    #[test]
    fn axiom_provenance_stamps_truth_package_seed_facts() {
        let package = vendor_package();
        let clause = package.source_jtbd.clauses.first().unwrap();

        let proposal = truth_package_seed_fact(&package, clause);
        let wire = proposal.to_wire().unwrap();

        assert_eq!(AXIOM_PROVENANCE.as_str(), "axiom_truth_package");
        assert_eq!(wire.key, ContextKey::Seeds);
        assert_eq!(wire.provenance.as_str(), AXIOM_PROVENANCE.as_str());
        assert_eq!(
            wire.payload.family.as_str(),
            TruthPackageSeedPayload::FAMILY
        );
        assert_eq!(wire.payload.version.get(), TruthPackageSeedPayload::VERSION);
        assert_eq!(
            wire.payload.payload["package_id"],
            serde_json::json!(package.package_id)
        );
        assert_eq!(
            wire.payload.payload["clause_id"],
            serde_json::json!(clause.id)
        );
        assert_eq!(
            wire.payload.payload["clause_fingerprint"],
            serde_json::json!(clause.fingerprint)
        );
    }

    #[test]
    fn truth_package_seed_facts_cover_every_source_clause() {
        let package = vendor_package();
        let proposals = truth_package_seed_facts(&package);

        assert_eq!(proposals.len(), package.source_jtbd.clauses.len());
        assert!(proposals.iter().all(|proposal| {
            proposal.key == ContextKey::Seeds
                && proposal.provenance.as_str() == AXIOM_PROVENANCE.as_str()
        }));
    }
}
