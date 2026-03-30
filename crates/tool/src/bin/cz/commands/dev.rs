// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Development workflow commands: test, fmt, lint, ci.
//!
//! These shell out to `just` recipes in the workspace.

use crate::cli::DevArgs;
use crate::commands::{CmdError, CmdResult, find_workspace_root};
use colored::Colorize;
use std::process::Command;

/// Run tests across components.
pub async fn test(args: DevArgs) -> CmdResult {
    run_just_command("test", &args)
}

/// Format code across components.
pub async fn fmt(args: DevArgs) -> CmdResult {
    run_just_command("fmt", &args)
}

/// Run linters across components.
pub async fn lint(args: DevArgs) -> CmdResult {
    // Try clippy for Rust, lint for others
    run_just_command("lint", &args).or_else(|_| run_just_command("clippy", &args))
}

/// Run full CI pipeline.
pub async fn ci(args: DevArgs) -> CmdResult {
    run_just_command("ci", &args)
}

fn run_just_command(recipe: &str, args: &DevArgs) -> CmdResult {
    let root = find_workspace_root()
        .ok_or_else(|| CmdError::new("Could not find workspace root (no justfile found)"))?;

    println!();
    println!(
        "{} {}",
        "cz".bright_blue().bold(),
        recipe.bright_blue().bold()
    );
    println!();

    if let Some(component) = &args.component {
        // Run for specific component
        let component_dir = root.join(format!("converge-{component}"));
        if !component_dir.exists() {
            return Err(CmdError::new(format!(
                "Component not found: converge-{component}"
            )));
        }

        let has_justfile =
            component_dir.join("justfile").exists() || component_dir.join("Justfile").exists();

        if !has_justfile {
            return Err(CmdError::new(format!(
                "No justfile in converge-{component}"
            )));
        }

        println!("  {} converge-{}...", "Running".dimmed(), component);

        std::env::set_current_dir(&component_dir)
            .map_err(|e| CmdError::new(format!("Failed to change directory: {e}")))?;

        let status = Command::new("just")
            .arg(recipe)
            .status()
            .map_err(|e| CmdError::new(format!("Failed to run just: {e}")))?;

        std::env::set_current_dir(&root).ok();

        if !status.success() {
            return Err(CmdError::new(format!(
                "just {recipe} failed in converge-{component}"
            )));
        }
    } else {
        // Run for all components via root justfile
        std::env::set_current_dir(&root)
            .map_err(|e| CmdError::new(format!("Failed to change directory: {e}")))?;

        let status = Command::new("just")
            .arg(recipe)
            .status()
            .map_err(|e| CmdError::new(format!("Failed to run just: {e}")))?;

        if !status.success() && !args.keep_going {
            return Err(CmdError::new(format!("just {recipe} failed")));
        }
    }

    println!();
    println!("{}", format!("✓ {recipe} complete").green());
    println!();

    Ok(())
}
