# Wolfgang Alignment

This note records the decision to use `wolfgang-app` as the leading reference
for hosted application deployment patterns.

Reference repo:

- `/Users/kpernyer/repo/wolfgang-app`

## What Wolfgang Already Solves

- frontend/backend split
- Firebase client auth
- Rust backend auth middleware
- Firebase Hosting deployment
- Cloud Run backend deployment
- Secret Manager-backed env injection
- Terraform modules for Cloud Run and related GCP resources
- operational workflows centralized in a `Justfile`

## What Converge Should Reuse Conceptually

- Firebase as the default hosted identity layer
- Cloud Run as the default backend runtime
- Terraform as the preferred infrastructure source of truth
- Secret Manager for production provider/API keys
- same style of local auth bypass only for development
- shared backend auth middleware for protected HTTP routes

## What Should Stay Converge-Specific

- runtime API shape
- multi-agent execution model
- policy/governance model
- provider catalog and worker topology
- enterprise-facing security/compliance docs

## Recommended Porting Order

1. Terraform module structure
2. Secret Manager env wiring
3. Firebase Hosting / frontend routing conventions
4. GPU worker integration after the main hosted path is stable
5. route-level authorization and production CORS hardening
