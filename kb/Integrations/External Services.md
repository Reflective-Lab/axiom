---
tags: [integrations]
---
# External Services and Mocking

When real enterprise services are unavailable, mock them behind the same interface. Don't collapse the architecture — preserve the integration shape.

## Pattern

1. Define a trait for the capability the agent needs
2. Implement it against the real service for production
3. Implement it as a local mock for development
4. Inject it into the agent at construction time

The agent never knows whether it talks to a real service or a mock.

```rust
trait PolicyService: Send + Sync {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String>;
}

struct RealPolicyService {
    base_url: String,
}

impl PolicyService for RealPolicyService {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        // HTTP call to real service
        todo!()
    }
}

struct MockPolicyService;

impl PolicyService for MockPolicyService {
    fn get_policies(&self, _jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        Ok(vec![
            PolicyRule { id: "gdpr-1".into(), description: "Data must stay in EU".into() },
            PolicyRule { id: "ai-act-1".into(), description: "High-risk AI requires conformity assessment".into() },
        ])
    }
}

// Inject into agent:
struct ComplianceScreenerAgent {
    policies: Arc<dyn PolicyService>,
}
```

Swapping from mock to real is a one-line change at the injection site.

## Good Mock Candidates

- Vendor profile service — certifications, regions, pricing
- Policy engine — guardrails, jurisdiction rules
- Procurement approval — budget thresholds, escalation
- Compliance evidence store — structured documents
- Pricing catalog — token costs, volume discounts

See also: [[Building/Writing Agents]], [[Integrations/MCP Tools]]
