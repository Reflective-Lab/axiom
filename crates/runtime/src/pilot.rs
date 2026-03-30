//! Release-readiness pilot HTTP endpoints.
//!
//! This module provides an end-to-end, invite-gated demo transport for the
//! release-readiness UI. It exposes:
//! - `POST /api/pilot/release-readiness/runs`
//! - `GET  /api/pilot/release-readiness/runs/:run_id/events?since=...`

use axum::{
    Json, Router,
    extract::{Path, Query},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{LazyLock, RwLock};
use std::time::Duration;
use tracing::warn;

use crate::error::RuntimeError;

static RELEASE_READINESS_RUNS: LazyLock<RwLock<HashMap<String, Vec<RunEventEnvelope>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

const DEFAULT_RELEASE_WINDOW: &str = "2026-03-05 19:00-20:00 UTC";
const REQUIRED_SIGNAL_IDS: [&str; 10] = [
    "signal:coverage:unit",
    "signal:coverage:integration",
    "signal:tests:status",
    "signal:security:vulnerabilities",
    "signal:security:licenses",
    "signal:dependencies:outdated",
    "signal:dependencies:pinning",
    "signal:performance:regression",
    "signal:docs:changelog",
    "signal:docs:api",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunRequest {
    pub candidate: Option<String>,
    pub environment: Option<String>,
    pub release_window: Option<String>,
    pub owner: Option<String>,
    pub notes: Option<String>,
    pub invite_session_id: Option<String>,
    pub invite_code_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunResponse {
    #[serde(rename = "runId")]
    pub run_id: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "streamPath")]
    pub stream_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQuery {
    pub since: Option<u64>,
    pub token: Option<String>,
    pub candidate: Option<String>,
    pub environment: Option<String>,
    pub release_window: Option<String>,
    pub invite_session_id: Option<String>,
    pub invite_code_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEventEnvelope {
    #[serde(rename = "runId")]
    pub run_id: String,
    pub sequence: u64,
    #[serde(rename = "emittedAt")]
    pub emitted_at: String,
    pub event: ReleaseRunEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReleaseRunEvent {
    #[serde(rename = "run.started")]
    RunStarted {
        candidate: String,
        environment: String,
        #[serde(rename = "releaseWindow")]
        release_window: String,
    },
    #[serde(rename = "log.append")]
    LogAppend { message: String, level: String },
    #[serde(rename = "step.started")]
    StepStarted {
        #[serde(rename = "stepId")]
        step_id: String,
    },
    #[serde(rename = "step.completed")]
    StepCompleted {
        #[serde(rename = "stepId")]
        step_id: String,
    },
    #[serde(rename = "summary.ready")]
    SummaryReady { summary: RunSummary },
    #[serde(rename = "run.completed")]
    RunCompleted { recommendation: String },
    #[serde(rename = "run.failed")]
    RunFailed { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    #[serde(rename = "runId")]
    pub run_id: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    pub recommendation: String,
    #[serde(rename = "riskStatus")]
    pub risk_status: String,
    #[serde(rename = "reviewFlags")]
    pub review_flags: Vec<String>,
    #[serde(rename = "hardGates")]
    pub hard_gates: Vec<GateResult>,
    pub highlights: Vec<String>,
    #[serde(rename = "evidenceSignalIds")]
    pub evidence_signal_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub status: String,
    pub detail: String,
}

fn normalized_candidate(candidate: Option<String>) -> String {
    candidate
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "v1.4.0-rc.2".to_string())
}

fn normalized_environment(environment: Option<String>) -> String {
    match environment.as_deref() {
        Some("staging") => "staging".to_string(),
        _ => "production".to_string(),
    }
}

fn normalized_release_window(window: Option<String>) -> String {
    window
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .unwrap_or_else(|| DEFAULT_RELEASE_WINDOW.to_string())
}

fn emitted_at_for_sequence(sequence: u64) -> String {
    let minute = (sequence / 60) % 60;
    let second = sequence % 60;
    format!("2026-03-02T12:{minute:02}:{second:02}.000Z")
}

fn build_release_readiness_script(
    run_id: &str,
    candidate: &str,
    environment: &str,
    release_window: &str,
) -> Vec<RunEventEnvelope> {
    const STEPS: [(&str, &str, &str); 4] = [
        (
            "checks",
            "Collect release checks",
            "Coverage, security, dependency, performance, and docs summaries ingested.",
        ),
        (
            "normalize",
            "Normalize signals",
            "Convert raw check summaries into deterministic release-readiness facts.",
        ),
        (
            "evaluate",
            "Evaluate gates",
            "Apply completeness, minimum coverage, and vulnerability gates.",
        ),
        (
            "summarize",
            "Generate risk summary",
            "Surface review flags and recommend decision posture for human signoff.",
        ),
    ];

    fn push_event(
        events: &mut Vec<RunEventEnvelope>,
        run_id: &str,
        sequence: &mut u64,
        event: ReleaseRunEvent,
    ) {
        events.push(RunEventEnvelope {
            run_id: run_id.to_string(),
            sequence: *sequence,
            emitted_at: emitted_at_for_sequence(*sequence),
            event,
        });
        *sequence += 1;
    }

    let mut events = Vec::new();
    let mut sequence: u64 = 1;

    push_event(
        &mut events,
        run_id,
        &mut sequence,
        ReleaseRunEvent::RunStarted {
            candidate: candidate.to_string(),
            environment: environment.to_string(),
            release_window: release_window.to_string(),
        },
    );

    push_event(
        &mut events,
        run_id,
        &mut sequence,
        ReleaseRunEvent::LogAppend {
            message: format!("Starting release-readiness run for {candidate} ({environment})"),
            level: "info".to_string(),
        },
    );

    for (step_id, label, detail) in STEPS {
        push_event(
            &mut events,
            run_id,
            &mut sequence,
            ReleaseRunEvent::StepStarted {
                step_id: step_id.to_string(),
            },
        );
        push_event(
            &mut events,
            run_id,
            &mut sequence,
            ReleaseRunEvent::LogAppend {
                message: format!("{label}: {detail}"),
                level: "info".to_string(),
            },
        );
        push_event(
            &mut events,
            run_id,
            &mut sequence,
            ReleaseRunEvent::StepCompleted {
                step_id: step_id.to_string(),
            },
        );
    }

    let generated_at = emitted_at_for_sequence(sequence + 1);
    let summary = RunSummary {
        run_id: run_id.to_string(),
        generated_at: generated_at.clone(),
        recommendation: "READY".to_string(),
        risk_status: "REVIEW".to_string(),
        review_flags: vec![
            "2 medium vulnerabilities (dev dependencies only)".to_string(),
            "2 minor dependency updates available".to_string(),
            "Benchmark variance within 5% of baseline (review threshold)".to_string(),
        ],
        hard_gates: vec![
            GateResult {
                name: "Completeness".to_string(),
                status: "pass".to_string(),
                detail: "All required release check categories present (10/10 signals)."
                    .to_string(),
            },
            GateResult {
                name: "Coverage".to_string(),
                status: "pass".to_string(),
                detail: "Unit 87%, integration 72%, all reported tests passing.".to_string(),
            },
            GateResult {
                name: "Security".to_string(),
                status: "review".to_string(),
                detail: "0 critical vulnerabilities; medium findings require human review."
                    .to_string(),
            },
            GateResult {
                name: "Dependencies".to_string(),
                status: "review".to_string(),
                detail:
                    "Pinned dependencies and no security patch blockers; minor updates available."
                        .to_string(),
            },
            GateResult {
                name: "Performance + Docs".to_string(),
                status: "pass".to_string(),
                detail: "No regressions or leaks detected; changelog and API docs present."
                    .to_string(),
            },
        ],
        highlights: vec![
            format!("Release candidate {candidate} is eligible for human signoff."),
            "Human approval is required before GO commitment.".to_string(),
            "Runtime-backed SSE is active for this pilot run.".to_string(),
        ],
        evidence_signal_ids: REQUIRED_SIGNAL_IDS
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    };

    push_event(
        &mut events,
        run_id,
        &mut sequence,
        ReleaseRunEvent::SummaryReady { summary },
    );
    push_event(
        &mut events,
        run_id,
        &mut sequence,
        ReleaseRunEvent::LogAppend {
            message: "Run complete. Recommendation READY with REVIEW risk posture.".to_string(),
            level: "info".to_string(),
        },
    );
    push_event(
        &mut events,
        run_id,
        &mut sequence,
        ReleaseRunEvent::RunCompleted {
            recommendation: "READY".to_string(),
        },
    );

    events
}

fn put_script(run_id: &str, script: Vec<RunEventEnvelope>) -> Result<(), RuntimeError> {
    let mut runs = RELEASE_READINESS_RUNS
        .write()
        .map_err(|_| RuntimeError::Config("Release pilot run store lock poisoned".to_string()))?;
    runs.insert(run_id.to_string(), script);
    Ok(())
}

fn get_or_create_script(
    run_id: &str,
    candidate: &str,
    environment: &str,
    release_window: &str,
) -> Result<Vec<RunEventEnvelope>, RuntimeError> {
    let mut runs = RELEASE_READINESS_RUNS
        .write()
        .map_err(|_| RuntimeError::Config("Release pilot run store lock poisoned".to_string()))?;

    if let Some(existing) = runs.get(run_id) {
        return Ok(existing.clone());
    }

    let script = build_release_readiness_script(run_id, candidate, environment, release_window);
    runs.insert(run_id.to_string(), script.clone());
    Ok(script)
}

pub async fn create_release_readiness_run(
    Json(request): Json<CreateRunRequest>,
) -> Result<Json<CreateRunResponse>, RuntimeError> {
    let run_id = format!("rr-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    let candidate = normalized_candidate(request.candidate);
    let environment = normalized_environment(request.environment);
    let release_window = normalized_release_window(request.release_window);

    if request
        .invite_session_id
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return Err(RuntimeError::Authentication(
            "invite_session_id is required.".to_string(),
        ));
    }
    if request
        .invite_code_id
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return Err(RuntimeError::Authentication(
            "invite_code_id is required.".to_string(),
        ));
    }

    if request.owner.as_deref().map(str::is_empty).unwrap_or(true) {
        warn!("create_release_readiness_run called without owner");
    }
    if request.notes.as_deref().map(str::is_empty).unwrap_or(true) {
        warn!("create_release_readiness_run called without notes");
    }

    let script = build_release_readiness_script(&run_id, &candidate, &environment, &release_window);
    put_script(&run_id, script)?;

    Ok(Json(CreateRunResponse {
        run_id: run_id.clone(),
        started_at: emitted_at_for_sequence(1),
        stream_path: format!("/api/pilot/release-readiness/runs/{run_id}/events"),
    }))
}

pub async fn stream_release_readiness_events(
    Path(run_id): Path<String>,
    Query(query): Query<StreamQuery>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, RuntimeError> {
    if query
        .invite_session_id
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return Err(RuntimeError::Authentication(
            "invite_session_id is required.".to_string(),
        ));
    }
    if query
        .invite_code_id
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return Err(RuntimeError::Authentication(
            "invite_code_id is required.".to_string(),
        ));
    }

    let candidate = normalized_candidate(query.candidate);
    let environment = normalized_environment(query.environment);
    let release_window = normalized_release_window(query.release_window);
    let since = query.since.unwrap_or(0);

    let events = get_or_create_script(&run_id, &candidate, &environment, &release_window)?
        .into_iter()
        .filter(|envelope| envelope.sequence > since)
        .collect::<Vec<_>>();

    let stream = async_stream::stream! {
        for envelope in events {
            match serde_json::to_string(&envelope) {
                Ok(payload) => {
                    yield Ok(Event::default().data(payload));
                }
                Err(error) => {
                    let fallback = RunEventEnvelope {
                        run_id: run_id.clone(),
                        sequence: envelope.sequence,
                        emitted_at: emitted_at_for_sequence(envelope.sequence),
                        event: ReleaseRunEvent::RunFailed {
                            message: format!("serialization failure: {error}"),
                        },
                    };
                    if let Ok(payload) = serde_json::to_string(&fallback) {
                        yield Ok(Event::default().data(payload));
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(180)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    ))
}

pub fn router() -> Router<()> {
    Router::new()
        .route(
            "/api/pilot/release-readiness/runs",
            post(create_release_readiness_run),
        )
        .route(
            "/api/pilot/release-readiness/runs/{run_id}/events",
            get(stream_release_readiness_events),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn build_script_has_expected_lifecycle() {
        let script = build_release_readiness_script(
            "rr-demo",
            "v1.4.0-rc.2",
            "production",
            DEFAULT_RELEASE_WINDOW,
        );
        assert!(script.len() > 5);
        assert_eq!(script.first().map(|e| e.sequence), Some(1));
        assert!(matches!(
            script.last().map(|e| &e.event),
            Some(ReleaseRunEvent::RunCompleted { .. })
        ));
    }

    #[tokio::test]
    async fn short_token_is_rejected_in_non_firebase_build() {
        let result = crate::http_auth::validate_token("short").await;
        #[cfg(not(feature = "firebase"))]
        assert!(result.is_err());
        #[cfg(feature = "firebase")]
        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn release_script_is_deterministic(
            run_id in "rr-[a-z0-9\\-]{3,16}",
            candidate in "[A-Za-z0-9\\.-]{1,24}"
        ) {
            let left = build_release_readiness_script(&run_id, &candidate, "production", DEFAULT_RELEASE_WINDOW);
            let right = build_release_readiness_script(&run_id, &candidate, "production", DEFAULT_RELEASE_WINDOW);
            prop_assert_eq!(left.len(), right.len());
            prop_assert!(left.iter().zip(&right).all(|(a, b)| a.sequence == b.sequence && a.emitted_at == b.emitted_at));
        }
    }
}
