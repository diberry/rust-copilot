use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use github_copilot_sdk::handler::{
    HandlerEvent, HandlerResponse, PermissionResult, SessionHandler,
};
use github_copilot_sdk::tool::{
    JsonSchema, ToolHandler, ToolHandlerRouter, schema_for, tool_parameters,
};
use github_copilot_sdk::types::{
    MessageOptions, SessionConfig, SessionEvent, Tool, ToolInvocation, ToolResult,
};
use github_copilot_sdk::{Client, ClientOptions, Error};
use serde::Deserialize;

// --- Custom tool: greet a user by name ---

#[derive(Deserialize, JsonSchema)]
struct GreetParams {
    /// The name of the person to greet
    name: String,
}

struct GreetTool;

#[async_trait]
impl ToolHandler for GreetTool {
    fn tool(&self) -> Tool {
        let mut tool = Tool::default();
        tool.name = "greet".to_string();
        tool.description = "Greet someone by name".to_string();
        tool.parameters = tool_parameters(schema_for::<GreetParams>());
        tool
    }

    async fn call(&self, invocation: ToolInvocation) -> Result<ToolResult, Error> {
        let params: GreetParams = serde_json::from_value(invocation.arguments)?;
        Ok(ToolResult::Text(format!(
            "Hello, {}! Welcome to the Rust Copilot SDK agentic pipeline. 🦀",
            params.name
        )))
    }
}

// --- Event handler that prints streaming responses ---

struct PrintHandler;

#[async_trait]
impl SessionHandler for PrintHandler {
    async fn on_event(&self, event: HandlerEvent) -> HandlerResponse {
        match event {
            HandlerEvent::SessionEvent { event, .. } => {
                print_session_event(&event);
                HandlerResponse::Ok
            }
            HandlerEvent::PermissionRequest { .. } => {
                HandlerResponse::Permission(PermissionResult::Approved)
            }
            _ => HandlerResponse::Ok,
        }
    }
}

fn print_session_event(event: &SessionEvent) {
    match event.event_type.as_str() {
        "assistant.message_delta" => {
            let text = event
                .data
                .get("deltaContent")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            print!("{text}");
            io::stdout().flush().ok();
        }
        "assistant.message" => {
            println!();
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🦀 Rust Copilot SDK — Hello World Agentic Pipeline");
    println!("===================================================\n");

    // 1. Start the Copilot client (spawns the copilot CLI process)
    println!("Starting Copilot client...");
    let client = Client::start(ClientOptions::default()).await?;
    println!("Client started.\n");

    // 2. Set up the tool router with our custom greet tool
    let router = ToolHandlerRouter::new(
        vec![Box::new(GreetTool)],
        Arc::new(PrintHandler),
    );
    let tools = router.tools();
    let handler = Arc::new(router);

    // 3. Create a session with tools and streaming enabled
    let config = {
        let mut cfg = SessionConfig::default();
        cfg.tools = Some(tools);
        cfg.streaming = Some(true);
        cfg.with_handler(handler)
    };

    println!("Creating session...");
    let session = client.create_session(config).await?;
    println!("Session {} created.\n", session.id());

    // 4. Send a prompt that should trigger our custom tool
    println!("Sending prompt: \"Please greet Dina using the greet tool.\"\n");
    println!("--- Agent Response ---");
    session
        .send_and_wait(
            MessageOptions::new("Please greet Dina using the greet tool.")
                .with_wait_timeout(Duration::from_secs(60)),
        )
        .await?;
    println!("--- End Response ---\n");

    // 5. Clean up
    session.destroy().await?;
    println!("Done! 🎉");

    Ok(())
}
