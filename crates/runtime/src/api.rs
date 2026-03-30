// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! `OpenAPI` schema definitions for Converge Runtime.

use utoipa::OpenApi;

use crate::error::RuntimeErrorResponse;
use crate::handlers::{
    ContextSummary, FormApprovalRequest, FormApprovalResponse, FormExecuteRequest,
    FormExecuteResponse, FormFieldInput, FormPlanRequest, FormPlanResponse, JobMetadata,
    JobRequest, JobResponse, ValidateRulesRequest, ValidateRulesResponse, ValidationIssueResponse,
};
use crate::templates::{
    AgentOverrides, AgentWiring, BudgetConfig, CompatibilityRequirements, CustomRequirements,
    JobOverrides, PackConfig, PackJobRequest, PackSummary, ProviderPreferences, RequirementsConfig,
    SeedFact,
};

/// `OpenAPI` schema for Converge Runtime API.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health,
        crate::handlers::ready,
        crate::handlers::handle_job,
        crate::handlers::plan_form,
        crate::handlers::approve_form,
        crate::handlers::execute_form,
        crate::handlers::validate_rules,
        crate::handlers::list_templates,
        crate::handlers::get_template,
        crate::handlers::execute_template_job,
    ),
    components(schemas(
        JobRequest,
        JobResponse,
        JobMetadata,
        ContextSummary,
        FormFieldInput,
        FormPlanRequest,
        FormPlanResponse,
        FormApprovalRequest,
        FormApprovalResponse,
        FormExecuteRequest,
        FormExecuteResponse,
        ValidateRulesRequest,
        ValidateRulesResponse,
        ValidationIssueResponse,
        RuntimeErrorResponse,
        // Pack types (new names)
        PackConfig,
        PackSummary,
        PackJobRequest,
        BudgetConfig,
        CompatibilityRequirements,
        AgentWiring,
        RequirementsConfig,
        CustomRequirements,
        SeedFact,
        JobOverrides,
        AgentOverrides,
        ProviderPreferences,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "jobs", description = "Job execution endpoints"),
        (name = "forms", description = "Form planning and execution endpoints"),
        (name = "templates", description = "Job template management"),
        (name = "validation", description = "Converge Rules validation"),
    ),
    info(
        title = "Converge Runtime API",
        description = "HTTP API for the Converge Agent OS - supports template-based multi-agent job execution",
        version = "0.1.0",
        contact(
            name = "Converge",
            url = "https://github.com/converge",
        ),
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
    ),
)]
pub struct ApiDoc;
