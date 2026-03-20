// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Error types for Converge.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::context::Context;
use crate::invariant::InvariantClass;

/// Top-level error type for Converge operations.
///
/// Note: Context is boxed in error variants to keep the error type small,
/// as recommended by clippy. Access via `error.context()` method.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ConvergeError {
    /// Budget limit exceeded (cycles, facts, or time).
    #[error("budget exhausted: {kind}")]
    BudgetExhausted { kind: String },

    /// An invariant was violated during execution.
    #[error("{class:?} invariant '{name}' violated: {reason}")]
    InvariantViolation {
        /// Name of the violated invariant.
        name: String,
        /// Class of the invariant (Structural, Semantic, Acceptance).
        class: InvariantClass,
        /// Reason for the violation.
        reason: String,
        /// Final context state (including diagnostic facts). Boxed to reduce error size.
        context: Box<Context>,
    },

    /// Agent execution failed.
    #[error("agent failed: {agent_id}")]
    AgentFailed { agent_id: String },

    /// Conflicting facts detected for the same ID.
    #[error(
        "conflict detected for fact '{id}': existing content '{existing}' vs new content '{new}'"
    )]
    Conflict {
        /// ID of the conflicting fact.
        id: String,
        /// Existing content.
        existing: String,
        /// New conflicting content.
        new: String,
        /// Final context state. Boxed to reduce error size.
        context: Box<Context>,
    },
}

impl ConvergeError {
    /// Returns a reference to the context if this error variant carries one.
    #[must_use]
    pub fn context(&self) -> Option<&Context> {
        match self {
            Self::InvariantViolation { context, .. } | Self::Conflict { context, .. } => {
                Some(context)
            }
            Self::BudgetExhausted { .. } | Self::AgentFailed { .. } => None,
        }
    }
}
