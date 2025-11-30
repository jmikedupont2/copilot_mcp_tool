# Development Plan and Status for copilot_mcp_tool

This document outlines the current development status, immediate next steps, and the broader vision for the `copilot_mcp_tool` project.

## 1. Current Architectural Status

The `copilot_mcp_tool` has been refactored into a unified client-daemon application written in Rust.

*   **Single Binary (`copilot_mcp_tool`):** Acts as both an MCP server and a command-line client.
    *   **Server Mode:** Managed as a detached background process, storing its PID and port in a lock file.
    *   **Client Mode:** Connects to the running server to execute commands.
*   **Web GUI (`mcp_web_client`):** A separate web frontend that connects to the running `copilot_mcp_tool` server.
*   **Integrated MCP Services:** The `copilot_mcp_tool` currently hosts:
    *   `EchoServerTool`: Provides basic `echo_message` and `kill_process` functionalities.
    *   `ServiceManagerAgent`: A newly added service (Rust-based) intended to manage system services (e.g., Systemd, Windows Services). Currently provides dummy implementations for `list_system_services`, `get_system_service_status`, `start_system_service`, `stop_system_service`, `restart_system_service`.
*   **Submodules Integrated:**
    *   `services/meta-introspector`: Framework for deploying and managing services, often via AWS SSM and Docker.
    *   `services/ai-agent-terraform`: Terraform project for deploying AI agent API infrastructure.
    *   `services/n00b`: Beginner-friendly AI coding starter kit (documentation/guidance focused).

## 2. Immediate Next Steps (Addressing Current Build & Integration Issues)

### 2.1 Resolve `src/main.rs` Compilation Issues

The `src/main.rs` currently fails to compile due to:
*   `main` function conflict: `#[tokio::main]` on `server_main` clashes with the main entry point.
*   `impl Future` return type mismatch in `CombinedServerHandler`: Traits returning `impl Future` are not object safe, leading to compiler confusion when delegating across enum variants.

**Action Plan:**
1.  **Remove `#[tokio::main]` from `async fn server_main()`**: The execution will be handled by `main_dispatcher`'s `block_on` call.
2.  **Convert `ServerHandler` methods to `BoxFuture`**: Modify all methods in `EchoServerTool`, `ServiceManagerAgent`, and `CombinedServerHandler` that return `impl Future` to instead return `BoxFuture<'_, Result<..., McpError>>` or `BoxFuture<'_, ()>`. This requires `use futures::future::{BoxFuture, FutureExt};` and wrapping the async bodies with `.boxed()`.
3.  **Build and Test**: Confirm the build passes and all existing and new tools are callable.

### 2.2 Integrate `ai-ml-zk-ops` Submodule (WSL Approach)

Due to Windows path length limitations causing issues when adding `https://github.com/meta-introspector/ai-ml-zk-ops/` as a submodule directly, the following approach is planned:

**Action Plan (User Manual Steps):**
1.  **Open WSL terminal.**
2.  **Clone `copilot_mcp_tool`** (or navigate to existing clone) *inside WSL*.
3.  **Add `ai-ml-zk-ops` as a submodule *from within WSL***:
    `git submodule add https://github.com/meta-introspector/ai-ml-zk-ops/ services/ai-ml-zk-ops`
4.  **Inspect and shorten excessively long filenames** within the `services/ai-ml-zk-ops` submodule *inside WSL*.
5.  **Commit filename changes** within the `ai-ml-zk-ops` submodule's repository.
6.  **Update `copilot_mcp_tool`'s submodule reference** in WSL.
7.  **Push changes** from WSL to the remote `copilot_mcp_tool` repository.

## 3. Broader Vision & Future Work

The ultimate goal is to evolve `copilot_mcp_tool` into a powerful, extensible controller for managing various services and deployments across different environments.

### 3.1 Enhanced System Management Agent (`ServiceManagerAgent`)

*   **Linux (Systemd) Integration:** Implement actual calls to `systemctl` (or equivalent D-Bus API) for `list_system_services`, `get_system_service_status`, `start_system_service`, `stop_system_service`, `restart_system_service`.
*   **Windows Services Integration:** Add corresponding tools for managing Windows services (e.g., `list_windows_services`, `start_windows_service`, etc.) using platform-specific APIs or external commands (e.g., `sc.exe`).
*   **Service Replication & Orchestration:** Explore tools and patterns for deploying and managing instances of `copilot_mcp_tool` itself, or other services, across multiple servers/VMs. This will tie into the meta-introspector's deployment model.

### 3.2 Remote Execution and Deployment

*   **Native SSH Integration:** Leverage Rust crates (e.g., `ssh2-rs` or `russh`) to enable the `ServiceManagerAgent` to execute commands directly over SSH on remote servers. This would be crucial for deploying and managing services on servers not directly accessible via AWS SSM.
*   **Native Git Integration:** Integrate Git operations (clone, pull, status) within the agent to manage service repositories on remote machines.
*   **Keystore Integration:** Implement secure handling of credentials (SSH keys, API tokens like `GITHUB_TOKEN`, AWS credentials) for remote operations, possibly integrating with a local or remote keystore/secrets manager.
*   **Deployment State Management:** Define mechanisms for tracking the deployment status and configuration of services across multiple target servers, potentially leveraging the `ai-ml-zk-ops` (Zero-Knowledge Ops) submodule for advanced verification or state synchronization.

### 3.3 Leveraging Submodules for Advanced Capabilities

*   **`meta-introspector/services`:** Use its deployment scripts and patterns to build out robust deployment workflows.
*   **`ai-agent-terraform`:** Integrate with this Terraform project to provision and manage infrastructure directly from MCP tool calls. This would enable AI agents to dynamically create and configure cloud resources.
*   **`ai-ml-zk-ops`:** Explore how this submodule can contribute to AI/ML model deployment, zero-knowledge proof operations, and secure, verifiable operational workflows.
*   **`n00b`:** Use the documentation and examples from `n00b` to guide the creation of more user-friendly tooling and setup procedures for new services.

### 3.4 Plugin-based Architecture & New Features

The project is evolving towards a highly extensible, plugin-based architecture, where core functionalities and new features are implemented as dynamically loaded ABI plugins within the `mcpdesk` application. This "systemd-like" approach will allow for flexible management of plugin lifecycles and seamless integration of advanced capabilities.

#### 3.4.1 Core MCP Service Plugin (`mcpdesk_mcp_plugin`)

*   **Objective:** Transform the initial `rustdesk_mcp_service` into a dynamically loaded plugin responsible for exposing core `mcpdesk` (formerly RustDesk) functionalities via MCP.
*   **Key Tasks:**
    *   **ABI Design:** Define a stable C-compatible ABI (Application Binary Interface) for the plugin, including functions for initialization, attachment, detachment, and shutdown.
    *   **`cdylib` Implementation:** Implement the plugin as a Rust `cdylib` (C-compatible dynamic library), exposing the defined ABI.
    *   **MCP Integration:** Wrap relevant `mcpdesk` functionalities and expose them as MCP tools through this plugin.
    *   **Plugin Loading:** Implement dynamic plugin loading and lifecycle management within the `mcpdesk` application (using `libloading` or similar).
    *   **Testing:** Develop and execute comprehensive tests for the plugin's ABI compliance and MCP functionality.

#### 3.4.2 WireGuard Plugin (`mcpdesk_wg_plugin`)

*   **Objective:** Integrate WireGuard VPN functionality as a dynamically loaded plugin to secure connections.
*   **Key Tasks:**
    *   **Research:** Identify suitable Rust crates (e.g., `boringtun`, `wireguard-rs`) for WireGuard integration that support `cdylib` and FFI.
    *   **ABI Design:** Define C-compatible functions for configuration management, tunnel establishment, and status monitoring.
    *   **MCP Commands:** Design MCP tools (e.g., `wg_up`, `wg_down`, `wg_status`, `wg_add_peer`, `wg_remove_peer`, `wg_list_tunnels`) exposed by the plugin.
    *   **Implementation:** Implement `mcpdesk_wg_plugin` as a Rust `cdylib`.
    *   **Integration:** Implement plugin discovery and loading within `mcpdesk` for this plugin.
    *   **Testing:** Develop and execute tests for WireGuard functionality.

#### 3.4.3 OBS Streaming Plugin (`mcpdesk_obs_plugin`)

*   **Objective:** Provide OBS streaming capabilities as a dynamically loaded plugin.
*   **Key Tasks:**
    *   **Research:** Explore Rust bindings for the OBS SDK or a Rust-native headless streaming solution (e.g., based on `gstreamer-rs` or custom WebRTC).
    *   **ABI Design:** Define C-compatible functions for streaming control (start/stop), source selection, quality settings.
    *   **MCP Commands:** Design MCP tools (e.g., `obs_start_stream`, `obs_stop_stream`, `obs_set_source`, `obs_get_status`, `obs_set_settings`) exposed by the plugin.
    *   **Implementation:** Implement `mcpdesk_obs_plugin` as a Rust `cdylib`.
    *   **Integration:** Implement plugin discovery and loading within `mcpdesk` for this plugin.
    *   **Testing:** Develop and execute tests for streaming functionality.

#### 3.4.4 Asciinema Recording Plugin (`mcpdesk_asciinema_plugin`)

*   **Objective:** Implement Asciinema-like text-based terminal recording as a dynamically loaded plugin, avoiding heavy multimedia dependencies.
*   **Key Tasks:**
    *   **Research:** Identify Rust libraries for efficient terminal I/O capture and `.cast` file format generation.
    *   **ABI Design:** Define C-compatible functions for recording control (start/stop, pause, resume), and saving recordings.
    *   **MCP Commands:** Design MCP tools (e.g., `asciinema_start_record`, `asciinema_stop_record`, `asciinema_pause_record`, `asciinema_save_recording`) exposed by the plugin.
    *   **Implementation:** Implement `mcpdesk_asciinema_plugin` as a Rust `cdylib`.
    *   **Integration:** Implement plugin discovery and loading within `mcpdesk` for this plugin.
    *   **Testing:** Develop and execute tests for Asciinema recording functionality.

#### 3.4.5 Nix Packaging

*   **Objective:** Achieve fully reproducible builds and deployments across various environments using Nix.
*   **Key Tasks:**
    *   **Research:** Investigate `naersk` or other Rust-on-Nix tooling.
    *   **Design Nix Expressions:** Create Nix expressions for the `copilot_mcp_tool`, `mcpdesk` (including its plugins), and all their Rust and C/C++ (including `vcpkg`-managed) dependencies.
    *   **Implementation:** Implement the Nix packaging solution.
    *   **Testing:** Develop and execute tests to validate the Nix packages.

