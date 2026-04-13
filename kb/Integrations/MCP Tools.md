---
tags: [integrations, mcp]
source: mixed
---
# MCP Tools

Model Context Protocol lets agents call external tools using a structured tool-call interface instead of raw HTTP.

## Usage

```rust
use converge_mcp::{McpClient, McpTransport};

let mcp = McpClient::new(
    "vendor-registry",
    McpTransport::Http {
        url: "https://mcp.example.com/vendor-registry".into(),
    },
);

// Discover available tools
let tools = mcp.list_tools()?;

// Call a specific tool
let result = mcp.call_tool("lookup_vendor", serde_json::json!({
    "vendor_name": "Acme AI",
    "fields": ["certifications", "regions", "pricing"]
}))?;
```

## MCP vs REST

| Use case | Choose |
|---|---|
| Known endpoint and payload at compile time | REST |
| Agent needs to discover tools dynamically | MCP |
| Tool server exposes many actions | MCP |
| LLM selects which tool to call based on context | MCP |

## Good MCP Candidates

- Vendor registry lookups
- Policy checks
- Compliance evidence retrieval
- Approval workflow actions

See also: [[Integrations/External Services]], [[Building/Crate Catalog]]
