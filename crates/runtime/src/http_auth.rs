//! Shared HTTP authentication middleware for Axum routes.

use axum::{
    extract::Request,
    http::{HeaderMap, Uri, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::error::RuntimeError;

#[cfg(feature = "firebase")]
use crate::auth::{FirebaseConfig, FirebaseValidator, UserIdentity};

#[cfg(not(feature = "firebase"))]
use crate::auth::UserIdentity;

/// Authenticated user stored in request extensions.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub UserIdentity);

fn auth_disabled() -> bool {
    std::env::var("DISABLE_AUTH")
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn sanitize_token(token: String) -> Result<String, RuntimeError> {
    let trimmed = token.trim().to_string();
    if trimmed.is_empty() {
        return Err(RuntimeError::Authentication(
            "Missing bearer token.".to_string(),
        ));
    }
    Ok(trimmed)
}

fn extract_bearer_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .map(std::string::ToString::to_string)
}

fn extract_token_from_query(uri: &Uri) -> Option<String> {
    uri.query().and_then(|query| {
        query.split('&').find_map(|part| {
            let (key, value) = part.split_once('=')?;
            if key == "token" && !value.is_empty() {
                Some(value.to_string())
            } else {
                None
            }
        })
    })
}

pub(crate) async fn authenticate_request(
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<UserIdentity, RuntimeError> {
    if auth_disabled() {
        return Ok(UserIdentity::new("dev-user").with_roles(vec!["dev".to_string()]));
    }

    let token = sanitize_token(
        extract_bearer_from_headers(headers)
            .or_else(|| extract_token_from_query(uri))
            .ok_or_else(|| RuntimeError::Authentication("Missing bearer token.".to_string()))?,
    )?;

    validate_token(&token).await
}

pub(crate) async fn validate_token(token: &str) -> Result<UserIdentity, RuntimeError> {
    #[cfg(feature = "firebase")]
    {
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
            .or_else(|_| std::env::var("GCP_PROJECT_ID"))
            .map_err(|_| {
                RuntimeError::Config(
                    "FIREBASE_PROJECT_ID (or GCP project env) is required for Firebase auth."
                        .to_string(),
                )
            })?;

        let validator = FirebaseValidator::new(FirebaseConfig::new(&project_id));
        return validator
            .validate(token)
            .await
            .map_err(|e| RuntimeError::Authentication(format!("Invalid Firebase token: {e}")));
    }

    #[cfg(not(feature = "firebase"))]
    {
        // Non-firebase builds still enforce a token-shaped contract.
        if token.len() < 20 {
            return Err(RuntimeError::Authentication(
                "Bearer token is too short.".to_string(),
            ));
        }

        Ok(UserIdentity::new("token-user"))
    }
}

pub async fn require_auth(mut request: Request, next: Next) -> Result<Response, RuntimeError> {
    let identity = authenticate_request(request.headers(), request.uri()).await?;
    request.extensions_mut().insert(AuthenticatedUser(identity));
    Ok(next.run(request).await)
}
