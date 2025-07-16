use std::time::Duration;
use anyhow::Result;
use clap::{Parser, ValueEnum};
use mcp_core::{
    client::ClientBuilder,
    protocol::RequestOptions,
    transport::ClientSseTransportBuilder,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum, default_value_t = TransportType::Sse)]
    transport: TransportType,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum TransportType {
    Stdio,
    Sse,
}

pub struct MCPClient {
    pub(crate) inner: mcp_core::client::Client<mcp_core::transport::ClientSseTransport>,
}

impl MCPClient {
    pub async fn new() -> Result<Self> {
        // Use default MCP URL from environment or fallback
        let mcp_url = std::env::var("MCP_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000/sse".to_string());
        Self::new_with_url(&mcp_url).await
    }
    
    pub async fn new_with_url(mcp_agent_path_sse: &str) -> Result<Self> {
        // Build SSE transport and initialize client
        // Adjust the URL as needed for your setup
        tracing::info!("Initializing MCP client with SSE transport...");
        let transport = ClientSseTransportBuilder::new(mcp_agent_path_sse.to_string())
            .build();

        // Build client with transport
        let client = ClientBuilder::new(transport)
            .set_protocol_version(mcp_core::types::ProtocolVersion::V2024_11_05)
            .set_client_info("tool_list_client".to_string(), "0.1.0".to_string())
            .build();

        client.open().await.expect("Failed to open client");
        client.initialize().await.expect("Failed to initialize client");

        Ok(MCPClient { inner: client })
    }

    async fn _request(&self, endpoint: &str, params: Option<serde_json::Value>, options: RequestOptions) -> Result<serde_json::Value> {
        self.inner.request(endpoint, params, options).await
    }

    pub async fn _get_tools_list(&self) -> Result<serde_json::Value> {
        self._request("tools/list", None, RequestOptions::default().timeout(Duration::from_secs(5))).await
    }

    pub async fn _run_tool(&self, tool_name: &str, tool_arguments: serde_json::Value) -> Result<serde_json::Value> {
        // The JSON-RPC method should always be "tools/call" for executing tools
        let jsonrpc_method = "tools/call";

        // The 'params' field of the JSON-RPC request needs to contain
        // 'name' (the actual tool name) and 'arguments' (the tool's parameters)
        let jsonrpc_params = serde_json::json!({
            "name": tool_name,          // This is the actual tool name (e.g., "list_tables")
            "arguments": tool_arguments // This is the map of arguments for the tool function
        });

        tracing::info!("Attempting to call MCP tool via 'tools/call' method:");
        tracing::info!("JSON-RPC Method: {}", jsonrpc_method);
        tracing::info!("JSON-RPC Params: {}", jsonrpc_params.to_string());

        // Call the inner request method with the correct JSON-RPC method and params structure
        self._request(
            jsonrpc_method,
            Some(jsonrpc_params),
            RequestOptions::default().timeout(Duration::from_secs(5))
        ).await
    }

}
