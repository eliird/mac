use anyhow::Error;
use dotenv::dotenv;
use std::fs;
use chrono::{DateTime, Local};

mod gemini;
mod local;
mod mcp_test;
mod model_selector;
mod file_tools;
mod context_workflow;

use std::io::{self, Write};
use crate::mcp_test::MCPClient;
use crate::model_selector::AgentWrapper;
use crate::context_workflow::ContextWorkflow;
use rig::completion::{Message, Prompt, Chat};

fn read_text_file(file: &str) -> Result<String, Error> {
    fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read prompt.txt: {}", e))
}


pub async fn run_chat_with_tools(agent: AgentWrapper) -> Result<(), Error> {

    println!("Prompting RIG agent:");
    let mut history = Vec::new();

    let question1 = "What database tables do we have?";
    println!("Question: {}", question1);
    let response = agent.prompt("what database tables do we have?").await?;
    println!("Agent response: {:?}", response);
    history.push(Message::user(question1));
    history.push(Message::assistant(response));
    println!("------------------------------------------");
    let question2 = "What is inside the tables of the database?";
    println!("Question: {}", question2);
    let response = agent.prompt("what database tables do we have in `my_dummy_db`?").await?;
    println!("Agent response: {:?}", response);

    history.push(Message::user(question2));
    history.push(Message::assistant(response));

    println!("------------------------------------------");

    Ok(())
}


async fn run_cli_chat(agent: AgentWrapper) -> Result<(), Error> {

    println!("✨ Welcome to the Context-Aware LLMO Assistant! ✨");
    println!("I have full knowledge of the test_code directory and can help with:");
    println!("• Code analysis and optimization suggestions");
    println!("• Performance bottleneck identification");
    println!("• Architecture and structure questions");
    println!("• Code modifications and improvements");
    println!("Feel free to ask me anything! Type 'exit' or 'quit' when you're done.");
    println!("---------------------------------------------------");

    let mut history = Vec::new();
    let stdin = io::stdin();


    loop {
        print!("You: ");
        io::stdout().flush()?; // Print prompt immediately

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        history.push(Message::user(input));

        println!("🤔 Processing your request...");
        match agent.chat(input, history.clone()).await {
            Ok(response) => {
                println!("Assistant: {}", response);
                history.push(Message::assistant(response));
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
        println!("---------------------------------------------------");
    }

    Ok(())
}

async fn create_contextual_agent(
    system_prompt: &str,
    mcp_client: MCPClient,
    tools: mcp_core::types::ToolsListResponse,
    codebase_path: &str,
    exection_script: &str,
) -> Result<AgentWrapper, Error> {
    println!("🔍 Analyzing codebase at: {}", codebase_path);

    // Analyze codebase directly (no LLM calls, avoids MaxDepthError)
    let codebase_content = ContextWorkflow::analyze_codebase(codebase_path).await?;
    println!("✅ Codebase analysis completed ({} characters)", codebase_content.len());

    // Create context documents
    let context_docs = ContextWorkflow::create_context_docs(&codebase_content, None, exection_script);
    println!("✅ Context documents prepared ({} docs)", context_docs.len());

    // Create agent with context
    let agent = model_selector::get_agent_with_context(system_prompt, mcp_client, tools, context_docs);
    println!("✅ Context-aware agent created");

    Ok(agent)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🚀 Starting LLMO application...");

    println!("📄 Loading environment variables...");
    dotenv().ok();

    println!("📋 Loading system prompt...");
    let system_prompt = read_text_file("prompt.txt")?;
    println!("✅ System prompt loaded successfully");

    println!("🔌 Initializing MCP client...");
    let mcp_client = MCPClient::new().await?;
    println!("✅ MCP client initialized");

    println!("🛠️  Listing available tools...");
    let tools = mcp_client.inner.list_tools(None, None).await?;
    println!("✅ Found {:?} tools", tools);

    println!("🤖 Setting up context-aware agent...");
    let codebase_path = "test_code";
    let job_execution_script = "./test_code/run.sh";
    println!("📂 Codebase path: {}", codebase_path);
    println!("📄 Job execution script: {}", job_execution_script);
    let agent = create_contextual_agent(&system_prompt, mcp_client, tools, codebase_path, job_execution_script).await?;
    println!("✅ Context-aware agent ready with knowledge of {}", codebase_path);

    run_cli_chat(agent).await?;
    Ok(())
}
