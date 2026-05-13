# rust-copilot

Agentic pipeline built with Rust and the [GitHub Copilot SDK](https://github.com/github/copilot-sdk).

## Prerequisites

- Rust 1.94.0+ (2024 edition)
- [Copilot CLI](https://github.com/github/copilot-sdk) installed and in PATH
- GitHub Copilot subscription (or BYOK configuration)

## Quick start

```bash
cargo run
```

## What it does

This hello-world pipeline demonstrates the Copilot SDK's agentic capabilities:

1. Starts a Copilot client (spawns the CLI process)
2. Registers a custom `greet` tool
3. Creates a streaming session with the tool
4. Sends a prompt that triggers the agent to call the tool
5. Prints the streamed response

## Architecture

```
main.rs
  ├── GreetTool        — custom ToolHandler returning a greeting
  ├── PrintHandler     — SessionHandler that streams response deltas to stdout
  └── ToolHandlerRouter — routes tool calls to GreetTool, delegates events to PrintHandler
```
