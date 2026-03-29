# Terraform GCP Deployment

This is now the default hosted deployment path for `converge-runtime`.

Reference layout:

- `infra/modules/artifact-registry`
- `infra/modules/cloud-run-service`
- `infra/environments/prod/converge-runtime`

## One-Time Setup

1. Create a Terraform state bucket.
2. Initialize Terraform with that bucket.
3. Create the Secret Manager secrets you want mounted into Cloud Run.

Example:

```bash
export PROJECT_ID=your-gcp-project
export REGION=europe-west1
export TF_STATE_BUCKET=your-terraform-state-bucket

just infra-bootstrap-state
just infra-init
```

## Secrets

Production secrets should be stored in Google Secret Manager, not `.env`.

Create the secret containers:

```bash
just secret-create converge-openai-api-key
just secret-create converge-anthropic-api-key
just secret-create converge-gemini-api-key
```

Then add a version from a local file:

```bash
just secret-put-file converge-openai-api-key ~/.config/converge/openai.key
```

## Deploy

1. Build and push the runtime image with Cloud Build.
2. Apply Terraform with the image tag you want to serve.

Example:

```bash
export PROJECT_ID=your-gcp-project
export REGION=europe-west1
export TF_STATE_BUCKET=your-terraform-state-bucket

just infra-init
just infra-apply
just cloud-build latest
just deploy-runtime latest
```

## Notes

- Cloud Run remains publicly invokable by default because app-level Firebase
  auth protects the API routes.
- Secret mappings are declared in Terraform through `secret_env_vars`.
- Local `.env` files are still fine for native development, but not for hosted
  production.
