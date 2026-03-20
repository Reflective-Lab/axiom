// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: LicenseRef-Proprietary

//! CLI definition for cz - the Converge Zone workspace orchestrator.

use clap::{Parser, Subcommand};

/// cz - Converge Zone CLI
///
/// The workspace orchestrator that bootstraps the environment, runs builds and
/// tests, manages services, and enforces governance (gates, policies, audit trails).
#[derive(Parser, Debug)]
#[command(name = "cz")]
#[command(author = "Aprio One AB")]
#[command(version)]
#[command(about = "Converge Zone - workspace orchestrator", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // =========================================================================
    // Bootstrap & Diagnostics
    // =========================================================================
    /// Check development environment health
    ///
    /// Verifies that all required tools are installed and configured correctly.
    /// Prints copy-pasteable fix commands for any issues found.
    Doctor(DoctorArgs),

    /// Set up development environment
    ///
    /// Installs dependencies, creates .env from template, and validates
    /// that required binaries exist.
    Bootstrap(BootstrapArgs),

    // =========================================================================
    // Development Workflow
    // =========================================================================
    /// Run tests across all components
    Test(DevArgs),

    /// Format code across all components
    Fmt(DevArgs),

    /// Run linters across all components
    Lint(DevArgs),

    /// Run full CI pipeline locally
    ///
    /// Executes the same checks that run in GitHub Actions.
    Ci(DevArgs),

    // =========================================================================
    // Service Orchestration
    // =========================================================================
    /// Start services (Docker Compose)
    Up(ServiceArgs),

    /// Stop services
    Down(ServiceArgs),

    /// View service logs
    Logs(LogsArgs),

    // =========================================================================
    // Governance Workflows
    // =========================================================================
    /// Validate specs and produce findings
    ///
    /// Runs validation checks on .truth/.feature files and produces findings
    /// in .converge/findings/. Use --enforce to apply Cedar policies.
    Validate(ValidateArgs),

    /// Produce "what needs attention" summary
    ///
    /// Summarizes open findings, pending acknowledgements, and escalations.
    Digest(DigestArgs),

    /// Acknowledge a finding
    ///
    /// Creates an append-only audit artifact recording the acknowledgement.
    Ack(AckArgs),

    /// Escalate a finding
    ///
    /// Creates an escalation record and optionally notifies the target team.
    Escalate(EscalateArgs),

    /// Assign a finding to an owner
    Assign(AssignArgs),
}

// =============================================================================
// Argument Structs
// =============================================================================

#[derive(Parser, Debug)]
pub struct DoctorArgs {
    /// Only check specific component (e.g., "runtime", "www", "ledger")
    #[arg(short, long)]
    pub component: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Show verbose output including all checks
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct BootstrapArgs {
    /// Skip dependency installation
    #[arg(long)]
    pub skip_deps: bool,

    /// Skip .env creation
    #[arg(long)]
    pub skip_env: bool,

    /// Force overwrite existing .env
    #[arg(long)]
    pub force: bool,

    /// Use mise for toolchain management if available
    #[arg(long)]
    pub mise: bool,
}

#[derive(Parser, Debug)]
pub struct DevArgs {
    /// Only run for specific component
    #[arg(short, long)]
    pub component: Option<String>,

    /// Continue on error (don't fail fast)
    #[arg(long)]
    pub keep_going: bool,
}

#[derive(Parser, Debug)]
pub struct ServiceArgs {
    /// Docker Compose profile to use
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Run in detached mode
    #[arg(short, long)]
    pub detach: bool,
}

#[derive(Parser, Debug)]
pub struct LogsArgs {
    /// Service name to show logs for
    pub service: Option<String>,

    /// Follow log output
    #[arg(short, long)]
    pub follow: bool,

    /// Number of lines to show
    #[arg(short = 'n', long, default_value = "100")]
    pub tail: u32,
}

#[derive(Parser, Debug)]
pub struct ValidateArgs {
    /// Files or directories to validate
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// Only check conventions (no LLM calls, fast)
    #[arg(long)]
    pub conventions_only: bool,

    /// Skip business sense validation (requires LLM)
    #[arg(long)]
    pub skip_business_sense: bool,

    /// Skip compilability validation (requires LLM)
    #[arg(long)]
    pub skip_compilability: bool,

    /// Enforce Cedar policies (block invalid states)
    #[arg(long)]
    pub enforce: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Write findings to .converge/findings/ (default: true)
    #[arg(long, default_value = "true")]
    pub persist: bool,
}

#[derive(Parser, Debug)]
pub struct DigestArgs {
    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Include resolved/acknowledged items
    #[arg(long)]
    pub include_resolved: bool,
}

#[derive(Parser, Debug)]
pub struct AckArgs {
    /// Finding ID to acknowledge
    pub finding_id: String,

    /// Note explaining the acknowledgement
    #[arg(short, long)]
    pub note: Option<String>,
}

#[derive(Parser, Debug)]
pub struct EscalateArgs {
    /// Finding ID to escalate
    pub finding_id: String,

    /// Team or person to escalate to
    #[arg(short, long)]
    pub to: Option<String>,

    /// Note explaining the escalation
    #[arg(short, long)]
    pub note: Option<String>,
}

#[derive(Parser, Debug)]
pub struct AssignArgs {
    /// Finding ID to assign
    pub finding_id: String,

    /// Owner ID to assign to
    #[arg(short, long)]
    pub owner: String,

    /// Note explaining the assignment
    #[arg(short, long)]
    pub note: Option<String>,
}

// =============================================================================
// Shared Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable colored output
    Pretty,
    /// JSON output for scripting
    Json,
    /// Minimal output (exit codes only)
    Quiet,
}
