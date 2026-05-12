//! Dracon TUI Preview LSP Server
//!
//! This LSP server compiles and runs Dracon examples, streaming terminal output
//! back to the VSCode extension for live preview.

use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

// ============================================================================
// Types and Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "processId")]
    pub process_id: Option<u32>,
    #[serde(rename = "rootUri")]
    pub root_uri: Option<String>,
    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(rename = "workspace")]
    pub workspace: Option<WorkspaceCapability>,
    #[serde(rename = "textDocument")]
    pub text_document: Option<TextDocumentCapability>,
    #[serde(rename = "window")]
    pub window: Option<WindowCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCapability {
    #[serde(rename = "applyEdit")]
    pub apply_edit: Option<bool>,
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentCapability {
    pub synchronization: Option<TextDocumentSyncCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentSyncCapability {
    #[serde(rename = "willSave")]
    pub will_save: Option<bool>,
    #[serde(rename = "didSave")]
    pub did_save: Option<bool>,
    #[serde(rename = "willSaveWaitUntil")]
    pub will_save_wait_until: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowCapability {
    #[serde(rename = "workDoneProgress")]
    pub work_done_progress: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: Option<TextDocumentSyncOptions>,
    #[serde(rename = "hoverProvider")]
    pub hover_provider: Option<bool>,
    #[serde(rename = "completionProvider")]
    pub completion_provider: Option<CompletionOptions>,
    #[serde(rename = "signatureHelpProvider")]
    pub signature_help_provider: Option<SignatureHelpOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentSyncOptions {
    pub open_close: bool,
    pub save: Option<SaveOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveOptions {
    pub include_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub resolve_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpOptions {
    pub trigger_characters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentDidChangeParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<i32>,
    pub code: Option<Value>,
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

// LSP Message Types
#[derive(Debug)]
pub enum LspMessage {
    Request(Request),
    Response(Response),
    Notification(Notification),
    ParseError,
}

#[derive(Debug)]
pub struct Request {
    pub id: Value,
    pub method: String,
    pub params: Value,
}

#[derive(Debug)]
pub struct Response {
    pub id: Value,
    pub result: Option<Value>,
    pub error: Option<LspError>,
}

#[derive(Debug)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspError {
    pub code: i32,
    pub message: String,
}

// Preview Events for streaming
#[derive(Debug, Clone, Serialize)]
pub struct PreviewEvent {
    pub event: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum PreviewState {
    Idle,
    Compiling,
    Running,
    Error,
}

// ============================================================================
// LSP Server Implementation
// ============================================================================

struct LspServer {
    capabilities: ServerCapabilities,
    documents: HashMap<String, String>,
    workspace_root: Option<PathBuf>,
    example_path: Option<String>,
    preview_process: Option<Child>,
    preview_state: PreviewState,
    content_queue: Arc<Mutex<Vec<String>>>,
}

impl LspServer {
    fn new() -> Self {
        Self {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncOptions {
                    open_close: true,
                    save: Some(SaveOptions { include_text: true }),
                }),
                hover_provider: Some(true),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                }),
            },
            documents: HashMap::new(),
            workspace_root: None,
            example_path: None,
            preview_process: None,
            preview_state: PreviewState::Idle,
            content_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn initialize(&mut self, params: InitializeParams) -> InitializeResult {
        // Extract workspace root
        if let Some(ref uri) = params.root_uri {
            if let Ok(path) = uri.replace("file://", "").parse::<PathBuf>() {
                self.workspace_root = Some(path);
            }
        }

        InitializeResult {
            capabilities: self.capabilities.clone(),
        }
    }

    fn handle_text_document_did_open(&mut self, params: TextDocumentItem) {
        self.documents.insert(params.uri.clone(), params.text);
    }

    fn handle_text_document_did_change(&mut self, params: TextDocumentDidChangeParams) {
        if let Some(doc) = self.documents.get_mut(&params.text_document.uri) {
            // Apply all content changes
            for change in params.content_changes {
                *doc = change.text;
            }
        }
    }

    fn handle_text_document_did_save(&mut self, _uri: &str) {
        // Trigger recompilation on save
        self.trigger_preview_rebuild();
    }

    fn trigger_preview_rebuild(&mut self) {
        // Clone workspace root before stopping preview
        let root = self.workspace_root.clone();

        // Stop any existing preview
        self.stop_preview();

        // Start new compilation
        if let Some(ref root) = root {
            self.start_preview_compilation(root);
        }
    }

    fn start_preview_compilation(&mut self, workspace_root: &PathBuf) {
        self.preview_state = PreviewState::Compiling;

        // Emit compilation started event
        let event = PreviewEvent {
            event: "compile_start".to_string(),
            data: "Starting compilation...".to_string(),
        };
        self.queue_event(event);

        // Compile the example in a separate thread
        let root = workspace_root.clone();
        let example_path = self.example_path.clone();
        let queue = Arc::clone(&self.content_queue);

        thread::spawn(move || {
            // Determine the example to compile
            let example = example_path
                .as_ref()
                .map(|p| {
                    PathBuf::from(p)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("showcase")
                        .to_string()
                })
                .unwrap_or_else(|| "showcase".to_string());

            // Run cargo build
            let mut cmd = Command::new("cargo");
            cmd.arg("build")
                .arg("--example")
                .arg(&example)
                .arg("--quiet")
                .current_dir(&root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = cmd.output();

            match output {
                Ok(result) => {
                    let queue_clone = Arc::clone(&queue);
                    let mut runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    runtime.block_on(async {
                        let mut q = queue_clone.lock().await;
                        if result.status.success() {
                            q.push(serde_json::to_string(&PreviewEvent {
                                event: "compile_success".to_string(),
                                data: format!("Compiled successfully: {}", example),
                            }).unwrap());
                        } else {
                            let stderr = String::from_utf8_lossy(&result.stderr);
                            q.push(serde_json::to_string(&PreviewEvent {
                                event: "compile_error".to_string(),
                                data: stderr.to_string(),
                            }).unwrap());
                        }
                    });
                }
                Err(e) => {
                    let queue_clone = Arc::clone(&queue);
                    let mut runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    runtime.block_on(async {
                        let mut q = queue_clone.lock().await;
                        q.push(serde_json::to_string(&PreviewEvent {
                            event: "compile_error".to_string(),
                            data: format!("Failed to run cargo: {}", e),
                        }).unwrap());
                    });
                }
            }
        });
    }

    fn start_preview_run(&mut self, workspace_root: &PathBuf) {
        self.preview_state = PreviewState::Running;

        let example = self.example_path
            .as_ref()
            .map(|p| {
                PathBuf::from(p)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("showcase")
                    .to_string()
            })
            .unwrap_or_else(|| "showcase".to_string());

        let binary_path = workspace_root.join("target").join("debug").join("examples").join(&example);

        // Get theme from environment
        let theme = env::var("DTRON_THEME").unwrap_or_else(|_| "nord".to_string());

        // Spawn the binary
        match Command::new(&binary_path)
            .env("DTRON_THEME", &theme)
            .env("DTRON_NO_MOUSE", "1") // Disable mouse to simplify output
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                self.preview_process = Some(child);

                // Capture stdout in a thread
                let queue = Arc::clone(&self.content_queue);
                if let Some(stdout) = self.preview_process.as_mut().and_then(|p| p.stdout.take()) {
                    let reader = BufReader::new(stdout);
                    thread::spawn(move || {
                        let mut runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();

                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let queue_clone = Arc::clone(&queue);
                                runtime.block_on(async {
                                    let mut q = queue_clone.lock().await;
                                    q.push(serde_json::to_string(&PreviewEvent {
                                        event: "output".to_string(),
                                        data: line,
                                    }).unwrap());
                                });
                            }
                        }
                    });
                }

                // Capture stderr
                let queue = Arc::clone(&self.content_queue);
                if let Some(stderr) = self.preview_process.as_mut().and_then(|p| p.stderr.take()) {
                    let reader = BufReader::new(stderr);
                    thread::spawn(move || {
                        let mut runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();

                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let queue_clone = Arc::clone(&queue);
                                runtime.block_on(async {
                                    let mut q = queue_clone.lock().await;
                                    q.push(serde_json::to_string(&PreviewEvent {
                                        event: "error".to_string(),
                                        data: format!("\x1b[31m{}\x1b[0m", line),
                                    }).unwrap());
                                });
                            }
                        }
                    });
                }

                // Monitor process exit
                let queue = Arc::clone(&self.content_queue);
                let process_id = self.preview_process.as_mut().map(|p| p.id());
                thread::spawn(move || {
                    // Wait for process
                    thread::sleep(Duration::from_millis(100));
                    if let Some(pid) = process_id {
                        let queue_clone = Arc::clone(&queue);
                        let mut runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();

                        runtime.block_on(async {
                            let mut q = queue_clone.lock().await;
                            q.push(serde_json::to_string(&PreviewEvent {
                                event: "exit".to_string(),
                                data: format!("Process {} exited", pid),
                            }).unwrap());
                        });
                    }
                });
            }
            Err(e) => {
                let event = PreviewEvent {
                    event: "error".to_string(),
                    data: format!("Failed to start preview: {}", e),
                };
                self.queue_event(event);
                self.preview_state = PreviewState::Error;
            }
        }
    }

    fn stop_preview(&mut self) {
        if let Some(ref mut child) = self.preview_process {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.preview_process = None;
        self.preview_state = PreviewState::Idle;

        let event = PreviewEvent {
            event: "stopped".to_string(),
            data: "Preview stopped".to_string(),
        };
        self.queue_event(event);
    }

    fn queue_event(&self, event: PreviewEvent) {
        let queue = Arc::clone(&self.content_queue);
        let mut runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let mut q = queue.lock().await;
            q.push(serde_json::to_string(&event).unwrap());
        });
    }

    fn process_queue(&self) -> Vec<String> {
        let queue = Arc::clone(&self.content_queue);
        let mut runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let mut q = queue.lock().await;
            let events = q.drain(..).collect();
            events
        })
    }
}

// ============================================================================
// JSON-RPC Protocol Handler
// ============================================================================

struct JsonRpcHandler {
    server: LspServer,
}

impl JsonRpcHandler {
    fn new() -> Self {
        Self {
            server: LspServer::new(),
        }
    }

    fn read_message<R: Read + BufRead>(&self, reader: &mut R) -> Option<LspMessage> {
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut content_length: Option<usize> = None;

        // Read headers
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).ok()? == 0 {
                return None;
            }
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().to_string();
                let value = line[colon + 1..].trim().to_string();
                let value_lower = key.to_lowercase();
                headers.insert(value_lower.clone(), value.clone());
                if value_lower == "content-length" {
                    content_length = value.parse().ok();
                }
            }
        }

        let length = content_length?;

        // Read content
        let mut content = vec![0u8; length];
        reader.read_exact(&mut content).ok()?;
        let content_str = String::from_utf8(content).ok()?;

        // Parse JSON
        let json: Value = serde_json::from_str(&content_str).ok()?;

        // Determine message type
        if json.get("id").is_some() {
            if json.get("method").is_some() {
                // Request
                Some(LspMessage::Request(Request {
                    id: json["id"].clone(),
                    method: json["method"].as_str()?.to_string(),
                    params: json["params"].clone(),
                }))
            } else {
                // Response
                Some(LspMessage::Response(Response {
                    id: json["id"].clone(),
                    result: json.get("result").cloned(),
                    error: json.get("error").and_then(|e| {
                        e.get("code").and_then(|c| {
                            let code = c.as_i64().unwrap_or(0) as i32;
                            e.get("message").and_then(|m| {
                                m.as_str().map(|msg| LspError {
                                    code,
                                    message: msg.to_string(),
                                })
                            })
                        })
                    }),
                }))
            }
        } else if json.get("method").is_some() {
            // Notification
            Some(LspMessage::Notification(Notification {
                method: json["method"].as_str()?.to_string(),
                params: json["params"].clone(),
            }))
        } else {
            Some(LspMessage::ParseError)
        }
    }

    fn write_message<W: Write>(&self, writer: &mut W, message: &Value) -> io::Result<()> {
        let content = serde_json::to_string(message).unwrap_or_default();
        writeln!(writer, "Content-Length: {}\r", content.len())?;
        writeln!(writer)?;
        writer.write_all(content.as_bytes())?;
        writer.flush()
    }

    fn handle_message(&mut self, message: LspMessage) -> Option<Value> {
        match message {
            LspMessage::Request(request) => {
                let result: Option<Value> = match request.method.as_str() {
                    "initialize" => {
                        let params: InitializeParams =
                            serde_json::from_value(request.params).ok()?;
                        let result = self.server.initialize(params);
                        serde_json::to_value(result).ok()
                    }
                    "shutdown" => Some(serde_json::Value::Null),
                    "dracon/startPreview" => {
                        // Custom command to start preview
                        let root = self.server.workspace_root.clone();
                        if let Some(ref root) = root {
                            let root = root.clone();
                            self.server.start_preview_compilation(&root);
                        }
                        Some(serde_json::Value::Null)
                    }
                    "dracon/stopPreview" => {
                        self.server.stop_preview();
                        Some(serde_json::Value::Null)
                    }
                    "dracon/getEvents" => {
                        // Custom method to get queued events
                        let events = self.server.process_queue();
                        Some(serde_json::to_value(events).unwrap_or_default())
                    }
                    "textDocument/hover" => {
                        // Simple hover implementation
                        Some(serde_json::json!({
                            "contents": {
                                "kind": "markdown",
                                "value": "**Dracon TUI Preview**\n\nA live preview for Dracon Terminal Engine examples."
                            }
                        }))
                    }
                    _ => None,
                };

                if let Some(result) = result {
                    Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request.id,
                        "result": result
                    }))
                } else {
                    Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request.id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {}", request.method)
                        }
                    }))
                }
            }
            LspMessage::Notification(notification) => {
                match notification.method.as_str() {
                    "initialized" => {
                        // Client initialized
                    }
                    "textDocument/didOpen" => {
                        if let Ok(params) =
                            serde_json::from_value::<TextDocumentItem>(notification.params)
                        {
                            self.server.handle_text_document_did_open(params);
                        }
                    }
                    "textDocument/didChange" => {
                        if let Ok(params) =
                            serde_json::from_value::<TextDocumentDidChangeParams>(
                                notification.params,
                            )
                        {
                            self.server.handle_text_document_did_change(params);
                        }
                    }
                    "textDocument/didSave" => {
                        if let Some(uri) = notification.params.get("textDocument").and_then(|d| d.get("uri")).and_then(|u| u.as_str()) {
                            self.server.handle_text_document_did_save(uri);
                        }
                    }
                    "textDocument/didClose" => {
                        if let Some(uri) = notification.params.get("textDocument").and_then(|d| d.get("uri")).and_then(|u| u.as_str()) {
                            self.server.documents.remove(uri);
                        }
                    }
                    "workspace/didChangeConfiguration" => {
                        // Handle configuration changes
                    }
                    _ => {}
                }
                None
            }
            LspMessage::Response(_) => None,
            LspMessage::ParseError => {
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    }
                }))
            }
        }
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let mut use_stdin = false;
    let mut workspace_root: Option<PathBuf> = None;
    let mut example_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--stdio" => use_stdin = true,
            "--workspace-root" => {
                i += 1;
                if i < args.len() {
                    workspace_root = Some(PathBuf::from(&args[i]));
                }
            }
            "--example" => {
                i += 1;
                if i < args.len() {
                    example_path = Some(args[i].clone());
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Create LSP handler
    let mut handler = JsonRpcHandler::new();

    // Set workspace root and example path if provided
    if let Some(root) = workspace_root {
        handler.server.workspace_root = Some(root);
    }
    if let Some(example) = example_path {
        handler.server.example_path = Some(example);
    }

    if use_stdin {
        // Run in LSP mode
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        loop {
            match handler.read_message(&mut reader) {
                Some(message) => {
                    if let LspMessage::Request(ref req) = message {
                        if req.method == "shutdown" {
                            // Send shutdown response
                            let response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "result": null
                            });
                            handler.write_message(&mut writer, &response).ok();
                            break;
                        }
                    }

                    if let Some(response) = handler.handle_message(message) {
                        handler.write_message(&mut writer, &response).ok();
                    }

                    // Send any queued events
                    let events = handler.server.process_queue();
                    for event in events {
                        if let Ok(event_json) = serde_json::from_str::<Value>(&event) {
                            let notification = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "dracon/event",
                                "params": event_json
                            });
                            handler.write_message(&mut writer, &notification).ok();
                        }
                    }
                }
                None => break,
            }
        }
    } else {
        // Interactive mode - show help
        eprintln!("Dracon TUI Preview LSP Server");
        eprintln!();
        eprintln!("Usage: dracon-lsp [options]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --stdio             Use JSON-RPC over stdio (LSP mode)");
        eprintln!("  --workspace-root    Set the workspace root directory");
        eprintln!("  --example <path>    Set the example file to preview");
        eprintln!();
        eprintln!("Environment:");
        eprintln!("  DTRON_THEME         Set the theme for the preview (default: nord)");
        eprintln!("  DTRON_NO_MOUSE      Disable mouse input in preview");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_event_serialization() {
        let event = PreviewEvent {
            event: "output".to_string(),
            data: "Hello, World!".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("output"));
        assert!(json.contains("Hello, World!"));
    }

    #[test]
    fn test_lsp_server_creation() {
        let server = LspServer::new();
        assert!(matches!(server.preview_state, PreviewState::Idle));
        assert!(server.documents.is_empty());
    }
}
