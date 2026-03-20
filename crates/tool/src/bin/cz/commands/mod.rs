// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: LicenseRef-Proprietary

//! Command implementations for cz CLI.
//!
//! Each module handles a category of commands:
//! - `doctor`: Environment health checks
//! - `bootstrap`: Development setup
//! - `dev`: Build/test/lint workflows
//! - `services`: Docker Compose orchestration
//! - `governance`: Validate/ack/escalate workflows

pub mod bootstrap;
pub mod dev;
pub mod doctor;
pub mod governance;
pub mod services;

use std::process::Command;

/// Result type for command execution.
pub type CmdResult = Result<(), CmdError>;

/// Error type for command execution.
#[derive(Debug)]
pub struct CmdError {
    pub message: String,
    pub details: Option<String>,
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message)?;
        if let Some(details) = &self.details {
            write!(f, "\n{details}")?;
        }
        Ok(())
    }
}

impl CmdError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: None,
        }
    }
}

/// Run a shell command and return success/failure.
pub fn run_cmd(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to execute {program}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} exited with code {}",
            status.code().unwrap_or(-1)
        ))
    }
}

/// Run a shell command and capture output.
pub fn run_cmd_output(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute {program}: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Check if a command exists in PATH.
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Find workspace root by looking for justfile.
pub fn find_workspace_root() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("justfile").exists() || dir.join("Justfile").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
