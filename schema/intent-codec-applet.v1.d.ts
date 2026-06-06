export type AppletManifestVersion = "intent-codec-applet.v1";

export type AppletStatus =
  | "proposed"
  | "code-backed"
  | "executable"
  | "retired";

export type Reversibility =
  | "reversible"
  | "partially_reversible"
  | "irreversible";

export type EvidenceAuthority = "primary" | "corroborating" | "advisory";

export type ConflictPolicy =
  | "stop"
  | "ask_operator"
  | "prefer_primary"
  | "run_adversarial_review";

export interface FunctionalNeed {
  outcome: string;
  inputs: string[];
  output: string;
  constraints: string[];
  success_signal: string;
}

export interface EmotionalNeed {
  operator_anxiety: string;
  desired_confidence: string;
  tolerance: string;
}

export interface RelationalNeed {
  dependent_parties: string[];
  trust_obligation: string;
  handoff_created: string;
}

export interface AuthorityEnvelope {
  requester: string;
  approvers: string[];
  allowed_actions: string[];
  forbidden_actions: string[];
  approval_points: string[];
  reversibility: Reversibility;
  expiry: string;
  audit_visibility: string[];
}

export interface EvidenceSource {
  source: string;
  freshness: string;
  authority: EvidenceAuthority;
}

export interface EvidenceContract {
  required_sources: EvidenceSource[];
  disallowed_sources: string[];
  confidence_floor: string;
  conflict_policy: ConflictPolicy;
  sensitive_fields: string[];
}

export interface AppletProjection {
  operator_view: string;
  customer_or_partner_view?: string | null;
}

export interface AppletManifest {
  manifest_version: AppletManifestVersion;
  job_name: string;
  primary_job_key: string;
  status: AppletStatus;
  source_schema?: string | null;
  human_readable?: string | null;
  trigger: string;
  current_workaround: string;
  source_evidence?: Record<string, string>;
  functional_need: FunctionalNeed;
  emotional_need: EmotionalNeed;
  relational_need: RelationalNeed;
  failure_modes: string[];
  authority: AuthorityEnvelope;
  evidence_contract: EvidenceContract;
  runtime_needs: string[];
  commercial_needs: string[];
  projection: AppletProjection;
  non_goals: string[];
  layer_mapping: Record<string, string>;
}
