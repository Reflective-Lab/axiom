//! MCP request handlers.

use super::types::*;
use crate::core::{KnowledgeBase, KnowledgeEntry, SearchOptions};

use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// MCP request handler.
pub struct McpHandler {
    kb: Arc<RwLock<KnowledgeBase>>,
}

impl McpHandler {
    /// Create a new handler.
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
        Self { kb }
    }

    /// Handle an MCP request.
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id).await,
            "initialized" => JsonRpcResponse::success(request.id, json!({})),
            "tools/list" => self.handle_list_tools(request.id).await,
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "resources/list" => self.handle_list_resources(request.id).await,
            "resources/read" => self.handle_read_resource(request.id, request.params).await,
            "ping" => JsonRpcResponse::success(request.id, json!({})),
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: ServerInfo {
                name: "converge-knowledge".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    /// Handle list tools request.
    async fn handle_list_tools(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "knowledge_search".to_string(),
                description: "Search the knowledge base for relevant entries".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default: 5)",
                            "default": 5
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "knowledge_add".to_string(),
                description: "Add a new entry to the knowledge base".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the knowledge entry"
                        },
                        "content": {
                            "type": "string",
                            "description": "Full content of the entry"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category for classification"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags for the entry"
                        }
                    },
                    "required": ["title", "content"]
                }),
            },
            Tool {
                name: "knowledge_get".to_string(),
                description: "Get a specific knowledge entry by ID".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The entry ID (UUID)"
                        }
                    },
                    "required": ["id"]
                }),
            },
            Tool {
                name: "knowledge_feedback".to_string(),
                description: "Provide feedback on a search result to improve future searches"
                    .to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "entry_id": {
                            "type": "string",
                            "description": "The entry ID to provide feedback on"
                        },
                        "helpful": {
                            "type": "boolean",
                            "description": "Whether the result was helpful"
                        }
                    },
                    "required": ["entry_id", "helpful"]
                }),
            },
            Tool {
                name: "knowledge_stats".to_string(),
                description: "Get statistics about the knowledge base".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        let result = ListToolsResult { tools };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    /// Handle call tool request.
    async fn handle_call_tool(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string());
            }
        };

        let request: CallToolRequest = match serde_json::from_value(params) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        };

        let result = match request.name.as_str() {
            "knowledge_search" => self.tool_search(&request.arguments).await,
            "knowledge_add" => self.tool_add(&request.arguments).await,
            "knowledge_get" => self.tool_get(&request.arguments).await,
            "knowledge_feedback" => self.tool_feedback(&request.arguments).await,
            "knowledge_stats" => self.tool_stats().await,
            _ => Err(format!("Unknown tool: {}", request.name)),
        };

        match result {
            Ok(content) => JsonRpcResponse::success(id, serde_json::to_value(content).unwrap()),
            Err(e) => {
                let error_result = CallToolResult {
                    content: vec![ToolContent::Text { text: e }],
                    is_error: Some(true),
                };
                JsonRpcResponse::success(id, serde_json::to_value(error_result).unwrap())
            }
        }
    }

    /// Handle knowledge search tool.
    async fn tool_search(
        &self,
        args: &std::collections::HashMap<String, serde_json::Value>,
    ) -> std::result::Result<CallToolResult, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'query' argument".to_string())?;

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

        let kb = self.kb.read().await;
        let results = kb
            .search(query, SearchOptions::new(limit))
            .await
            .map_err(|e| e.to_string())?;

        let mut text = format!("Found {} results:\n\n", results.len());

        for (i, result) in results.iter().enumerate() {
            text.push_str(&format!(
                "{}. **{}** (score: {:.2})\n   ID: {}\n   {}\n\n",
                i + 1,
                result.entry.title,
                result.score,
                result.entry.id,
                truncate(&result.entry.content, 200)
            ));
        }

        Ok(CallToolResult {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        })
    }

    /// Handle knowledge add tool.
    async fn tool_add(
        &self,
        args: &std::collections::HashMap<String, serde_json::Value>,
    ) -> std::result::Result<CallToolResult, String> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'title' argument".to_string())?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'content' argument".to_string())?;

        let mut entry = KnowledgeEntry::new(title, content);

        if let Some(category) = args.get("category").and_then(|v| v.as_str()) {
            entry = entry.with_category(category);
        }

        if let Some(tags) = args.get("tags").and_then(|v| v.as_array()) {
            let tags: Vec<String> = tags
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            entry = entry.with_tags(tags);
        }

        let kb = self.kb.read().await;
        let id = kb.add_entry(entry).await.map_err(|e| e.to_string())?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: format!("Added entry with ID: {}", id),
            }],
            is_error: None,
        })
    }

    /// Handle knowledge get tool.
    async fn tool_get(
        &self,
        args: &std::collections::HashMap<String, serde_json::Value>,
    ) -> std::result::Result<CallToolResult, String> {
        let id_str = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'id' argument".to_string())?;

        let id = Uuid::parse_str(id_str).map_err(|e| format!("Invalid UUID: {}", e))?;

        let kb = self.kb.read().await;
        match kb.get(id) {
            Some(entry) => {
                let text = format!(
                    "**{}**\n\nID: {}\nCategory: {}\nTags: {}\nCreated: {}\nAccess Count: {}\n\n---\n\n{}",
                    entry.title,
                    entry.id,
                    entry.category.as_deref().unwrap_or("None"),
                    entry.tags.join(", "),
                    entry.created_at,
                    entry.access_count,
                    entry.content
                );

                Ok(CallToolResult {
                    content: vec![ToolContent::Text { text }],
                    is_error: None,
                })
            }
            None => Err(format!("Entry not found: {}", id_str)),
        }
    }

    /// Handle knowledge feedback tool.
    async fn tool_feedback(
        &self,
        args: &std::collections::HashMap<String, serde_json::Value>,
    ) -> std::result::Result<CallToolResult, String> {
        let id_str = args
            .get("entry_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'entry_id' argument".to_string())?;

        let helpful = args
            .get("helpful")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| "Missing 'helpful' argument".to_string())?;

        let id = Uuid::parse_str(id_str).map_err(|e| format!("Invalid UUID: {}", e))?;

        let kb = self.kb.read().await;
        kb.record_feedback(id, helpful)
            .await
            .map_err(|e| e.to_string())?;

        let text = if helpful {
            "Thank you for the positive feedback! The knowledge base will learn from this."
        } else {
            "Thank you for the feedback. The knowledge base will adjust its rankings."
        };

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: text.to_string(),
            }],
            is_error: None,
        })
    }

    /// Handle knowledge stats tool.
    async fn tool_stats(&self) -> std::result::Result<CallToolResult, String> {
        let kb = self.kb.read().await;
        let stats = kb.stats();

        let text = format!(
            "Knowledge Base Statistics:\n\n\
            - Total Entries: {}\n\
            - Unique Categories: {}\n\
            - Unique Tags: {}\n\
            - Total Access Count: {}\n\
            - Embedding Dimensions: {}\n\
            - Learning Enabled: {}",
            stats.total_entries,
            stats.unique_categories,
            stats.unique_tags,
            stats.total_access_count,
            stats.dimensions,
            stats.learning_enabled
        );

        Ok(CallToolResult {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        })
    }

    /// Handle list resources request.
    async fn handle_list_resources(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let kb = self.kb.read().await;
        let entries = kb.all_entries();

        let resources: Vec<Resource> = entries
            .iter()
            .map(|e| Resource {
                uri: format!("knowledge://{}", e.id),
                name: e.title.clone(),
                description: Some(truncate(&e.content, 100)),
                mime_type: Some("text/markdown".to_string()),
            })
            .collect();

        let result = ListResourcesResult { resources };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    /// Handle read resource request.
    async fn handle_read_resource(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string());
            }
        };

        let request: ReadResourceRequest = match serde_json::from_value(params) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        };

        // Parse URI: knowledge://{id}
        let entry_id = request
            .uri
            .strip_prefix("knowledge://")
            .unwrap_or(&request.uri);

        let uuid = match Uuid::parse_str(entry_id) {
            Ok(u) => u,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid entry ID: {}", e));
            }
        };

        let kb = self.kb.read().await;
        match kb.get(uuid) {
            Some(entry) => {
                let content = format!(
                    "# {}\n\n{}\n\n---\n\n**Category:** {}\n**Tags:** {}\n**Created:** {}",
                    entry.title,
                    entry.content,
                    entry.category.as_deref().unwrap_or("None"),
                    entry.tags.join(", "),
                    entry.created_at
                );

                let result = ReadResourceResult {
                    contents: vec![ResourceContent {
                        uri: request.uri,
                        mime_type: Some("text/markdown".to_string()),
                        text: Some(content),
                    }],
                };

                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            None => JsonRpcResponse::error(id, -32602, format!("Entry not found: {}", entry_id)),
        }
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
