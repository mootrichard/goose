# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Goose is a local, extensible, open source AI agent that automates engineering tasks. It's built in Rust with a multi-crate workspace architecture and includes both CLI and desktop UI interfaces. The project supports multiple LLM providers and uses an extension system for adding capabilities.

## Development Commands

### Building and Running
- `just release-binary` - Build release binaries and copy to UI directory
- `just run-ui` - Build and run the desktop UI
- `just run-server` - Run the goose server
- `cargo build` - Build debug version
- `cargo build --release` - Build release version
- `cargo test` - Run tests
- `cargo clippy` - Run linting

### Cross-platform Building
- `just release-windows` - Build Windows executable using Docker
- `just release-intel` - Build for Intel Mac
- `just make-ui-windows` - Build Windows desktop package
- `just make-ui-intel` - Build Intel Mac desktop package

### Development Utilities
- `just run-dev` - Build debug and run UI
- `just install-deps` - Install all dependencies (run once after clone)
- `just generate-openapi` - Generate OpenAPI schema and frontend API
- `just check-openapi-schema` - Verify OpenAPI schema is up-to-date
- `just run-docs` - Run Docusaurus documentation server

### Testing and Quality
- `cargo test --workspace` - Run all tests
- `just lint-ui` - Lint the UI code
- `cd ui/desktop && npm run lint:check` - Check UI linting

## Architecture

### Core Structure
The project is organized as a Rust workspace with these main crates:

- **goose** - Core library with agents, providers, extensions, and scheduling
- **goose-cli** - Command-line interface
- **goose-server** - HTTP server for desktop UI
- **goose-mcp** - MCP (Model Context Protocol) extensions
- **goose-bench** - Benchmarking and evaluation tools
- **mcp-client**, **mcp-core**, **mcp-server** - MCP infrastructure

### Extension System
Goose uses an extension-based architecture where capabilities are added through extensions that provide:
- Tools (functions the AI can call)
- Prompts and system messages
- State management
- Dependencies on other extensions

Extensions are configured in profiles and can depend on each other. The core "developer" extension provides file operations, shell commands, and planning capabilities.

### Key Components
- **Agents** (`crates/goose/src/agents/`) - Core agent logic, tool execution, context management
- **Providers** (`crates/goose/src/providers/`) - LLM provider integrations (OpenAI, Anthropic, etc.)
- **Extensions** - Plugin system for adding capabilities
- **Scheduler** - Task scheduling and workflow management
- **MCP Integration** - Model Context Protocol for tool connectivity

### Desktop UI
The desktop application is built with Electron/Tauri located in `ui/desktop/`. It communicates with the Rust backend via HTTP API and provides a graphical interface for interacting with goose.

## File Patterns

### Configuration
- Profiles define model configuration and extensions in YAML format
- Extensions are registered in `crates/goose/src/config/extensions.rs`
- Provider configurations in `crates/goose/src/providers/`

### Adding New Providers
1. Create provider module in `crates/goose/src/providers/`
2. Implement the `LanguageModelProvider` trait
3. Add provider to factory in `crates/goose/src/providers/factory.rs`
4. Add configuration options to relevant config structs

### Adding New Extensions
1. Create extension module in `crates/goose-mcp/src/` or appropriate location
2. Implement the `Extension` trait with tools marked with `#[tool]`
3. Register extension in the extension manager
4. Add configuration if needed

## Testing

- Run `cargo test --workspace` for all Rust tests
- Benchmarking available via `just` commands for `goose-bench`
- UI tests in `ui/desktop/tests/`
- Integration tests in various `tests/` directories

## Development Notes

- The project uses `just` as a command runner (Justfile contains all build commands)
- Cross-compilation is handled via Docker for Windows builds
- OpenAPI schema must be kept in sync - run `just check-openapi-schema` before commits
- UI development requires Node.js dependencies: `cd ui/desktop && npm install`
- Documentation is built with Docusaurus in the `documentation/` directory