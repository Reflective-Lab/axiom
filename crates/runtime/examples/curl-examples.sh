#!/bin/bash
# Converge Runtime API - cURL Examples
#
# Make sure the server is running:
#   cd converge-runtime && cargo run
#
# Server should be available at http://localhost:8080

BASE_URL="http://localhost:8080"

echo "=== Converge Runtime API Examples ==="
echo ""

# Health check
echo "1. Health Check"
echo "GET $BASE_URL/health"
curl -X GET "$BASE_URL/health" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"
echo ""
echo ""

# Readiness check
echo "2. Readiness Check"
echo "GET $BASE_URL/ready"
curl -X GET "$BASE_URL/ready" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"
echo ""
echo ""

# Submit a job (empty context)
echo "3. Submit Job (Empty Context)"
echo "POST $BASE_URL/api/v1/jobs"
curl -X POST "$BASE_URL/api/v1/jobs" \
  -H "Content-Type: application/json" \
  -d '{
    "context": null
  }' \
  -w "\nStatus: %{http_code}\n"
echo ""
echo ""

# Submit a job (with context data)
echo "4. Submit Job (With Context)"
echo "POST $BASE_URL/api/v1/jobs"
curl -X POST "$BASE_URL/api/v1/jobs" \
  -H "Content-Type: application/json" \
  -d '{
    "context": {
      "initial_data": "test"
    }
  }' \
  -w "\nStatus: %{http_code}\n"
echo ""
echo ""

# Get OpenAPI spec
echo "5. Get OpenAPI Specification"
echo "GET $BASE_URL/api-docs/openapi.json"
curl -X GET "$BASE_URL/api-docs/openapi.json" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"
echo ""
echo ""

echo "=== Examples Complete ==="

