Nargo MCP (Model Context Protocol) Server

This module implements the Model Context Protocol server for Nargo,
providing AI assistants with tools, resources, and prompts for
package management and project operations.

# Features

- **Tools**: Commands for package management, building, testing, etc.
- **Resources**: Access to project configuration, dependencies, and metadata
- **Prompts**: Pre-defined prompts for common Nargo workflows

# Example

```ignore
use nargo_mcp::{NargoMcpHandler, run_stdio};

// Run the MCP server via stdio
run_stdio().await?;
```
