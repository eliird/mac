use rig::tool::Tool;
use rig::completion::request::ToolDefinition;
use serde::Deserialize;
use schemars::JsonSchema;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::io::Write;
use std::time::{SystemTime, Duration};
use chrono::{DateTime, Local};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum FileToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("String not found in file")]
    StringNotFound,
}

// FileReader Tool
#[derive(Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    /// Path to the file to read
    path: String,
}

#[derive(Debug, Clone)]
pub struct FileReader;

impl Tool for FileReader {
    const NAME: &'static str = "read_file";
    type Error = FileToolError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read the contents of a file. Returns the entire file content as a string.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("🔧 Tool: read_file | Path: {}", args.path);

        let path = Path::new(&args.path);

        if !path.exists() {
            println!("❌ Tool: read_file | Error: File not found");
            return Err(FileToolError::FileNotFound(args.path));
        }

        match fs::read_to_string(&args.path).await {
            Ok(content) => {
                println!("✅ Tool: read_file | Success: Read {} bytes", content.len());
                Ok(content)
            },
            Err(e) => {
                println!("❌ Tool: read_file | Error: {}", e);
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(FileToolError::PermissionDenied(args.path))
                } else {
                    Err(FileToolError::Io(e))
                }
            }
        }
    }
}

// FileWriter Tool
#[derive(Deserialize, JsonSchema)]
pub struct WriteFileArgs {
    /// Path to the file to write
    path: String,
    /// Content to write to the file
    content: String,
}

#[derive(Debug, Clone)]
pub struct FileWriter;

impl Tool for FileWriter {
    const NAME: &'static str = "write_file";
    type Error = FileToolError;
    type Args = WriteFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("🔧 Tool: write_file | Path: {} | Size: {} bytes", args.path, args.content.len());

        let path = Path::new(&args.path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        match fs::write(&args.path, &args.content).await {
            Ok(_) => {
                println!("✅ Tool: write_file | Success: Wrote {} bytes", args.content.len());
                Ok(format!("Successfully wrote {} bytes to {}", args.content.len(), args.path))
            },
            Err(e) => {
                println!("❌ Tool: write_file | Error: {}", e);
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(FileToolError::PermissionDenied(args.path))
                } else {
                    Err(FileToolError::Io(e))
                }
            }
        }
    }
}

// FileEditor Tool
#[derive(Deserialize, JsonSchema)]
pub struct EditFileArgs {
    /// Path to the file to edit
    path: String,
    /// String to search for in the file
    search: String,
    /// String to replace the search string with
    replace: String,
    /// Replace all occurrences (default: false, only first occurrence)
    #[serde(default)]
    replace_all: bool,
}

// Advanced Code Editor Tool
#[derive(Deserialize, JsonSchema)]
pub struct CodeEditArgs {
    /// Path to the file to edit
    path: String,
    /// Starting line number (1-based) for the edit
    start_line: usize,
    /// Ending line number (1-based) for the edit (inclusive)
    end_line: usize,
    /// New content to replace the specified lines
    new_content: String,
}

#[derive(Debug, Clone)]
pub struct FileEditor;

impl Tool for FileEditor {
    const NAME: &'static str = "edit_file";
    type Error = FileToolError;
    type Args = EditFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Edit a file by replacing text. Can replace first occurrence or all occurrences.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "search": {
                        "type": "string",
                        "description": "String to search for in the file"
                    },
                    "replace": {
                        "type": "string",
                        "description": "String to replace the search string with"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default: false)",
                        "default": false
                    }
                },
                "required": ["path", "search", "replace"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.path.clone()));
        }

        let content = fs::read_to_string(&args.path).await?;

        let new_content = if args.replace_all {
            if !content.contains(&args.search) {
                return Err(FileToolError::StringNotFound);
            }
            content.replace(&args.search, &args.replace)
        } else {
            if let Some(index) = content.find(&args.search) {
                let mut result = String::new();
                result.push_str(&content[..index]);
                result.push_str(&args.replace);
                result.push_str(&content[index + args.search.len()..]);
                result
            } else {
                return Err(FileToolError::StringNotFound);
            }
        };

        fs::write(&args.path, &new_content).await?;

        let replacements = if args.replace_all {
            content.matches(&args.search).count()
        } else {
            1
        };

        Ok(format!("Successfully edited {}. Replaced {} occurrence(s)", args.path, replacements))
    }
}

// Advanced Code Editor Tool
#[derive(Debug, Clone)]
pub struct CodeEditor;

impl Tool for CodeEditor {
    const NAME: &'static str = "edit_code_lines";
    type Error = FileToolError;
    type Args = CodeEditArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Edit specific lines in a code file. Replace lines between start_line and end_line (inclusive) with new content. Perfect for code modifications.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Starting line number (1-based) for the edit",
                        "minimum": 1
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "Ending line number (1-based) for the edit (inclusive)",
                        "minimum": 1
                    },
                    "new_content": {
                        "type": "string",
                        "description": "New content to replace the specified lines"
                    }
                },
                "required": ["path", "start_line", "end_line", "new_content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.path.clone()));
        }

        let content = fs::read_to_string(&args.path).await?;
        let lines: Vec<&str> = content.lines().collect();
        println!("📝 Tool: edit_code_lines | File has {} lines", lines.len());

        if args.start_line == 0 || args.end_line == 0 {
            println!("❌ Tool: edit_code_lines | Error: Invalid line numbers (must be 1-based)");
            return Err(FileToolError::InvalidPath("Line numbers must be 1-based (start from 1)".to_string()));
        }

        if args.start_line > lines.len() || args.end_line > lines.len() {
            return Err(FileToolError::InvalidPath(format!("Line numbers out of range. File has {} lines", lines.len())));
        }

        if args.start_line > args.end_line {
            return Err(FileToolError::InvalidPath("start_line must be <= end_line".to_string()));
        }

        let mut new_lines = Vec::new();

        // Add lines before the edit range
        for line in &lines[0..(args.start_line - 1)] {
            new_lines.push(line.to_string());
        }

        // Add the new content (split by lines)
        for line in args.new_content.lines() {
            new_lines.push(line.to_string());
        }

        // Add lines after the edit range
        for line in &lines[args.end_line..] {
            new_lines.push(line.to_string());
        }

        let new_content = new_lines.join("\n");
        fs::write(&args.path, &new_content).await?;

        let lines_replaced = args.end_line - args.start_line + 1;
        let new_line_count = args.new_content.lines().count();

        Ok(format!("Successfully edited {}. Replaced {} lines ({}:{}) with {} lines",
                   args.path, lines_replaced, args.start_line, args.end_line, new_line_count))
    }
}

// Code Insertion Tool
#[derive(Deserialize, JsonSchema)]
pub struct InsertCodeArgs {
    /// Path to the file to edit
    path: String,
    /// Line number (1-based) where to insert new content (content will be inserted AFTER this line)
    after_line: usize,
    /// Content to insert
    content: String,
}

#[derive(Debug, Clone)]
pub struct CodeInserter;

impl Tool for CodeInserter {
    const NAME: &'static str = "insert_code";
    type Error = FileToolError;
    type Args = InsertCodeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Insert new content after a specific line in a file. Perfect for adding new functions, imports, or code blocks.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "after_line": {
                        "type": "integer",
                        "description": "Line number (1-based) after which to insert content. Use 0 to insert at the beginning of the file.",
                        "minimum": 0
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to insert"
                    }
                },
                "required": ["path", "after_line", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.path.clone()));
        }

        let content = fs::read_to_string(&args.path).await?;
        let lines: Vec<&str> = content.lines().collect();

        if args.after_line > lines.len() {
            return Err(FileToolError::InvalidPath(format!("Line number out of range. File has {} lines", lines.len())));
        }

        let mut new_lines = Vec::new();

        // Add lines up to insertion point
        for (i, line) in lines.iter().enumerate() {
            new_lines.push(line.to_string());

            // Insert new content after the specified line
            if i == args.after_line {
                for new_line in args.content.lines() {
                    new_lines.push(new_line.to_string());
                }
            }
        }

        // If inserting at the beginning (after_line = 0)
        if args.after_line == 0 {
            let mut insertion_lines = Vec::new();
            for new_line in args.content.lines() {
                insertion_lines.push(new_line.to_string());
            }
            insertion_lines.extend(new_lines);
            new_lines = insertion_lines;
        }

        let new_content = new_lines.join("\n");
        fs::write(&args.path, &new_content).await?;

        let inserted_line_count = args.content.lines().count();

        Ok(format!("Successfully inserted {} lines after line {} in {}",
                   inserted_line_count, args.after_line, args.path))
    }
}

// Additional utility tool for creating directories
#[derive(Deserialize, JsonSchema)]
pub struct CreateDirArgs {
    /// Path to the directory to create
    path: String,
}

#[derive(Debug, Clone)]
pub struct CreateDirectory;

impl Tool for CreateDirectory {
    const NAME: &'static str = "create_directory";
    type Error = FileToolError;
    type Args = CreateDirArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Create a directory. Creates parent directories if they don't exist.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to create"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match fs::create_dir_all(&args.path).await {
            Ok(_) => Ok(format!("Successfully created directory: {}", args.path)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(FileToolError::PermissionDenied(args.path))
                } else {
                    Err(FileToolError::Io(e))
                }
            }
        }
    }
}

// List files in directory tool
#[derive(Deserialize, JsonSchema)]
pub struct ListFilesArgs {
    /// Path to the directory to list
    path: String,
    /// Include hidden files (starting with .)
    #[serde(default)]
    include_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct ListFiles;

impl Tool for ListFiles {
    const NAME: &'static str = "list_files";
    type Error = FileToolError;
    type Args = ListFilesArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "List files and directories in a given path.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "Include hidden files (default: false)",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.path));
        }

        if !path.is_dir() {
            return Err(FileToolError::InvalidPath(format!("{} is not a directory", args.path)));
        }

        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if !args.include_hidden && file_name.starts_with('.') {
                    continue;
                }

                let metadata = entry.metadata().await?;
                let prefix = if metadata.is_dir() { "[DIR] " } else { "[FILE]" };
                entries.push(format!("{} {}", prefix, file_name));
            }
        }

        entries.sort();
        Ok(entries)
    }
}

// Codebase Analysis Tool
#[derive(Deserialize, JsonSchema)]
pub struct AnalyzeCodebaseArgs {
    /// Path to the codebase directory to analyze
    pub path: String,
    /// Maximum file size to include in analysis (default: 10000 bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
    /// Maximum directory depth to traverse (default: 10)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_max_file_size() -> usize { 10000 }
fn default_max_depth() -> usize { 10 }

#[derive(Debug, Clone)]
pub struct CodebaseAnalyzer;

impl Tool for CodebaseAnalyzer {
    const NAME: &'static str = "analyze_codebase";
    type Error = FileToolError;
    type Args = AnalyzeCodebaseArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Analyze a codebase directory and return a comprehensive summary including project structure, configuration files, and code content. Perfect for understanding unfamiliar codebases.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the codebase directory to analyze"
                    },
                    "max_file_size": {
                        "type": "integer",
                        "description": "Maximum file size in bytes to include in analysis (default: 10000)",
                        "default": 10000,
                        "minimum": 1000
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum directory depth to traverse (default: 10)",
                        "default": 10,
                        "minimum": 1
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("🔧 Tool: analyze_codebase | Path: {} | Max Size: {} | Max Depth: {}",
                args.path, args.max_file_size, args.max_depth);

        let path = Path::new(&args.path);

        if !path.exists() {
            println!("❌ Tool: analyze_codebase | Error: Directory not found");
            return Err(FileToolError::FileNotFound(args.path.clone()));
        }

        if !path.is_dir() {
            println!("❌ Tool: analyze_codebase | Error: Path is not a directory");
            return Err(FileToolError::InvalidPath(format!("{} is not a directory", args.path)));
        }

        let mut codebase_content = String::new();

        // Common code file extensions to include
        let code_extensions = vec![
            "rs", "py", "js", "ts", "jsx", "tsx", "java", "cpp", "c", "h", "hpp",
            "go", "rb", "php", "cs", "swift", "kt", "scala", "r", "m", "mm",
            "sql", "sh", "bash", "yaml", "yml", "json", "xml", "toml", "ini",
            "md", "txt", "dockerfile", "makefile", "cmake"
        ];

        // Directories to skip
        let skip_dirs = vec![
            ".git", "node_modules", "target", "dist", "build", "out", ".idea",
            ".vscode", "__pycache__", ".pytest_cache", "venv", "env", ".env",
            "vendor", "bower_components", ".next", ".nuxt", "coverage",
            "out-shakespeare-char"
        ];

        codebase_content.push_str(&format!("\n=== CODEBASE ANALYSIS FOR: {} ===\n\n", args.path));

        // Add project structure overview
        codebase_content.push_str("\n=== PROJECT STRUCTURE ===\n");
        let mut dir_structure = String::new();

        for entry in WalkDir::new(path)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                !skip_dirs.iter().any(|dir| e.path().to_string_lossy().contains(dir))
            })
        {
            if let Ok(entry) = entry {
                let depth = entry.depth();
                let indent = "  ".repeat(depth);
                let file_name = entry.file_name().to_string_lossy();

                if entry.file_type().is_dir() {
                    dir_structure.push_str(&format!("{}📁 {}\n", indent, file_name));
                } else if depth <= 2 {
                    dir_structure.push_str(&format!("{}📄 {}\n", indent, file_name));
                }
            }
        }
        codebase_content.push_str(&dir_structure);

        // Read important configuration files
        let config_files = vec![
            "Cargo.toml", "package.json", "requirements.txt", "setup.py",
            "pom.xml", "build.gradle", "CMakeLists.txt", "Makefile",
            "docker-compose.yml", "Dockerfile", ".gitignore", "README.md"
        ];

        for config_file in &config_files {
            let config_path = path.join(config_file);
            if config_path.exists() {
                match fs::read_to_string(&config_path).await {
                    Ok(content) => {
                        codebase_content.push_str(&format!("\n\n=== FILE: {} ===\n", config_file));
                        codebase_content.push_str(&content);
                    }
                    Err(_) => continue,
                }
            }
        }

        // Read actual code files
        codebase_content.push_str("\n\n=== CODE FILES ===\n");

        for entry in WalkDir::new(path)
            .max_depth(args.max_depth)
            .into_iter()
            .filter_entry(|e| {
                !skip_dirs.iter().any(|dir| e.path().to_string_lossy().contains(dir))
            })
        {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let extension = file_path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");

                    let file_name = file_path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("");

                    // Check if it's a code file by extension or by name
                    let is_code_file = code_extensions.contains(&extension.to_lowercase().as_str()) ||
                        file_name.to_lowercase() == "dockerfile" ||
                        file_name.to_lowercase() == "makefile" ||
                        file_name.to_lowercase() == "cmakelists.txt";

                    if is_code_file {
                        match fs::read_to_string(&file_path).await {
                            Ok(content) => {
                                // Only include files under specified size limit
                                if content.len() < args.max_file_size {
                                    let relative_path = file_path.strip_prefix(path)
                                        .unwrap_or(&file_path)
                                        .to_string_lossy();

                                    codebase_content.push_str(&format!("\n\n=== FILE: {} ===\n", relative_path));
                                    codebase_content.push_str(&content);
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        println!("✅ Tool: analyze_codebase | Success: Generated {} characters of analysis", codebase_content.len());
        Ok(codebase_content)
    }
}

// Context Management Tool
#[derive(Deserialize, JsonSchema)]
pub struct SetContextArgs {
    /// Context key/name for the information
    key: String,
    /// Content to store in context
    content: String,
}

#[derive(Debug, Clone)]
pub struct ContextManager {
    pub context: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            context: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn get_all_context(&self) -> String {
        let context = self.context.lock().unwrap();
        if context.is_empty() {
            return "No context stored yet.".to_string();
        }

        let mut result = String::new();
        result.push_str("=== CURRENT CONTEXT ===\n\n");

        for (key, value) in context.iter() {
            result.push_str(&format!("--- {} ---\n{}\n\n", key.to_uppercase(), value));
        }

        result
    }
}

impl Tool for ContextManager {
    const NAME: &'static str = "set_context";
    type Error = FileToolError;
    type Args = SetContextArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Store information in the conversation context for future reference. Perfect for storing codebase analysis, configurations, or any important data that should be remembered.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Name/key for this context (e.g., 'codebase_analysis', 'project_config')"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to store in context"
                    }
                },
                "required": ["key", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut context = self.context.lock().unwrap();
        context.insert(args.key.clone(), args.content.clone());

        Ok(format!("Successfully stored '{}' in context. Context now contains {} items.",
                   args.key, context.len()))
    }
}

// Job Execution Result
#[derive(Debug, Clone)]
pub struct JobResult {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration: Duration,
    pub exit_code: i32,
    pub output_file: String,
}

impl fmt::Display for JobResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Job Execution Result:\n\
             Start Time: {}\n\
             End Time: {}\n\
             Duration: {:?}\n\
             Exit Code: {}\n\
             Output File: {}",
            self.start_time,
            self.end_time,
            self.duration,
            self.exit_code,
            self.output_file
        )
    }
}

// Job Executor Tool
#[derive(Deserialize, JsonSchema)]
pub struct ExecuteJobArgs {
    /// Path to the script to execute
    pub script_path: String,
    /// Optional output file name (defaults to timestamped file)
    #[serde(default)]
    pub output_file: Option<String>,
    /// Working directory for script execution (defaults to script's directory)
    #[serde(default)]
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobExecutor;

impl Tool for JobExecutor {
    const NAME: &'static str = "execute_job";
    type Error = FileToolError;
    type Args = ExecuteJobArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Execute a training job script and capture its output. Supports Python, Shell, JavaScript, Ruby, and Perl scripts. Returns execution results including timing and exit status.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "script_path": {
                        "type": "string",
                        "description": "Path to the script to execute (e.g., './test_code/run.sh', 'train.py')"
                    },
                    "output_file": {
                        "type": "string",
                        "description": "Optional output file name (defaults to timestamped file)"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Working directory for script execution (defaults to script's directory)"
                    }
                },
                "required": ["script_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("🔧 Tool: execute_job | Script: {}", args.script_path);
        if let Some(ref wd) = args.working_directory {
            println!("📁 Tool: execute_job | Working Directory: {}", wd);
        }

        let script_path = Path::new(&args.script_path);

        if !script_path.exists() {
            println!("❌ Tool: execute_job | Error: Script not found");
            return Err(FileToolError::FileNotFound(args.script_path.clone()));
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let output_file = args.output_file.unwrap_or_else(|| format!("job_output_{}.log", timestamp));

        let start_time = Local::now();
        let start_system_time = SystemTime::now();

        // Create output file
        let mut log_file = tokio::fs::File::create(&output_file).await
            .map_err(|e| FileToolError::Io(e))?;

        // Write header
        let header = format!("Job: {}\nStart Time: {}\n----------------------------------------\n",
                           args.script_path, start_time);
        log_file.write_all(header.as_bytes()).await
            .map_err(|e| FileToolError::Io(e))?;

        // Determine executor
        let (program, script_args) = Self::determine_executor(&args.script_path)?;
        println!("⚙️  Tool: execute_job | Executor: {} {:?}", program, script_args);

        // Set working directory
        let working_dir = if let Some(wd) = args.working_directory {
            Path::new(&wd).to_path_buf()
        } else {
            script_path.parent()
                .ok_or_else(|| FileToolError::InvalidPath("Failed to get script directory".to_string()))?
                .to_path_buf()
        };
        println!("📂 Tool: execute_job | Working Directory: {:?}", working_dir);

        // Execute the command
        println!("🚀 Tool: execute_job | Starting execution...");
        let mut child = Command::new(&program)
            .args(&script_args)
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| FileToolError::Io(e))?;

        // Capture output
        let mut output_lines = Vec::new();
        let mut error_lines = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    output_lines.push(format!("[STDOUT] {}", line));
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    error_lines.push(format!("[STDERR] {}", line));
                }
            }
        }

        // Wait for completion
        println!("⏳ Tool: execute_job | Waiting for completion...");
        let exit_status = child.wait()
            .map_err(|e| FileToolError::Io(e))?;

        let end_time = Local::now();
        let duration = start_system_time.elapsed()
            .unwrap_or(Duration::from_secs(0));

        println!("✅ Tool: execute_job | Completed | Duration: {:?} | Exit Code: {}",
                duration, exit_status.code().unwrap_or(-1));

        // Write all output to file
        let mut all_output = output_lines;
        all_output.extend(error_lines);

        for line in &all_output {
            log_file.write_all(format!("{}\n", line).as_bytes()).await
                .map_err(|e| FileToolError::Io(e))?;
        }

        // Write footer
        let footer = format!("----------------------------------------\nEnd Time: {}\nDuration: {:?}\nExit Code: {}\n",
                           end_time, duration, exit_status.code().unwrap_or(-1));
        log_file.write_all(footer.as_bytes()).await
            .map_err(|e| FileToolError::Io(e))?;

        let result = JobResult {
            start_time,
            end_time,
            duration,
            exit_code: exit_status.code().unwrap_or(-1),
            output_file: output_file.clone(),
        };

        let preview: Vec<String> = all_output.iter().rev().take(10).rev().map(|s| s.clone()).collect();
        let summary = format!(
            "Job execution completed!\n\n{}\n\nOutput preview (last 10 lines):\n{}",
            result,
            preview.join("\n")
        );

        Ok(summary)
    }
}

impl JobExecutor {
    fn determine_executor(script_path: &str) -> Result<(String, Vec<String>), FileToolError> {
        let path = Path::new(script_path);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let script_name = path.file_name()
            .ok_or_else(|| FileToolError::InvalidPath("Failed to get script filename".to_string()))?
            .to_str()
            .ok_or_else(|| FileToolError::InvalidPath("Failed to convert filename to string".to_string()))?
            .to_string();

        let (program, args) = match extension {
            "py" => ("python".to_string(), vec![script_name]),
            "sh" => ("bash".to_string(), vec![script_name]),
            "js" => ("node".to_string(), vec![script_name]),
            "rb" => ("ruby".to_string(), vec![script_name]),
            "pl" => ("perl".to_string(), vec![script_name]),
            "" | _ => {
                // Try to execute directly (might be executable)
                (format!("./{}", script_name), vec![])
            }
        };

        Ok((program, args))
    }
}
