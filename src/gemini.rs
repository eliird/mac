// src/gemini.rs
use anyhow::Error;
use mcp_core::types::ToolsListResponse;
use rig::agent::{self, Agent};
use rig::{completion::Prompt};
use rig::client::ProviderClient;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;
use crate::mcp_test::MCPClient;
use crate::file_tools::{FileReader, FileWriter, FileEditor, CreateDirectory, ListFiles, CodeEditor, CodeInserter, CodebaseAnalyzer, JobExecutor};
use rig::providers::gemini::{completion, Client as GeminiClient};
use serde_json;


fn _get_model() -> GeminiCompletionModel{
    let model_name = std::env::var("GEMINI_MODEL_NAME").unwrap_or_else(|_| "gemini-1.5-pro".to_string());
    let client = GeminiClient::from_env();
    let model = GeminiCompletionModel::new(client, model_name.as_str());
    model
}

pub fn _get_agent(prompt:&str, mcp_client: MCPClient, tools: ToolsListResponse) -> Agent<GeminiCompletionModel> {
    let model = _get_model();
    let mut builder = agent::AgentBuilder::new(model)
        .preamble(prompt)
        .temperature(0.2)
        .max_tokens(1000)
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

    builder = tools.tools
        .into_iter()
        .fold(builder, |builder, tool| {
            builder.mcp_tool(tool, mcp_client.inner.clone().into())
        });
    builder.build()
}

pub fn _get_agent_with_context(prompt:&str, mcp_client: MCPClient, tools: ToolsListResponse, context_docs: Vec<String>) -> Agent<GeminiCompletionModel> {
    let model = _get_model();
    let mut builder = agent::AgentBuilder::new(model)
        .preamble(prompt)
        .temperature(0.2)
        .max_tokens(1000)
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

    builder = tools.tools
        .into_iter()
        .fold(builder, |builder, tool| {
            builder.mcp_tool(tool, mcp_client.inner.clone().into())
        });
    builder.build()
}

pub async fn _run_gemini() -> Result<(), Error> {
    let client = GeminiClient::from_env();
    let agent = client
        .agent(completion::GEMINI_1_5_PRO)
        .preamble("You are precise and informative.")
        .temperature(0.2)
        .additional_params(serde_json::to_value(completion::gemini_api_types::GenerationConfig {
            top_k: Some(1),
            top_p: Some(0.95),
            candidate_count: Some(1),
            ..Default::default()
        })?)
        .build();

    let resp = agent.prompt("What is the capital of Japan?").await?;
    println!("Gemini client says: {}", resp);
    Ok(())
}
