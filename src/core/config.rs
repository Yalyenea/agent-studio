use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub agent_servers: HashMap<String, AgentProcessConfig>,
    #[serde(default = "default_upload_dir")]
    pub upload_dir: PathBuf,
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,
}

fn default_upload_dir() -> PathBuf {
    PathBuf::from(".")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentProcessConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Model configuration for LLM providers
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model_name: String,
}

/// MCP (Model Context Protocol) server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub enabled: bool,
    pub description: String,
    #[serde(default)]
    pub config: HashMap<String, String>,
}

/// Custom command/shortcut configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandConfig {
    pub description: String,
    pub template: String,
}
