// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! `cz doctor` - Environment health checks.
//!
//! Verifies that all required tools are installed and configured correctly.

use crate::cli::{DoctorArgs, OutputFormat};
use crate::commands::{CmdError, CmdResult, command_exists, find_workspace_root, run_cmd_output};
use colored::Colorize;
use std::collections::HashMap;

/// A single health check result.
#[derive(Debug)]
struct Check {
    name: String,
    status: CheckStatus,
    version: Option<String>,
    fix_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// Run the doctor command.
pub async fn run(args: DoctorArgs) -> CmdResult {
    let checks = gather_checks(&args).await;
    let (pass, warn, fail) = count_status(&checks);

    match args.format {
        OutputFormat::Pretty => print_pretty(&checks, pass, warn, fail),
        OutputFormat::Json => print_json(&checks),
        OutputFormat::Quiet => {}
    }

    if fail > 0 {
        Err(CmdError::new(format!("{fail} check(s) failed")))
    } else {
        Ok(())
    }
}

async fn gather_checks(args: &DoctorArgs) -> Vec<Check> {
    let mut checks = vec![
        check_tool("rustc", &["--version"], "https://rustup.rs"),
        check_tool("cargo", &["--version"], "https://rustup.rs"),
        check_tool("just", &["--version"], "cargo install just"),
        check_tool("docker", &["--version"], "https://docker.com"),
        check_tool("git", &["--version"], "https://git-scm.com"),
    ];

    // Language-specific (check if component exists before requiring)
    if args.component.is_none() || args.component.as_deref() == Some("www") {
        checks.push(check_tool("bun", &["--version"], "https://bun.sh"));
    }

    if args.component.is_none() || args.component.as_deref() == Some("ledger") {
        checks.push(check_tool("mix", &["--version"], "https://elixir-lang.org"));
    }

    // Docker daemon
    checks.push(check_docker_running());

    // Rust version
    checks.push(check_rust_version());

    // Git hooks
    checks.push(check_git_hooks());

    // .env file
    checks.push(check_env_file());

    // Cargo tools (optional)
    if args.verbose {
        checks.push(check_cargo_tool("cargo-watch"));
        checks.push(check_cargo_tool("cargo-deny"));
        checks.push(check_cargo_tool("cargo-llvm-cov"));
    }

    checks
}

fn check_tool(name: &str, version_args: &[&str], install_hint: &str) -> Check {
    if !command_exists(name) {
        return Check {
            name: name.to_string(),
            status: CheckStatus::Fail,
            version: None,
            fix_hint: Some(install_hint.to_string()),
        };
    }

    let version = run_cmd_output(name, version_args)
        .ok()
        .map(|v| v.lines().next().unwrap_or("").trim().to_string());

    Check {
        name: name.to_string(),
        status: CheckStatus::Pass,
        version,
        fix_hint: None,
    }
}

fn check_cargo_tool(name: &str) -> Check {
    // cargo tools are invoked as `cargo <tool>` not directly
    let tool_name = name.strip_prefix("cargo-").unwrap_or(name);
    let result = run_cmd_output("cargo", &[tool_name, "--version"]);

    match result {
        Ok(version) => Check {
            name: name.to_string(),
            status: CheckStatus::Pass,
            version: Some(version.lines().next().unwrap_or("").trim().to_string()),
            fix_hint: None,
        },
        Err(_) => Check {
            name: name.to_string(),
            status: CheckStatus::Warn,
            version: None,
            fix_hint: Some(format!("cargo install {name}")),
        },
    }
}

fn check_docker_running() -> Check {
    let result = run_cmd_output("docker", &["info"]);
    match result {
        Ok(_) => Check {
            name: "docker daemon".to_string(),
            status: CheckStatus::Pass,
            version: Some("running".to_string()),
            fix_hint: None,
        },
        Err(_) => Check {
            name: "docker daemon".to_string(),
            status: CheckStatus::Fail,
            version: None,
            fix_hint: Some("Start Docker Desktop or run: systemctl start docker".to_string()),
        },
    }
}

fn check_rust_version() -> Check {
    let result = run_cmd_output("rustc", &["--version"]);
    match result {
        Ok(version) => {
            // Parse version like "rustc 1.85.0 (..."
            let version_str = version.trim();
            let meets_minimum = version_str
                .split_whitespace()
                .nth(1)
                .and_then(|v| {
                    let parts: Vec<&str> = v.split('.').collect();
                    if parts.len() >= 2 {
                        let major: u32 = parts[0].parse().ok()?;
                        let minor: u32 = parts[1].parse().ok()?;
                        Some(major > 1 || (major == 1 && minor >= 85))
                    } else {
                        None
                    }
                })
                .unwrap_or(false);

            if meets_minimum {
                Check {
                    name: "rust version".to_string(),
                    status: CheckStatus::Pass,
                    version: Some(format!("{version_str} (>= 1.85 required)")),
                    fix_hint: None,
                }
            } else {
                Check {
                    name: "rust version".to_string(),
                    status: CheckStatus::Fail,
                    version: Some(version_str.to_string()),
                    fix_hint: Some("rustup update stable".to_string()),
                }
            }
        }
        Err(_) => Check {
            name: "rust version".to_string(),
            status: CheckStatus::Fail,
            version: None,
            fix_hint: Some("https://rustup.rs".to_string()),
        },
    }
}

fn check_git_hooks() -> Check {
    let result = run_cmd_output("git", &["config", "core.hooksPath"]);
    match result {
        Ok(path) if path.trim() == ".githooks" => Check {
            name: "git hooks".to_string(),
            status: CheckStatus::Pass,
            version: Some(".githooks".to_string()),
            fix_hint: None,
        },
        _ => Check {
            name: "git hooks".to_string(),
            status: CheckStatus::Warn,
            version: None,
            fix_hint: Some("cz bootstrap or: git config core.hooksPath .githooks".to_string()),
        },
    }
}

fn check_env_file() -> Check {
    if let Some(root) = find_workspace_root() {
        if root.join(".env").exists() {
            return Check {
                name: ".env file".to_string(),
                status: CheckStatus::Pass,
                version: Some("present".to_string()),
                fix_hint: None,
            };
        } else if root.join(".env.example").exists() {
            return Check {
                name: ".env file".to_string(),
                status: CheckStatus::Warn,
                version: None,
                fix_hint: Some("cz bootstrap or: cp .env.example .env".to_string()),
            };
        }
    }

    Check {
        name: ".env file".to_string(),
        status: CheckStatus::Warn,
        version: None,
        fix_hint: Some("Create .env with required configuration".to_string()),
    }
}

fn count_status(checks: &[Check]) -> (usize, usize, usize) {
    let mut pass = 0;
    let mut warn = 0;
    let mut fail = 0;
    for c in checks {
        match c.status {
            CheckStatus::Pass => pass += 1,
            CheckStatus::Warn => warn += 1,
            CheckStatus::Fail => fail += 1,
        }
    }
    (pass, warn, fail)
}

fn print_pretty(checks: &[Check], pass: usize, warn: usize, fail: usize) {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_blue()
            .bold()
    );
    println!("{}", "  CONVERGE ZONE - cz doctor".bright_blue().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .bright_blue()
            .bold()
    );
    println!();

    for check in checks {
        let (icon, color) = match check.status {
            CheckStatus::Pass => ("✓", "green"),
            CheckStatus::Warn => ("⚠", "yellow"),
            CheckStatus::Fail => ("✗", "red"),
        };

        let icon_colored = match color {
            "green" => icon.green(),
            "yellow" => icon.yellow(),
            "red" => icon.red(),
            _ => icon.normal(),
        };

        let version_str = check.version.as_deref().unwrap_or("");
        println!(
            "  {icon_colored} {:<20} {}",
            check.name,
            version_str.dimmed()
        );

        if let Some(hint) = &check.fix_hint {
            println!("    {} {}", "→".dimmed(), hint.dimmed());
        }
    }

    println!();
    println!(
        "{}",
        "───────────────────────────────────────────────────────────".dimmed()
    );

    let summary = format!("  {pass} passed, {warn} warnings, {fail} failed");
    if fail > 0 {
        println!("{}", summary.red());
    } else if warn > 0 {
        println!("{}", summary.yellow());
    } else {
        println!("{}", summary.green());
    }
    println!();
}

fn print_json(checks: &[Check]) {
    let json_checks: Vec<HashMap<&str, serde_json::Value>> = checks
        .iter()
        .map(|c| {
            let mut map = HashMap::new();
            map.insert("name", serde_json::Value::String(c.name.clone()));
            map.insert(
                "status",
                serde_json::Value::String(format!("{:?}", c.status).to_lowercase()),
            );
            if let Some(v) = &c.version {
                map.insert("version", serde_json::Value::String(v.clone()));
            }
            if let Some(h) = &c.fix_hint {
                map.insert("fix_hint", serde_json::Value::String(h.clone()));
            }
            map
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_checks).unwrap());
}
