// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Converge Remote - Runtime Driver
//!
//! This is the compatibility CLI that connects to `converge-runtime`.
//! The canonical Rust SDK now lives in `converge-client`, and the canonical
//! generated wire contract now lives in `converge-protocol`.
//!
//! # Architecture
//!
//! ```text
//! converge-remote ──gRPC──► converge-runtime
//!      │                          │
//!      │ Stream events            │ Run engine
//!      │ Send controls            │ Emit facts
//!      ▼                          ▼
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Run a job against the runtime
//! converge-remote run --template growth-strategy --server grpc://localhost:50051
//!
//! # Watch a running job
//! converge-remote watch <run_id> --server grpc://localhost:50051
//!
//! # Submit an observation to a running job
//! converge-remote observe --run-id <id> --observation '{"key": "Seeds", "payload": {"content": "..."}}'
//!
//! # Approve/reject a pending proposal
//! converge-remote approve <proposal_id>
//! converge-remote reject <proposal_id> --reason "..."
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use converge_client::{ConvergeClient, messages};
use converge_protocol::{
    prost_types,
    v1::{
        ApproveProposalRequest, Budget, CancelJobRequest, GetCapabilitiesRequest, GetJobRequest,
        PauseRunRequest, RejectProposalRequest, ResumeRunRequest, SeedFact, SubmitJobRequest,
        SubmitObservationRequest, SubscribeRequest, server_event,
    },
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[allow(dead_code)]
mod connection;
#[allow(dead_code)]
mod streaming;

/// Converge Remote - Runtime driver for distributed convergence
#[derive(Parser)]
#[command(name = "converge-remote")]
#[command(about = "Connect to converge-runtime for distributed convergence jobs")]
#[command(version)]
struct Cli {
    /// Runtime server address
    #[arg(
        long,
        short,
        env = "CONVERGE_SERVER",
        default_value = "http://localhost:50051"
    )]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a convergence job on the runtime
    Run {
        /// Blueprint/template to use
        #[arg(short, long)]
        template: String,

        /// Seeds as JSON (or @file.json)
        #[arg(short, long)]
        seeds: Option<String>,

        /// Max cycles budget
        #[arg(long, default_value = "50")]
        max_cycles: u32,

        /// Correlation ID to link related runs
        #[arg(long)]
        correlation_id: Option<String>,

        /// Use mock LLM for deterministic output
        #[arg(long)]
        mock: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Stream facts as they arrive
        #[arg(long)]
        stream: bool,

        /// Quiet mode: exit code only
        #[arg(long)]
        quiet: bool,
    },

    /// Watch a running job
    Watch {
        /// Run ID to watch
        run_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Submit an observation to a running job
    Observe {
        /// Run ID
        #[arg(long)]
        run_id: String,

        /// Observation as JSON
        #[arg(long)]
        observation: String,
    },

    /// Approve a pending proposal
    Approve {
        /// Proposal ID
        proposal_id: String,

        /// Run ID
        #[arg(long)]
        run_id: String,
    },

    /// Reject a pending proposal
    Reject {
        /// Proposal ID
        proposal_id: String,

        /// Run ID
        #[arg(long)]
        run_id: String,

        /// Reason for rejection
        #[arg(long)]
        reason: String,
    },

    /// Pause a running job
    Pause {
        /// Run ID
        run_id: String,
    },

    /// Resume a paused job
    Resume {
        /// Run ID
        run_id: String,
    },

    /// Cancel a job
    Cancel {
        /// Job ID
        job_id: String,

        /// Reason for cancellation
        #[arg(long)]
        reason: Option<String>,
    },

    /// Get job status
    Status {
        /// Job ID
        job_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Query server capabilities
    Capabilities,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Check if we should suppress tracing
    let suppress_tracing = matches!(&cli.command, Commands::Run { quiet: true, .. });

    if !suppress_tracing {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_target(false)
            .init();
    }

    let server = cli.server.clone();

    match cli.command {
        Commands::Run {
            template,
            seeds,
            max_cycles,
            correlation_id,
            mock,
            json,
            stream,
            quiet,
        } => {
            run_job(RunJobConfig {
                server: server.clone(),
                template,
                seeds,
                max_cycles,
                correlation_id,
                mock,
                json,
                stream,
                quiet,
            })
            .await?;
        }

        Commands::Watch { run_id, json } => {
            watch_job(&server, &run_id, json).await?;
        }

        Commands::Observe {
            run_id,
            observation,
        } => {
            submit_observation(&server, &run_id, &observation).await?;
        }

        Commands::Approve {
            proposal_id,
            run_id,
        } => {
            approve_proposal(&server, &run_id, &proposal_id).await?;
        }

        Commands::Reject {
            proposal_id,
            run_id,
            reason,
        } => {
            reject_proposal(&server, &run_id, &proposal_id, &reason).await?;
        }

        Commands::Pause { run_id } => {
            pause_run(&server, &run_id).await?;
        }

        Commands::Resume { run_id } => {
            resume_run(&server, &run_id).await?;
        }

        Commands::Cancel { job_id, reason } => {
            cancel_job(&server, &job_id, reason.as_deref()).await?;
        }

        Commands::Status { job_id, json } => {
            get_status(&server, &job_id, json).await?;
        }

        Commands::Capabilities => {
            get_capabilities(&server).await?;
        }
    }

    Ok(())
}

/// Configuration for running a job
#[allow(clippy::struct_excessive_bools)]
struct RunJobConfig {
    server: String,
    template: String,
    seeds: Option<String>,
    max_cycles: u32,
    correlation_id: Option<String>,
    #[allow(dead_code)]
    mock: bool,
    json: bool,
    stream: bool,
    quiet: bool,
}

async fn run_job(config: RunJobConfig) -> Result<()> {
    if !config.quiet {
        info!(server = %config.server, template = %config.template, "Connecting to runtime");
    }

    let mut client = ConvergeClient::connect(config.server.clone()).await?;

    // Parse seeds into SeedFacts
    let seed_list: Vec<SeedFact> = if let Some(seeds_raw) = config.seeds {
        let seeds_json = if let Some(path) = seeds_raw.strip_prefix('@') {
            std::fs::read_to_string(path)?
        } else {
            seeds_raw
        };

        let parsed: Vec<serde_json::Value> = serde_json::from_str(&seeds_json)?;
        parsed
            .into_iter()
            .map(|v| {
                let value = prost_types::Struct {
                    fields: v
                        .as_object()
                        .map(|obj| {
                            obj.iter()
                                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                                .collect()
                        })
                        .unwrap_or_default(),
                };
                SeedFact {
                    key: v["key"].as_str().unwrap_or("").to_string(),
                    value: Some(value),
                    truth_id: v["truth_id"].as_str().map(String::from),
                }
            })
            .collect()
    } else {
        vec![]
    };

    // Generate idempotency key
    let idempotency_key = format!(
        "cli:{}:run:{}:{}",
        hostname::get().map_or_else(
            |_| "unknown".to_string(),
            |h| h.to_string_lossy().to_string()
        ),
        chrono::Utc::now().timestamp_millis(),
        &uuid::Uuid::new_v4().to_string()[..4]
    );

    // Create job via unary RPC
    let submit_request = SubmitJobRequest {
        idempotency_key,
        blueprint_id: config.template.clone(),
        pack_ids: vec![],
        seeds: seed_list,
        budget: Some(Budget {
            max_cycles: Some(config.max_cycles),
            max_facts: None,
            max_duration_ms: Some(300_000),
            max_tokens: None,
            max_cost_usd: None,
        }),
        correlation_id: config.correlation_id.clone(),
        parent_trace_id: None,
    };

    let response = client.submit_job(submit_request).await?;
    let job_id = response.job_id;
    let run_id = response.run_id;

    if !config.quiet && !config.json {
        info!(job_id = %job_id, run_id = %run_id, "Job created");
    }

    if config.stream || !config.quiet {
        // Use bidirectional streaming to watch events
        let (tx, rx) = mpsc::channel(32);

        // Send subscribe request
        let subscribe_msg = messages::subscribe(
            format!("sub_{}", uuid::Uuid::new_v4()),
            SubscribeRequest {
                job_id: Some(job_id.clone()),
                run_id: Some(run_id.clone()),
                correlation_id: config.correlation_id.clone(),
                since_sequence: 0,
                entry_types: vec![],
            },
        );
        tx.send(subscribe_msg).await?;

        let outbound = ReceiverStream::new(rx);
        let mut inbound = client.stream(outbound).await?;

        // Process incoming events
        while let Some(event) = inbound.message().await? {
            if let Some(e) = &event.event {
                match e {
                    server_event::Event::Entry(entry) => {
                        if config.json {
                            let output = serde_json::json!({
                                "sequence": event.sequence,
                                "type": format!("{:?}", entry.entry_type),
                                "entry_id": entry.entry_id,
                                "run_id": entry.run_id,
                            });
                            println!("{}", serde_json::to_string(&output)?);
                        } else if !config.quiet {
                            println!(
                                "[seq:{}] {:?} | {}",
                                event.sequence, entry.entry_type, entry.entry_id
                            );
                        }
                    }
                    server_event::Event::RunStatus(status) => {
                        if !config.quiet {
                            println!(
                                "[{}] status={:?} facts={} cycles={}",
                                status.run_id,
                                status.status,
                                status.facts_count,
                                status.cycles_completed
                            );
                        }
                        // Check for terminal state
                        let status_code = status.status;
                        if status_code == 3 || status_code == 4 || status_code == 6 {
                            // CONVERGED, HALTED, or CANCELLED
                            break;
                        }
                    }
                    server_event::Event::JobCompleted(completed) => {
                        if !config.quiet {
                            println!(
                                "Job {} completed: {:?} ({} facts, {} cycles, {}ms)",
                                completed.job_id,
                                completed.final_status,
                                completed.total_facts,
                                completed.total_cycles,
                                completed.duration_ms
                            );
                        }
                        break;
                    }
                    server_event::Event::Error(err) => {
                        warn!("Error from server: {} - {}", err.code, err.message);
                        if !err.recoverable {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if config.quiet {
        // Get final status for exit code
        let status_response = client.get_job(GetJobRequest { job_id }).await?;

        let exit_code = match status_response.status {
            3 => 0, // CONVERGED
            4 => 1, // HALTED
            _ => 2, // RUNNING/WAITING = budget likely exceeded
        };
        std::process::exit(exit_code);
    }

    Ok(())
}

async fn watch_job(server: &str, run_id: &str, json: bool) -> Result<()> {
    info!(server = %server, run_id = %run_id, "Watching job");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    // Use bidirectional streaming
    let (tx, rx) = mpsc::channel(32);

    // Send subscribe request
    let subscribe_msg = messages::subscribe(
        format!("watch_{}", uuid::Uuid::new_v4()),
        SubscribeRequest {
            job_id: None,
            run_id: Some(run_id.to_string()),
            correlation_id: None,
            since_sequence: 0,
            entry_types: vec![],
        },
    );
    tx.send(subscribe_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    while let Some(event) = inbound.message().await? {
        if let Some(e) = &event.event {
            match e {
                server_event::Event::Entry(entry) => {
                    if json {
                        let output = serde_json::json!({
                            "sequence": event.sequence,
                            "type": format!("{:?}", entry.entry_type),
                            "entry_id": entry.entry_id,
                        });
                        println!("{}", serde_json::to_string(&output)?);
                    } else {
                        println!(
                            "[seq:{}] {:?} | {}",
                            event.sequence, entry.entry_type, entry.entry_id
                        );
                    }
                }
                server_event::Event::RunStatus(status) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "run_id": status.run_id,
                                "status": status.status,
                                "facts_count": status.facts_count,
                                "cycles": status.cycles_completed,
                            }))?
                        );
                    } else {
                        println!(
                            "[{}] status={:?} facts={} cycles={}",
                            status.run_id,
                            status.status,
                            status.facts_count,
                            status.cycles_completed
                        );
                    }

                    // Exit if terminal state
                    let status_code = status.status;
                    if status_code == 3 || status_code == 4 || status_code == 6 {
                        break;
                    }
                }
                server_event::Event::JobCompleted(_) => break,
                server_event::Event::Error(err) => {
                    warn!("Error: {} - {}", err.code, err.message);
                    if !err.recoverable {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

async fn submit_observation(server: &str, run_id: &str, observation_json: &str) -> Result<()> {
    info!(server = %server, run_id = %run_id, "Submitting observation");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let observation_value: serde_json::Value = serde_json::from_str(observation_json)?;
    let key = observation_value
        .get("key")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("observation JSON must include string field 'key'"))?
        .to_string();
    let payload = observation_payload(&observation_value)?;
    let target_truth_id = observation_value
        .get("target_truth_id")
        .and_then(serde_json::Value::as_str)
        .map(String::from);

    // Use bidirectional streaming to send observation request
    let (tx, rx) = mpsc::channel(32);

    let observation_msg = messages::submit_observation(
        format!("observe_{}", uuid::Uuid::new_v4()),
        SubmitObservationRequest {
            run_id: run_id.to_string(),
            key,
            payload: Some(json_to_prost_struct(&payload)),
            target_truth_id,
            idempotency_key: format!("observe_{}", uuid::Uuid::new_v4()),
        },
    );
    tx.send(observation_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    // Wait for ack
    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Observation submitted successfully");
            } else {
                warn!(
                    "Failed to submit observation: {}",
                    ack.error_message.unwrap_or_default()
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn observation_payload(observation: &serde_json::Value) -> Result<serde_json::Value> {
    if let Some(payload) = observation.get("payload") {
        return Ok(payload.clone());
    }

    let object = observation
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("observation JSON must be an object"))?;
    let mut payload = serde_json::Map::new();
    for (key, value) in object {
        if matches!(key.as_str(), "key" | "target_truth_id" | "idempotency_key") {
            continue;
        }
        payload.insert(key.clone(), value.clone());
    }

    if payload.is_empty() {
        return Err(anyhow::anyhow!(
            "observation JSON must include 'payload' or additional payload fields"
        ));
    }

    Ok(serde_json::Value::Object(payload))
}

async fn approve_proposal(server: &str, run_id: &str, proposal_id: &str) -> Result<()> {
    info!(server = %server, run_id = %run_id, proposal_id = %proposal_id, "Approving proposal");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let (tx, rx) = mpsc::channel(32);

    let approve_msg = messages::approve(
        format!("approve_{}", uuid::Uuid::new_v4()),
        ApproveProposalRequest {
            run_id: run_id.to_string(),
            proposal_id: proposal_id.to_string(),
            comment: None,
        },
    );
    tx.send(approve_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Proposal approved");
            } else {
                warn!(
                    "Failed to approve: {}",
                    ack.error_message.unwrap_or_default()
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn reject_proposal(
    server: &str,
    run_id: &str,
    proposal_id: &str,
    reason: &str,
) -> Result<()> {
    info!(server = %server, run_id = %run_id, proposal_id = %proposal_id, "Rejecting proposal");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let (tx, rx) = mpsc::channel(32);

    let reject_msg = messages::reject(
        format!("reject_{}", uuid::Uuid::new_v4()),
        RejectProposalRequest {
            run_id: run_id.to_string(),
            proposal_id: proposal_id.to_string(),
            reason: reason.to_string(),
        },
    );
    tx.send(reject_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Proposal rejected");
            } else {
                warn!(
                    "Failed to reject: {}",
                    ack.error_message.unwrap_or_default()
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn pause_run(server: &str, run_id: &str) -> Result<()> {
    info!(server = %server, run_id = %run_id, "Pausing run");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let (tx, rx) = mpsc::channel(32);

    let pause_msg = messages::pause(
        format!("pause_{}", uuid::Uuid::new_v4()),
        PauseRunRequest {
            run_id: run_id.to_string(),
            reason: None,
        },
    );
    tx.send(pause_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Run paused");
            } else {
                warn!("Failed to pause: {}", ack.error_message.unwrap_or_default());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn resume_run(server: &str, run_id: &str) -> Result<()> {
    info!(server = %server, run_id = %run_id, "Resuming run");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let (tx, rx) = mpsc::channel(32);

    let resume_msg = messages::resume(
        format!("resume_{}", uuid::Uuid::new_v4()),
        ResumeRunRequest {
            run_id: run_id.to_string(),
        },
    );
    tx.send(resume_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Run resumed");
            } else {
                warn!(
                    "Failed to resume: {}",
                    ack.error_message.unwrap_or_default()
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn cancel_job(server: &str, job_id: &str, reason: Option<&str>) -> Result<()> {
    info!(server = %server, job_id = %job_id, "Cancelling job");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let (tx, rx) = mpsc::channel(32);

    let cancel_msg = messages::cancel_job(
        format!("cancel_{}", uuid::Uuid::new_v4()),
        CancelJobRequest {
            job_id: job_id.to_string(),
            reason: reason.map(String::from),
        },
    );
    tx.send(cancel_msg).await?;

    let outbound = ReceiverStream::new(rx);
    let mut inbound = client.stream(outbound).await?;

    if let Some(event) = inbound.message().await? {
        if let Some(server_event::Event::Ack(ack)) = event.event {
            if ack.success {
                println!("Job cancelled");
            } else {
                warn!("Cancel failed: {}", ack.error_message.unwrap_or_default());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn get_status(server: &str, job_id: &str, json: bool) -> Result<()> {
    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let response = client
        .get_job(GetJobRequest {
            job_id: job_id.to_string(),
        })
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "job_id": response.job_id,
                "run_id": response.run_id,
                "status": response.status,
                "facts_count": response.facts_count,
                "cycles_completed": response.cycles_completed,
                "pending_proposals": response.pending_proposals,
            }))?
        );
    } else {
        println!("Job: {}", response.job_id);
        println!("Run: {}", response.run_id);
        println!("Status: {:?}", response.status);
        println!("Facts: {}", response.facts_count);
        println!("Cycles: {}", response.cycles_completed);
        println!("Pending proposals: {}", response.pending_proposals);
        if let Some(halt_info) = response.halt_info {
            println!("Halt reason: {}", halt_info.reason);
            println!("Halted by truth: {}", halt_info.truth_id);
        }
        if let Some(converged_info) = response.converged_info {
            println!("Duration: {}ms", converged_info.duration_ms);
        }
    }

    Ok(())
}

async fn get_capabilities(server: &str) -> Result<()> {
    info!(server = %server, "Querying capabilities");

    let mut client = ConvergeClient::connect(server.to_string()).await?;

    let hostname = hostname::get().map_or_else(
        |_| "unknown".to_string(),
        |h| h.to_string_lossy().to_string(),
    );
    let device_id = format!("cli-remote:{hostname}");

    let request = GetCapabilitiesRequest {
        device_id,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: "cli".to_string(),
    };

    let response = client.get_capabilities(request).await?;

    println!("Converge Remote v{}\n", env!("CARGO_PKG_VERSION"));
    println!("Server: {server}");
    println!("Server version: {}\n", response.server_version);

    println!("Packs:");
    for pack in &response.packs {
        println!("  - {} v{} ({})", pack.name, pack.version, pack.pack_id);
        for truth_id in &pack.truth_ids {
            println!("      truth: {truth_id}");
        }
    }

    println!("\nActive Truths:");
    for truth in &response.active_truths {
        println!("  - {} (pack: {})", truth.truth_id, truth.pack_id);
        println!("    {}", truth.description);
    }

    if let Some(streaming) = &response.streaming {
        println!("\nTransports:");
        for transport in &streaming.transports {
            let t_type = &transport.r#type;
            let t_status = &transport.status;
            println!("  - {t_type} ({t_status})");
        }
        println!("\nStreaming:");
        println!("  Default transport: {}", streaming.default_transport);
        println!("  Resume supported: {}", streaming.resume_supported);
        println!("  Max resume gap: {}", streaming.max_resume_gap);
    }

    println!("\nFeatures:");
    println!(
        "  Determinism mode: {}",
        response.determinism_mode_available
    );

    Ok(())
}

/// Convert a JSON value to `prost_types::Value`
fn json_to_prost_value(v: &serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;

    let kind = match v {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(obj) => Kind::StructValue(prost_types::Struct {
            fields: obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect(),
        }),
    };

    prost_types::Value { kind: Some(kind) }
}

/// Convert a JSON value to `prost_types::Struct`
fn json_to_prost_struct(v: &serde_json::Value) -> prost_types::Struct {
    if let serde_json::Value::Object(obj) = v {
        prost_types::Struct {
            fields: obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect(),
        }
    } else {
        prost_types::Struct::default()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // JSON to Prost Value Conversion Tests
    // =========================================================================

    #[test]
    fn json_null_converts_to_null_value() {
        let json = json!(null);
        let prost = json_to_prost_value(&json);
        assert!(matches!(
            prost.kind,
            Some(prost_types::value::Kind::NullValue(0))
        ));
    }

    #[test]
    fn json_bool_true_converts_correctly() {
        let json = json!(true);
        let prost = json_to_prost_value(&json);
        assert!(matches!(
            prost.kind,
            Some(prost_types::value::Kind::BoolValue(true))
        ));
    }

    #[test]
    fn json_bool_false_converts_correctly() {
        let json = json!(false);
        let prost = json_to_prost_value(&json);
        assert!(matches!(
            prost.kind,
            Some(prost_types::value::Kind::BoolValue(false))
        ));
    }

    #[test]
    fn json_integer_converts_to_number() {
        let json = json!(42);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::NumberValue(n)) = prost.kind {
            assert!((n - 42.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected NumberValue");
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn json_float_converts_to_number() {
        let json = json!(3.14159);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::NumberValue(n)) = prost.kind {
            assert!((n - 3.14159).abs() < 0.00001);
        } else {
            panic!("Expected NumberValue");
        }
    }

    #[test]
    fn json_negative_number_converts_correctly() {
        let json = json!(-999);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::NumberValue(n)) = prost.kind {
            assert!((n - (-999.0)).abs() < f64::EPSILON);
        } else {
            panic!("Expected NumberValue");
        }
    }

    #[test]
    fn json_string_converts_correctly() {
        let json = json!("hello world");
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StringValue(s)) = prost.kind {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected StringValue");
        }
    }

    #[test]
    fn json_empty_string_converts_correctly() {
        let json = json!("");
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StringValue(s)) = prost.kind {
            assert_eq!(s, "");
        } else {
            panic!("Expected StringValue");
        }
    }

    #[test]
    fn json_unicode_string_converts_correctly() {
        let json = json!("Hello 世界 🌍");
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StringValue(s)) = prost.kind {
            assert_eq!(s, "Hello 世界 🌍");
        } else {
            panic!("Expected StringValue");
        }
    }

    #[test]
    fn json_empty_array_converts_to_list() {
        let json = json!([]);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::ListValue(list)) = prost.kind {
            assert!(list.values.is_empty());
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn json_array_of_numbers_converts_correctly() {
        let json = json!([1, 2, 3]);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::ListValue(list)) = prost.kind {
            assert_eq!(list.values.len(), 3);
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn json_mixed_array_converts_correctly() {
        let json = json!([1, "two", true, null]);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::ListValue(list)) = prost.kind {
            assert_eq!(list.values.len(), 4);
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn json_empty_object_converts_to_struct() {
        let json = json!({});
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StructValue(s)) = prost.kind {
            assert!(s.fields.is_empty());
        } else {
            panic!("Expected StructValue");
        }
    }

    #[test]
    fn json_object_converts_to_struct() {
        let json = json!({"name": "test", "value": 42});
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StructValue(s)) = prost.kind {
            assert_eq!(s.fields.len(), 2);
            assert!(s.fields.contains_key("name"));
            assert!(s.fields.contains_key("value"));
        } else {
            panic!("Expected StructValue");
        }
    }

    #[test]
    fn json_nested_object_converts_correctly() {
        let json = json!({
            "outer": {
                "inner": {
                    "value": 123
                }
            }
        });
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::StructValue(s)) = prost.kind {
            assert!(s.fields.contains_key("outer"));
        } else {
            panic!("Expected StructValue");
        }
    }

    // =========================================================================
    // JSON to Prost Struct Conversion Tests
    // =========================================================================

    #[test]
    fn json_object_to_struct_basic() {
        let json = json!({"key": "value"});
        let prost = json_to_prost_struct(&json);
        assert_eq!(prost.fields.len(), 1);
        assert!(prost.fields.contains_key("key"));
    }

    #[test]
    fn json_non_object_to_struct_returns_empty() {
        let json = json!("not an object");
        let prost = json_to_prost_struct(&json);
        assert!(prost.fields.is_empty());
    }

    #[test]
    fn json_array_to_struct_returns_empty() {
        let json = json!([1, 2, 3]);
        let prost = json_to_prost_struct(&json);
        assert!(prost.fields.is_empty());
    }

    #[test]
    fn json_null_to_struct_returns_empty() {
        let json = json!(null);
        let prost = json_to_prost_struct(&json);
        assert!(prost.fields.is_empty());
    }

    #[test]
    fn json_complex_object_to_struct() {
        let json = json!({
            "string": "value",
            "number": 42,
            "bool": true,
            "null": null,
            "array": [1, 2, 3],
            "nested": {"a": 1}
        });
        let prost = json_to_prost_struct(&json);
        assert_eq!(prost.fields.len(), 6);
    }

    // =========================================================================
    // Edge Cases and Boundary Tests
    // =========================================================================

    #[test]
    fn json_very_large_number() {
        let json = json!(1e308);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::NumberValue(n)) = prost.kind {
            assert!(n.is_finite());
        } else {
            panic!("Expected NumberValue");
        }
    }

    #[test]
    fn json_very_small_number() {
        let json = json!(1e-308);
        let prost = json_to_prost_value(&json);
        if let Some(prost_types::value::Kind::NumberValue(n)) = prost.kind {
            assert!(n > 0.0);
        } else {
            panic!("Expected NumberValue");
        }
    }

    #[test]
    fn json_deeply_nested_structure() {
        let json = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "l4": {
                            "l5": "deep"
                        }
                    }
                }
            }
        });
        let prost = json_to_prost_struct(&json);
        assert!(prost.fields.contains_key("l1"));
    }

    #[test]
    fn json_special_characters_in_keys() {
        let json = json!({
            "key with spaces": 1,
            "key-with-dashes": 2,
            "key.with.dots": 3,
            "key:with:colons": 4
        });
        let prost = json_to_prost_struct(&json);
        assert_eq!(prost.fields.len(), 4);
    }

    #[test]
    fn json_empty_key() {
        let json = json!({"": "empty key"});
        let prost = json_to_prost_struct(&json);
        assert!(prost.fields.contains_key(""));
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prost_value_always_has_kind(s in ".*") {
            let json = serde_json::Value::String(s);
            let prost = json_to_prost_value(&json);
            prop_assert!(prost.kind.is_some());
        }

        #[test]
        fn string_roundtrip_preserves_content(s in ".*") {
            let json = serde_json::Value::String(s.clone());
            let prost = json_to_prost_value(&json);
            if let Some(prost_types::value::Kind::StringValue(result)) = prost.kind {
                prop_assert_eq!(result, s);
            } else {
                prop_assert!(false, "Expected StringValue");
            }
        }

        #[test]
        fn number_converts_to_finite_f64(n in -1e100f64..1e100f64) {
            let json = serde_json::json!(n);
            let prost = json_to_prost_value(&json);
            if let Some(prost_types::value::Kind::NumberValue(result)) = prost.kind {
                prop_assert!(result.is_finite());
            } else {
                prop_assert!(false, "Expected NumberValue");
            }
        }

        #[test]
        fn integer_converts_correctly(n in i32::MIN..i32::MAX) {
            let json = serde_json::json!(n);
            let prost = json_to_prost_value(&json);
            if let Some(prost_types::value::Kind::NumberValue(result)) = prost.kind {
                prop_assert!((result - f64::from(n)).abs() < 1.0);
            } else {
                prop_assert!(false, "Expected NumberValue");
            }
        }

        #[test]
        fn array_length_preserved(len in 0usize..100) {
            let arr: Vec<serde_json::Value> = (0..len).map(|i| serde_json::json!(i)).collect();
            let json = serde_json::Value::Array(arr);
            let prost = json_to_prost_value(&json);
            if let Some(prost_types::value::Kind::ListValue(list)) = prost.kind {
                prop_assert_eq!(list.values.len(), len);
            } else {
                prop_assert!(false, "Expected ListValue");
            }
        }

        #[test]
        fn object_field_count_preserved(num_fields in 0usize..50) {
            let mut obj = serde_json::Map::new();
            for i in 0..num_fields {
                obj.insert(format!("field_{i}"), serde_json::json!(i));
            }
            let json = serde_json::Value::Object(obj);
            let prost = json_to_prost_struct(&json);
            prop_assert_eq!(prost.fields.len(), num_fields);
        }

        #[test]
        fn bool_converts_correctly(b in proptest::bool::ANY) {
            let json = serde_json::json!(b);
            let prost = json_to_prost_value(&json);
            if let Some(prost_types::value::Kind::BoolValue(result)) = prost.kind {
                prop_assert_eq!(result, b);
            } else {
                prop_assert!(false, "Expected BoolValue");
            }
        }

        #[test]
        fn non_object_to_struct_is_empty(s in ".*") {
            let json = serde_json::Value::String(s);
            let prost = json_to_prost_struct(&json);
            prop_assert!(prost.fields.is_empty());
        }
    }
}
