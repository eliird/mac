use rig::agent::Agent;
use rig::providers::{openai, gemini};
use rig::completion::{Message, Prompt, Chat};
use crate::mcp_test::MCPClient;
use mcp_core::types::ToolsListResponse;
use anyhow::Error;

pub enum ModelType {
    Local,
    Gemini,
}

pub enum AgentWrapper {
    Local(Agent<openai::CompletionModel>),
    Gemini(Agent<gemini::completion::CompletionModel>),
}

impl AgentWrapper {
    pub async fn prompt(&self, prompt: &str) -> Result<String, Error> {
        match self {
            AgentWrapper::Local(agent) => agent.prompt(prompt).await.map_err(|e| anyhow::anyhow!(e)),
            AgentWrapper::Gemini(agent) => agent.prompt(prompt).await.map_err(|e| anyhow::anyhow!(e)),
        }
    }

    pub async fn chat(&self, prompt: &str, history: Vec<Message>) -> Result<String, Error> {
        match self {
            AgentWrapper::Local(agent) => agent.chat(prompt, history).await.map_err(|e| anyhow::anyhow!(e)),
            AgentWrapper::Gemini(agent) => agent.chat(prompt, history).await.map_err(|e| anyhow::anyhow!(e)),
        }
    }
}

pub fn get_model_type() -> ModelType {
    match std::env::var("USE_MODEL").as_deref() {
        Ok("gemini") => ModelType::Gemini,
        Ok("local") => ModelType::Local,
        _ => ModelType::Local, // Default to local
    }
}

pub fn get_agent(prompt: &str, mcp_client: MCPClient, tools: ToolsListResponse) -> AgentWrapper {
    match get_model_type() {
        ModelType::Local => {
            let agent = crate::local::get_agent(prompt, mcp_client, tools);
            AgentWrapper::Local(agent)
        }
        ModelType::Gemini => {
            let agent = crate::gemini::_get_agent(prompt, mcp_client, tools);
            AgentWrapper::Gemini(agent)
        }
    }
}

pub fn get_agent_with_context(prompt: &str, mcp_client: MCPClient, tools: ToolsListResponse, context_docs: Vec<String>) -> AgentWrapper {
    match get_model_type() {
        ModelType::Local => {
            let agent = crate::local::get_agent_with_context(prompt, mcp_client, tools, context_docs);
            AgentWrapper::Local(agent)
        }
        ModelType::Gemini => {
            let agent = crate::gemini::_get_agent_with_context(prompt, mcp_client, tools, context_docs);
            AgentWrapper::Gemini(agent)
        }
    }
}
