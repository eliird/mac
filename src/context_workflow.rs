use crate::file_tools::{CodebaseAnalyzer, FileToolError};
use rig::tool::Tool;
use anyhow::Error;

/// Helper functions for context-aware agent workflows
pub struct ContextWorkflow;

impl ContextWorkflow {
    /// Analyze a codebase and return the content for use as context
    pub async fn analyze_codebase(codebase_path: &str) -> Result<String, Error> {
        let analyzer = CodebaseAnalyzer;
        let args = crate::file_tools::AnalyzeCodebaseArgs {
            path: codebase_path.to_string(),
            max_file_size: 10000,
            max_depth: 10,
        };

        analyzer.call(args).await.map_err(|e: FileToolError| anyhow::anyhow!(e))
    }

    /// Generate a summary analysis of the codebase content
    pub fn create_analysis_prompt(codebase_content: &str, execution_script: &str) -> String {
        format!(
            "CODEBASE ANALYSIS AND CONTEXT:\n\n\
            This is a comprehensive analysis of the codebase you'll be working with. \
            Use this information to answer questions and make suggestions about the code.\n\n\
            {}\n\n\
            END OF CODEBASE ANALYSIS\n\n\
            Based on this codebase, you can now answer questions about:\n\
            - Code structure and architecture\n\
            - Performance optimization opportunities\n\
            - Bug fixes and improvements\n\
            - Adding new features\n\
            - Understanding existing functionality\n\
            - You can use this script to run the code and analyze it: {}\n\
            END OF CONTEXT\n\
            ",
            codebase_content,
            execution_script
        )
    }

    /// Create context documents for agent initialization
    pub fn create_context_docs(codebase_content: &str, additional_context: Option<String>, execution_script: &str) -> Vec<String> {
        let mut docs = vec![Self::create_analysis_prompt(codebase_content, execution_script)];

        if let Some(additional) = additional_context {
            docs.push(additional);
        }

        docs
    }
}
