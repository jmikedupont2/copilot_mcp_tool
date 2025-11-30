use rmcp::model::CallToolResult;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use async_trait::async_trait;


// --- Input/Output Structs for SystemCommand Trait ---

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct KillProcessInput {
    pub pid: u32,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage_kb: u64,
    pub virtual_memory_usage_kb: u64,
    pub status: String,
    pub parent_pid: Option<u32>,
    // Add more fields as needed
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct ListProcessesOutput {
    pub processes: Vec<ProcessInfo>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct MemoryUsageOutput {
    pub total_memory_kb: u64,
    pub used_memory_kb: u64,
    pub free_memory_kb: u64,
    pub available_memory_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct DiskUsageInfo {
    pub name: String,
    pub total_space_gb: u64,
    pub available_space_gb: u64,
    pub file_system: String,
    pub mount_point: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct DiskUsageOutput {
    pub disks: Vec<DiskUsageInfo>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct PortConnection {
    pub protocol: String, // e.g., "tcp", "udp"
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub status: String, // e.g., "LISTEN", "ESTABLISHED"
    pub pid: Option<u32>, // Process ID, if available
    pub process_name: Option<String>, // Process name, if available
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct ListPortsOutput {
    pub connections: Vec<PortConnection>,
}

// --- SystemCommand Trait Definition ---

#[async_trait]
pub trait SystemCommand: Send + Sync + 'static {
    // Kill a process by PID
    async fn kill_process(&self, input: KillProcessInput) -> CallToolResult;

    // List all running processes
    async fn list_processes(&self) -> CallToolResult;

    // Get overall memory usage
    async fn get_memory_usage(&self) -> CallToolResult;

    // Get disk usage for all mounted filesystems
    async fn get_disk_usage(&self) -> CallToolResult;

    // List all open network ports and connections
    async fn list_ports(&self) -> CallToolResult;
}

// --- LibSystemCommand Implementation (using sysinfo, netstat2) ---
pub struct LibSystemCommand;

#[async_trait]
impl SystemCommand for LibSystemCommand {
    async fn kill_process(&self, input: KillProcessInput) -> CallToolResult {
        // Implement using sysinfo::System::kill_process_by_pid or similar
        // For now, return a placeholder
        CallToolResult::structured_error(
            serde_json::json!({"error": format!("LibSystemCommand::kill_process for PID {} not yet implemented.", input.pid)})
        )
    }

    async fn list_processes(&self) -> CallToolResult {
        // Implement using sysinfo
        CallToolResult::structured_error(
            serde_json::json!({"error": "LibSystemCommand::list_processes not yet implemented."})
        )
    }

    async fn get_memory_usage(&self) -> CallToolResult {
        // Implement using sysinfo
        CallToolResult::structured_error(
            serde_json::json!({"error": "LibSystemCommand::get_memory_usage not yet implemented."})
        )
    }

    async fn get_disk_usage(&self) -> CallToolResult {
        // Implement using sysinfo
        CallToolResult::structured_error(
            serde_json::json!({"error": "LibSystemCommand::get_disk_usage not yet implemented."})
        )
    }

    async fn list_ports(&self) -> CallToolResult {
        // Implement using netstat2
        CallToolResult::structured_error(
            serde_json::json!({"error": "LibSystemCommand::list_ports not yet implemented."})
        )
    }
}

// --- BinSystemCommand Implementation (using external binaries) ---
pub struct BinSystemCommand;

// Helper function to run shell commands (similar to previous run_command)
async fn run_shell_command_bin(command: &str, args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    tokio::process::Command::new(command)
        .args(args)
        .output()
        .await
}

#[async_trait]
impl SystemCommand for BinSystemCommand {
    async fn kill_process(&self, input: KillProcessInput) -> CallToolResult {
        let pid = input.pid;
        let os = std::env::consts::OS;
        let command_result = match os {
            "windows" => {
                run_shell_command_bin("taskkill", &["/PID", &pid.to_string(), "/F"]).await
            },
            "linux" | "macos" => {
                run_shell_command_bin("kill", &["-9", &pid.to_string()]).await
            },
            _ => {
                return CallToolResult::structured_error(
                    serde_json::json!({"error": format!("Unsupported operating system: {}", os)})
                );
            }
        };

        match command_result {
            Ok(output) => {
                if output.status.success() {
                    CallToolResult::structured(
                        serde_json::json!({"message": format!("Process {} killed successfully.", pid)})
                    )
                } else {
                    CallToolResult::structured_error(
                        serde_json::json!({
                            "error": format!("Failed to kill process {}: {}", pid, String::from_utf8_lossy(&output.stderr)),
                            "stdout": String::from_utf8_lossy(&output.stdout),
                            "stderr": String::from_utf8_lossy(&output.stderr),
                        })
                    )
                }
            },
            Err(e) => {
                CallToolResult::structured_error(
                    serde_json::json!({"error": format!("Failed to execute kill command for PID {}: {}", pid, e)})
                )
            }
        }
    }

    async fn list_processes(&self) -> CallToolResult {
        // Implement using platform-specific commands (e.g., 'ps', 'tasklist')
        CallToolResult::structured_error(
            serde_json::json!({"error": "BinSystemCommand::list_processes not yet implemented."})
        )
    }

    async fn get_memory_usage(&self) -> CallToolResult {
        // Implement using platform-specific commands (e.g., 'free', 'wmic OS get FreePhysicalMemory')
        CallToolResult::structured_error(
            serde_json::json!({"error": "BinSystemCommand::get_memory_usage not yet implemented."})
        )
    }

    async fn get_disk_usage(&self) -> CallToolResult {
        // Implement using platform-specific commands (e.g., 'df', 'wmic logicaldisk get Caption,Size,Freespace')
        CallToolResult::structured_error(
            serde_json::json!({"error": "BinSystemCommand::get_disk_usage not yet implemented."})
        )
    }

    async fn list_ports(&self) -> CallToolResult {
        // Implement using platform-specific commands (e.g., 'netstat', 'ss')
        CallToolResult::structured_error(
            serde_json::json!({"error": "BinSystemCommand::list_ports not yet implemented."})
        )
    }
}
