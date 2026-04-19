// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! `cz bootstrap` - Development environment setup.
//!
//! Installs dependencies, creates .env, and configures git hooks.

use crate::cli::BootstrapArgs;
use crate::commands::{CmdError, CmdResult, command_exists, find_workspace_root, run_cmd};
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Run the bootstrap command.
pub async fn run(args: BootstrapArgs) -> CmdResult {
    let root = find_workspace_root()
        .ok_or_else(|| CmdError::new("Could not find workspace root (no justfile found)"))?;

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_blue()
            .bold()
    );
    println!("{}", "  CONVERGE ZONE - cz bootstrap".bright_blue().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_blue()
            .bold()
    );
    println!();

    // Step 1: Git hooks
    println!("{}", "Setting up git hooks...".bold());
    setup_git_hooks(&root)?;

    // Step 2: .env file
    if !args.skip_env {
        println!("{}", "Setting up .env...".bold());
        setup_env_file(&root, args.force)?;
    }

    // Step 3: Install dependencies
    if !args.skip_deps {
        println!("{}", "Installing dependencies...".bold());
        install_deps(&root, args.mise)?;
    }

    println!();
    println!("{}", "Bootstrap complete!".green().bold());
    println!();
    println!("Next steps:");
    println!("  {} Check environment health", "cz doctor".cyan());
    println!("  {} Run tests", "cz test".cyan());
    println!("  {} Start services", "cz up".cyan());
    println!();

    Ok(())
}

fn setup_git_hooks(root: &Path) -> CmdResult {
    let hooks_dir = root.join(".githooks");
    if hooks_dir.exists() {
        run_cmd("git", &["config", "core.hooksPath", ".githooks"])
            .map_err(|e| CmdError::new(format!("Failed to configure git hooks: {e}")))?;
        println!("  {} Git hooks configured (.githooks)", "✓".green());
    } else {
        println!("  {} No .githooks directory found", "⚠".yellow());
    }
    Ok(())
}

fn setup_env_file(root: &Path, force: bool) -> CmdResult {
    let env_file = root.join(".env");
    let env_example = root.join(".env.example");

    if env_file.exists() && !force {
        println!(
            "  {} .env already exists (use --force to overwrite)",
            "⚠".yellow()
        );
        return Ok(());
    }

    if env_example.exists() {
        fs::copy(&env_example, &env_file)
            .map_err(|e| CmdError::new(format!("Failed to copy .env.example: {e}")))?;
        println!("  {} Created .env from .env.example", "✓".green());
        println!("  {}", "Remember to fill in your API keys!".yellow().bold());
    } else {
        println!("  {} No .env.example found", "⚠".yellow());
    }

    Ok(())
}

fn install_deps(root: &Path, use_mise: bool) -> CmdResult {
    // Optional: Use mise for toolchain management
    if use_mise && command_exists("mise") {
        println!("  Installing toolchains with mise...");
        let _ = run_cmd("mise", &["install"]);
    }

    // Rust dependencies
    if root.join("Cargo.toml").exists() || has_cargo_workspace(root) {
        println!("  Fetching Rust dependencies...");
        run_cmd("cargo", &["fetch"])
            .map_err(|e| CmdError::new(format!("cargo fetch failed: {e}")))?;
        println!("  {} Rust dependencies fetched", "✓".green());
    }

    // Node/Bun dependencies (converge-www)
    let www_dir = root.join("converge-www");
    if www_dir.join("package.json").exists() {
        println!("  Installing Node dependencies (converge-www)...");
        let result = if command_exists("bun") {
            std::env::set_current_dir(&www_dir).ok();
            let r = run_cmd("bun", &["install"]);
            std::env::set_current_dir(root).ok();
            r
        } else if command_exists("npm") {
            std::env::set_current_dir(&www_dir).ok();
            let r = run_cmd("npm", &["install"]);
            std::env::set_current_dir(root).ok();
            r
        } else {
            Err("Neither bun nor npm found".to_string())
        };

        match result {
            Ok(()) => println!("  {} Node dependencies installed", "✓".green()),
            Err(e) => println!("  {} Node install failed: {}", "⚠".yellow(), e),
        }
    }

    // Elixir dependencies (converge-ledger)
    let ledger_dir = root.join("converge-ledger");
    if ledger_dir.join("mix.exs").exists() && command_exists("mix") {
        println!("  Installing Elixir dependencies (converge-ledger)...");
        std::env::set_current_dir(&ledger_dir).ok();
        let result = run_cmd("mix", &["deps.get"]);
        std::env::set_current_dir(root).ok();

        match result {
            Ok(()) => println!("  {} Elixir dependencies installed", "✓".green()),
            Err(e) => println!("  {} Elixir install failed: {}", "⚠".yellow(), e),
        }
    }

    Ok(())
}

fn has_cargo_workspace(root: &Path) -> bool {
    // Check if any converge-* directory has Cargo.toml
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name.starts_with("converge-")
                && path.join("Cargo.toml").exists()
            {
                return true;
            }
        }
    }
    false
}
