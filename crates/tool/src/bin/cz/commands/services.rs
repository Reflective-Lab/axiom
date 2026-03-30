// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Service orchestration commands: up, down, logs.
//!
//! These shell out to Docker Compose in converge-runtime.

use crate::cli::{LogsArgs, ServiceArgs};
use crate::commands::{CmdError, CmdResult, find_workspace_root};
use colored::Colorize;
use std::process::Command;

/// Start services.
pub async fn up(args: ServiceArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    let runtime_dir = root.join("converge-runtime");
    if !runtime_dir.join("docker-compose.yml").exists() {
        return Err(CmdError::new(
            "No docker-compose.yml found in converge-runtime",
        ));
    }

    println!();
    println!("{}", "cz up".bright_blue().bold());
    println!();

    std::env::set_current_dir(&runtime_dir)
        .map_err(|e| CmdError::new(format!("Failed to change directory: {e}")))?;

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "up"]);

    if args.detach {
        cmd.arg("-d");
    }

    if let Some(profile) = &args.profile {
        cmd.args(["--profile", profile]);
    }

    let status = cmd
        .status()
        .map_err(|e| CmdError::new(format!("Failed to run docker compose: {e}")))?;

    std::env::set_current_dir(&root).ok();

    if !status.success() {
        return Err(CmdError::new("docker compose up failed"));
    }

    if args.detach {
        println!();
        println!("{}", "Services started!".green().bold());
        println!();
        print_endpoints();
    }

    Ok(())
}

/// Stop services.
pub async fn down(args: ServiceArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    let runtime_dir = root.join("converge-runtime");
    if !runtime_dir.join("docker-compose.yml").exists() {
        return Err(CmdError::new(
            "No docker-compose.yml found in converge-runtime",
        ));
    }

    println!();
    println!("{}", "cz down".bright_blue().bold());
    println!();

    std::env::set_current_dir(&runtime_dir)
        .map_err(|e| CmdError::new(format!("Failed to change directory: {e}")))?;

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "down"]);

    if let Some(profile) = &args.profile {
        cmd.args(["--profile", profile]);
    }

    let status = cmd
        .status()
        .map_err(|e| CmdError::new(format!("Failed to run docker compose: {e}")))?;

    std::env::set_current_dir(&root).ok();

    if !status.success() {
        return Err(CmdError::new("docker compose down failed"));
    }

    println!();
    println!("{}", "Services stopped.".green());
    println!();

    Ok(())
}

/// View service logs.
pub async fn logs(args: LogsArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    let runtime_dir = root.join("converge-runtime");
    if !runtime_dir.join("docker-compose.yml").exists() {
        return Err(CmdError::new(
            "No docker-compose.yml found in converge-runtime",
        ));
    }

    std::env::set_current_dir(&runtime_dir)
        .map_err(|e| CmdError::new(format!("Failed to change directory: {e}")))?;

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "logs"]);

    if args.follow {
        cmd.arg("-f");
    }

    cmd.args(["--tail", &args.tail.to_string()]);

    if let Some(service) = &args.service {
        cmd.arg(service);
    }

    let status = cmd
        .status()
        .map_err(|e| CmdError::new(format!("Failed to run docker compose logs: {e}")))?;

    std::env::set_current_dir(&root).ok();

    if !status.success() {
        return Err(CmdError::new("docker compose logs failed"));
    }

    Ok(())
}

fn print_endpoints() {
    println!("Endpoints:");
    println!("  {} http://localhost:3000", "API:".cyan());
    println!("  {} http://localhost:4000", "Firebase UI:".cyan());
    println!("  {} http://localhost:8080", "Firestore:".cyan());
    println!();
    println!("Use {} to view logs", "cz logs -f".cyan());
}
