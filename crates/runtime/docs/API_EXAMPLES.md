# Converge Runtime API Examples

This document provides examples for using the Converge Runtime HTTP API.

## Prerequisites

Start the server:
```bash
cd converge-runtime
cargo run
```

The server will be available at `http://localhost:8080` by default.

## Endpoints

### Health Check

**GET** `/health`

Returns "ok" if the server is running.

```bash
curl -X GET http://localhost:8080/health
```

**Response:**
```
ok
```

### Readiness Check

**GET** `/ready`

Returns readiness status and service health.

```bash
curl -X GET http://localhost:8080/ready
```

**Response:**
```json
{
  "status": "ready",
  "services": {
    "engine": "ok"
  }
}
```

### Submit Job

**POST** `/api/v1/jobs`

Submits a new job to the Converge engine and runs it until convergence.

#### Request Body

```json
{
  "context": null
}
```

Or with initial context data:

```json
{
  "context": {
    "initial_data": "test"
  }
}
```

#### Example: Empty Context

```bash
curl -X POST http://localhost:8080/api/v1/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "context": null
  }'
```

#### Example: With Context

```bash
curl -X POST http://localhost:8080/api/v1/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "context": {
      "initial_data": "test"
    }
  }'
```

### Submit LinkedIn Research Job

**POST** `/api/v1/jobs`

Submit a LinkedIn Research request with scoped intent and compliance constraints.

```bash
curl -X POST http://localhost:8080/api/v1/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "context": {
      "Seeds": [
        {
          "id": "research-request-001",
          "content": {
            "intent_type": "linkedin.research",
            "use_case": "sales",
            "targets": [
              { "entity_type": "company", "name": "ExampleCorp", "company_domain": "example.com" },
              { "entity_type": "person", "name": "Alex Rivera", "linkedin_handle": "alex-rivera" }
            ],
            "constraints": {
              "authorized_access_only": true,
              "paid_services_allowed": true,
              "external_actions_require_approval": true,
              "min_evidence_count": 3
            },
            "requested_outputs": ["dossier", "network_paths"]
          }
        }
      ]
    }
  }'
```

**Note:** External actions (messages, connection requests) must be approved via `Approvals` facts before execution.

#### Response

```json
{
  "metadata": {
    "cycles": 1,
    "converged": true,
    "duration_ms": 5
  },
  "cycles": 1,
  "converged": true,
  "context_summary": {
    "fact_counts": {
      "Seeds": 0,
      "Hypotheses": 0,
      "Strategies": 0,
      "Constraints": 0,
      "Signals": 0,
      "Competitors": 0,
      "Evaluations": 0
    },
    "version": 0
  }
}
```

### OpenAPI Documentation

**GET** `/api-docs/openapi.json`

Get the OpenAPI specification.

```bash
curl -X GET http://localhost:8080/api-docs/openapi.json
```

### Swagger UI

**GET** `/swagger-ui`

Access interactive API documentation in your browser.

Open: http://localhost:8080/swagger-ui

## Error Responses

### 400 Bad Request

Invalid JSON or malformed request.

```json
{
  "error": "Invalid JSON: ...",
  "status": 400
}
```

### 409 Conflict

Conflicting facts detected.

```json
{
  "error": "Converge error: conflict detected for fact '...'",
  "status": 409
}
```

### 413 Payload Too Large

Budget exhausted (max cycles or max facts exceeded).

```json
{
  "error": "Converge error: budget exhausted: max_cycles (100)",
  "status": 413
}
```

### 422 Unprocessable Entity

Invariant violation.

```json
{
  "error": "Converge error: Structural invariant '...' violated: ...",
  "status": 422
}
```

### 500 Internal Server Error

Server error.

```json
{
  "error": "Internal server error: ...",
  "status": 500
}
```

## Using the Examples

### cURL Script

Run the provided script:

```bash
chmod +x examples/curl-examples.sh
./examples/curl-examples.sh
```

### Postman Collection

1. Import `examples/postman-collection.json` into Postman
2. Set the `base_url` variable to your server URL (default: `http://localhost:8080`)
3. Run the requests

### HTTPie

```bash
# Health check
http GET http://localhost:8080/health

# Submit job
http POST http://localhost:8080/api/v1/jobs context:=null
```

## Next Steps

- See `README.md` for more information about the runtime
- Check `docs/deployment/DEPLOYMENT.md` for deployment guidance
- Review OpenAPI spec at `/swagger-ui` for complete API documentation

