use rig::agent::Agent;
use rig::agent::AgentBuilder;
use rig::client::{CompletionClient};
use rig::providers::openai;

use crate::mcp_test::MCPClient;
use crate::file_tools::{FileReader, FileWriter, FileEditor, CreateDirectory, ListFiles, CodeEditor, CodeInserter, CodebaseAnalyzer, JobExecutor};
use mcp_core::types::ToolsListResponse;


fn get_model() -> openai::CompletionModel {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let api_base = std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "http://localhost:1234/".into());
    let client = openai::Client::from_url(&api_key, &api_base);
    let model = client.completion_model("Qwen/Qwen3-32B");
    model
}

pub fn get_agent(prompt: &str, mcp_config: Option<(MCPClient, ToolsListResponse)>) -> Agent<openai::CompletionModel> {
    let model = get_model();
    let builder = AgentBuilder::new(model)
        .preamble(prompt)
        .temperature(0.2)
        .max_tokens(3000)
        // Add file manipulation tools
        .tool(FileReader)
        .tool(FileWriter)
        .tool(FileEditor)
        .tool(CodeEditor)
        .tool(CodeInserter)
        .tool(CreateDirectory)
        .tool(ListFiles)
        .tool(CodebaseAnalyzer)
        .tool(JobExecutor);

    // Add MCP tools dynamically if MCP is configured
    let builder = if let Some((mcp_client, tools)) = mcp_config {
        tools.tools
            .into_iter()
            .fold(builder, |builder, tool| {
                builder.mcp_tool(tool, mcp_client.inner.clone().into())
            })
    } else {
        builder
    };

    builder.build()
}

pub fn get_agent_with_context(prompt: &str, mcp_config: Option<(MCPClient, ToolsListResponse)>, context_docs: Vec<String>) -> Agent<openai::CompletionModel> {
    let model = get_model();
    let mut builder = AgentBuilder::new(model)
        .preamble(prompt)
        .temperature(0.2)
        .max_tokens(3000)
        // Add file manipulation tools
        .tool(FileReader)
        .tool(FileWriter)
        .tool(FileEditor)
        .tool(CodeEditor)
        .tool(CodeInserter)
        .tool(CreateDirectory)
        .tool(ListFiles)
        .tool(CodebaseAnalyzer)
        .tool(JobExecutor);

    // Add context documents
    for context_doc in context_docs {
        builder = builder.context(&context_doc);
    }

    // Add MCP tools dynamically if MCP is configured
    let builder = if let Some((mcp_client, tools)) = mcp_config {
        tools.tools
            .into_iter()
            .fold(builder, |builder, tool| {
                builder.mcp_tool(tool, mcp_client.inner.clone().into())
            })
    } else {
        builder
    };

    builder.build()
}
