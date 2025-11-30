mod client;
mod system_commands;

use anyhow::{anyhow, Result};
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam, Tool, ErrorData as McpError},
    service::{ServiceExt, RequestContext, RoleServer},
    handler::server::ServerHandler,
    handler::server::tool::schema_for_type,
    model::CallToolRequestMethod,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{Pid, System};
use tracing::{info, error, debug};
use serde_json::Value;

use crate::client::McpClient;
use crate::system_commands::{SystemCommand, LibSystemCommand, KillProcessInput};

// --- Lock File Management ---

#[derive(Serialize, Deserialize, Debug)]
struct LockData {
    pid: u32,
    port: u16,
}

fn get_lock_file_path() -> Result<PathBuf> {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("copilot_mcp_tool.lock");
    Ok(temp_dir)
}

fn read_lock_file() -> Result<LockData> {
    let path = get_lock_file_path()?;
    let content = fs::read_to_string(path)?;
    let data: LockData = serde_json::from_str(&content)?;
    Ok(data)
}

fn server_is_running() -> Option<LockData> {
    if let Ok(data) = read_lock_file() {
        let system = System::new_all();
        if system.process(Pid::from_u32(data.pid)).is_some() {
            return Some(data);
        }
    }
    None
}

// --- Main Command Dispatch ---

fn main_dispatcher() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    if command == Some("run-server-internal") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()? 
            .block_on(run_server())?;
        return Ok(());
    }
    
    let command = command.unwrap_or("status");
    match command {
        "start" => start_server()?,
        "stop" => stop_server()?,
        "status" => show_status()?,
        "list" | "call" => {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()? 
                .block_on(run_client_command(args))?;
        }
        _ => {
             println!("Usage: copilot_mcp_tool [start|stop|status|list|call <tool> [params]]");
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    if env::args().nth(1) != Some("run-server-internal".to_string()) {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();
    }
    
    if let Err(e) = main_dispatcher() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}


// --- Server Commands ---

fn start_server() -> Result<()> {
    if let Some(lock_data) = server_is_running() {
        info!("Server is already running on port {} (PID: {}).", lock_data.port, lock_data.pid);
        return Ok(())
    }

    info!("Starting server in background...");
    let current_exe = env::current_exe()?;
    
    let log_dir = env::temp_dir();
    let stdout_log = fs::File::create(log_dir.join("copilot_mcp_server.stdout.log"))?;
    let stderr_log = fs::File::create(log_dir.join("copilot_mcp_server.stderr.log"))?;

    let mut cmd = Command::new(current_exe);
    cmd.arg("run-server-internal")
       .stdout(stdout_log)
       .stderr(stderr_log);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(DETACHED_PROCESS);
    }
    
    let child = cmd.spawn()?;
    info!("Server process started with PID: {}", child.id());
    println!("Server starting in background...");

    std::thread::sleep(std::time::Duration::from_secs(2));
    show_status()?;

    Ok(())
}

fn stop_server() -> Result<()> {
    if let Some(lock_data) = server_is_running() {
        info!("Stopping server (PID: {})...", lock_data.pid);
        let system = System::new_all();
        if let Some(process) = system.process(Pid::from_u32(lock_data.pid)) {
            process.kill();
            info!("Server stopped.");
        } else {
            error!("Process with PID {} not found, but lock file exists.", lock_data.pid);
        }
        fs::remove_file(get_lock_file_path()?)?;
    } else {
        info!("Server is not running.");
    }
    Ok(())
}

fn show_status() -> Result<()> {
    if let Some(lock_data) = server_is_running() {
        println!("Server is RUNNING on port {} (PID: {}).", lock_data.port, lock_data.pid);
    } else {
        println!("Server is STOPPED.");
    }
    Ok(())
}

// --- Client Commands ---

async fn run_client_command(args: Vec<String>) -> Result<()> {
    let lock_data = server_is_running().ok_or_else(|| anyhow!("Server is not running. Please start it first with 'copilot_mcp_tool start'."))?;
    
    let mut client = McpClient::new();
    client.connect(lock_data.port)?;
    
    debug!("Sending initialize request...");
    client.initialize()?;
    
    debug!("Sending initialized notification...");
    client.initialized_notification()?;
    
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match command {
        "list" => {
            let response = client.list_tools()?;
            println!("{}", serde_json::to_string_pretty(&response.result)?);
        }
        "call" => {
            let tool_name = args.get(2).ok_or_else(|| anyhow!("Tool name is required for 'call'. E.g., 'call echo_message message=\"hello world\"'"))?;
            
            let mut params_map = serde_json::Map::new();
            for arg in args.iter().skip(3) {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = serde_json::Value::String(parts[1].to_string());
                    params_map.insert(key, value);
                } else {
                    return Err(anyhow!("Invalid parameter format: '{}'. Expected 'key=value'.", arg));
                }
            }
            let params = Value::Object(params_map);

            let response = client.call_tool(tool_name, params)?;
            println!("{}", serde_json::to_string_pretty(&response.result)?);
        }
        _ => {}
    }
    
    Ok(())
}


// --- Server Implementation ---

#[derive(Clone)]
pub struct EchoServerTool {
    system_commands: Arc<dyn SystemCommand + Send + Sync>,
}

impl Default for EchoServerTool {
    fn default() -> Self {
        Self::new(Arc::new(LibSystemCommand))
    }
}

impl EchoServerTool {
    pub fn new(system_commands: Arc<dyn SystemCommand + Send + Sync>) -> Self {
        Self { system_commands }
    }
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct EchoInput {
    pub message: String,
}

impl ServerHandler for EchoServerTool {
    fn call_tool(
        &self,
        request_param: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            info!("Server: call_tool received request for: {}", request_param.name.as_ref());
            match request_param.name.as_ref() {
                "echo_message" => {
                    let args_value = request_param.arguments.unwrap_or_default();
                    let input: EchoInput = serde_json::from_value(serde_json::Value::Object(args_value))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    info!("EchoServerTool: Received message: {}", input.message);
                    Ok(CallToolResult::structured(serde_json::to_value(format!("Echoing: {}", input.message)).unwrap()))
                },
                "kill_process" => {
                    let args_value = request_param.arguments.unwrap_or_default();
                    let input: KillProcessInput = serde_json::from_value(serde_json::Value::Object(args_value))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    Ok(self.system_commands.kill_process(input).await)
                },
                _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
            }
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            info!("Server: list_tools called.");
            let tools = vec![
                Tool {
                    name: "echo_message".to_string().into(),
                    description: Some("Echoes a message back".to_string().into()),
                    input_schema: Arc::new(serde_json::to_value(schema_for_type::<EchoInput>()).unwrap().as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    title: None,
                    icons: None,
                    meta: None,
                },
                Tool {
                    name: "kill_process".to_string().into(),
                    description: Some("Kills a process by PID.".to_string().into()),
                    input_schema: Arc::new(serde_json::to_value(schema_for_type::<KillProcessInput>()).unwrap().as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    title: None,
                    icons: None,
                    meta: None,
                },
            ];
            Ok(ListToolsResult::with_all_items(tools))
        }
    }
}

async fn run_server() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    
    let lock_data = LockData {
        pid: std::process::id(),
        port: local_addr.port(),
    };
    let lock_file_path = get_lock_file_path()?;
    fs::write(&lock_file_path, serde_json::to_string_pretty(&lock_data)?)?;
    
    info!("Server listening on {} (PID: {})", local_addr, lock_data.pid);
    debug!("Lock file written to {:?}", lock_file_path);

    let server_impl = EchoServerTool::default();
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        debug!("Accepted connection from: {}", peer_addr);
        let server_clone = server_impl.clone();
        tokio::spawn(async move {
            match server_clone.serve(stream).await {
                Ok(running_service) => {
                    if let Err(e) = running_service.waiting().await {
                        debug!("Error waiting for client {}: {}", peer_addr, e);
                    }
                },
                Err(e) => {
                    debug!("Error serving client {}: {}", peer_addr, e);
                }
            }
            debug!("Client {} disconnected. Finished serving.", peer_addr);
        });
    }
}
