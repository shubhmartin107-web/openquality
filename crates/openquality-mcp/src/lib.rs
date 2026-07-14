//! # OpenQuality MCP
//!
//! Model Context Protocol server for LLM integration.
//!
//! Provides 8 tools that LLMs can invoke:
//! `list_monitors`, `run_monitor`, `list_incidents`, `acknowledge_incident`,
//! `resolve_incident`, `list_workspaces`, `profile_table`, `suggest_expectations`.

pub mod tools;
