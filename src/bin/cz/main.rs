// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
//! cz - Converge Zone CLI
//!
//! The workspace orchestrator that bootstraps, builds, and enforces governance.
//!
//! # Quick Start
//!
//! ```bash
//! cz doctor     # Check environment health
//! cz bootstrap  # Set up development environment
//! cz test       # Run all tests
//! cz up         # Start services
//! ```

mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Doctor(args) => commands::doctor::run(args).await,
        Commands::Bootstrap(args) => commands::bootstrap::run(args).await,
        Commands::Test(args) => commands::dev::test(args).await,
        Commands::Fmt(args) => commands::dev::fmt(args).await,
        Commands::Lint(args) => commands::dev::lint(args).await,
        Commands::Ci(args) => commands::dev::ci(args).await,
        Commands::Up(args) => commands::services::up(args).await,
        Commands::Down(args) => commands::services::down(args).await,
        Commands::Logs(args) => commands::services::logs(args).await,
        Commands::Validate(args) => commands::governance::validate(args).await,
        Commands::Digest(args) => commands::governance::digest(args).await,
        Commands::Ack(args) => commands::governance::ack(args).await,
        Commands::Escalate(args) => commands::governance::escalate(args).await,
        Commands::Assign(args) => commands::governance::assign(args).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
