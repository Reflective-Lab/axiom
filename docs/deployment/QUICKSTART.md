# Deployment Quickstart

This repository now supports three practical startup paths:

- native startup on macOS or Linux with Rust installed
- container startup with Docker, OrbStack, Colima, or Podman
- hosted deployment on Google Cloud Run with Terraform and Secret Manager

The default hosted shape is now:

- Firebase Auth for user identity
- Google Cloud Run for `converge-runtime`
- GCP project and Firebase project aligned by default

## 1. Native Quick Start

Best when you want the fastest local iteration on macOS.

```bash
cp .env.example .env
bash scripts/dev-up.sh native
bash scripts/smoke-test.sh
```

What this does:

- builds and starts `converge-runtime`
- binds to `http://127.0.0.1:8080`
- enables `LOCAL_DEV=true` by default
- compiles runtime with `gcp,auth,firebase` features by default
- keeps HTTP auth enabled unless you explicitly set `DISABLE_AUTH=true`

Stop it with:

```bash
bash scripts/dev-down.sh native
```

Logs are written to `.converge/runtime.log`.

## 2. Container Quick Start

Best when you do not want to install Rust locally or want a more portable setup.

Supported backends through the wrapper scripts:

- Docker Desktop
- OrbStack
- Colima with Docker
- Podman / `podman-compose`

Start:

```bash
bash scripts/dev-up.sh container
bash scripts/smoke-test.sh
```

Stop:

```bash
bash scripts/dev-down.sh container
```

### Optional extra services

The `compose.yaml` file includes optional profiles for:

- `nats`
- `surrealdb`

If you want them:

```bash
docker compose --profile extras up --build -d
```

Those services are not required for the baseline runtime bootstrap.

## 3. Cloud Run Quick Start

This is the recommended hosted deployment model for the current runtime.

Requirements:

- `gcloud` installed and authenticated
- `terraform` installed
- Cloud Build enabled
- Artifact Registry enabled
- Cloud Run enabled
- Secret Manager enabled

Deploy:

```bash
export PROJECT_ID=your-project-id
export REGION=europe-west1
export TF_STATE_BUCKET=your-terraform-state-bucket

just infra-bootstrap-state
just infra-init
just infra-apply
just cloud-build latest
just deploy-runtime latest
```

Notes:

- this path uses Terraform as the source of truth
- production secrets should come from Google Secret Manager
- Cloud Run remains stateless; Firestore is optional for persistence-backed features
- the older `scripts/deploy-cloud-run.sh` flow still exists as a thin fallback,
  but it is no longer the preferred default

## Firebase / Firestore Position

The runtime is now prepared with a Google/Firebase-first default posture:

- Cloud Run is the default hosted path
- Firebase Auth is the default identity path
- Firestore remains optional for persistence-backed features

For local development, you can still run without a real Firebase project by
using local env defaults and optionally Firebase emulators.

If you need a fast local bypass while wiring the frontend, use:

```bash
DISABLE_AUTH=true bash scripts/dev-up.sh native
```

That bypass is for development only. Hosted deployments should keep auth
enabled and provide valid Firebase bearer tokens to protected routes.

## Current Boundaries

This quickstart is intentionally minimal:

- it starts the runtime
- it exposes health and template endpoints
- it does not yet provide a full end-user web application bootstrap
- it does not yet provision frontend hosting, load balancing, or GPU workers

Those are the next deployment-hardening steps, not prerequisites for getting
started immediately.
