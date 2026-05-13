use std::io::{self, Write};

use async_trait::async_trait;
use github_copilot_sdk::handler::{
    HandlerEvent, HandlerResponse, PermissionResult, SessionHandler,
};
use github_copilot_sdk::tool::{JsonSchema, ToolHandler, schema_for, tool_parameters};
use github_copilot_sdk::types::{SessionEvent, Tool, ToolInvocation, ToolResult};
use github_copilot_sdk::Error;
use serde::Deserialize;

// --- Custom tool: greet a user by name ---

#[derive(Deserialize, JsonSchema)]
pub struct GreetParams {
    /// The name of the person to greet
    pub name: String,
}

pub struct GreetTool;

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

pub struct PrintHandler;

#[async_trait]
impl SessionHandler for PrintHandler {
    async fn on_event(&self, event: HandlerEvent) -> HandlerResponse {
        match event {
            HandlerEvent::SessionEvent { event, .. } => {
                print_session_event(&event);
                HandlerResponse::Ok
            }
            // Security: deny all permission requests by default.
            // Log the request for visibility, but never auto-approve
            // potentially dangerous operations (e.g., shell commands).
            HandlerEvent::PermissionRequest { ref data, .. } => {
                println!(
                    "[SECURITY] Permission DENIED for request: {}",
                    serde_json::to_string(&data).unwrap_or_else(|_| "unknown".into())
                );
                HandlerResponse::Permission(PermissionResult::Denied)
            }
            _ => HandlerResponse::Ok,
        }
    }
}

pub fn print_session_event(event: &SessionEvent) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use github_copilot_sdk::tool::ToolHandler;
    use serde_json::json;

    // --- GreetTool::tool() tests ---

    #[test]
    fn greet_tool_returns_correct_name() {
        let tool = GreetTool.tool();
        assert_eq!(tool.name, "greet");
    }

    #[test]
    fn greet_tool_returns_correct_description() {
        let tool = GreetTool.tool();
        assert_eq!(tool.description, "Greet someone by name");
    }

    #[test]
    fn greet_tool_has_parameters() {
        let tool = GreetTool.tool();
        assert!(!tool.parameters.is_empty());
        // Should have a "type" key with value "object"
        assert_eq!(tool.parameters.get("type").unwrap(), "object");
    }

    // --- GreetTool::call() tests ---

    fn make_tool_invocation(args: serde_json::Value) -> ToolInvocation {
        serde_json::from_value(json!({
            "sessionId": "test-session",
            "toolCallId": "call-1",
            "toolName": "greet",
            "arguments": args
        }))
        .expect("failed to construct ToolInvocation")
    }

    #[tokio::test]
    async fn greet_call_returns_greeting_for_name() {
        let invocation = make_tool_invocation(json!({"name": "Dina"}));
        let result = GreetTool.call(invocation).await.unwrap();
        match result {
            ToolResult::Text(text) => {
                assert!(text.contains("Hello, Dina!"));
                assert!(text.contains("🦀"));
            }
            _ => panic!("Expected ToolResult::Text"),
        }
    }

    #[tokio::test]
    async fn greet_call_returns_greeting_for_different_name() {
        let invocation = make_tool_invocation(json!({"name": "Ferris"}));
        let result = GreetTool.call(invocation).await.unwrap();
        match result {
            ToolResult::Text(text) => {
                assert!(text.contains("Hello, Ferris!"));
            }
            _ => panic!("Expected ToolResult::Text"),
        }
    }

    #[tokio::test]
    async fn greet_call_with_empty_name() {
        let invocation = make_tool_invocation(json!({"name": ""}));
        let result = GreetTool.call(invocation).await.unwrap();
        match result {
            ToolResult::Text(text) => {
                assert!(text.contains("Hello, !"));
            }
            _ => panic!("Expected ToolResult::Text"),
        }
    }

    // --- PrintHandler::on_event() tests ---

    fn make_session_event(event_type: &str, data: serde_json::Value) -> SessionEvent {
        serde_json::from_value(json!({
            "id": "evt-1",
            "timestamp": "2026-01-01T00:00:00Z",
            "type": event_type,
            "data": data
        }))
        .expect("failed to construct SessionEvent")
    }

    #[tokio::test]
    async fn print_handler_session_event_returns_ok() {
        let session_event = make_session_event("assistant.message_delta", json!({}));
        let handler_event = HandlerEvent::SessionEvent {
            session_id: "test-session".to_string().into(),
            event: session_event,
        };
        let response = PrintHandler.on_event(handler_event).await;
        assert!(matches!(response, HandlerResponse::Ok));
    }

    #[tokio::test]
    async fn print_handler_permission_request_returns_denied() {
        let handler_event = HandlerEvent::PermissionRequest {
            session_id: "test-session".to_string().into(),
            request_id: "req-1".to_string().into(),
            data: serde_json::from_value(json!({
                "tool": "bash",
                "command": "ls"
            }))
            .unwrap(),
        };
        let response = PrintHandler.on_event(handler_event).await;
        assert!(matches!(
            response,
            HandlerResponse::Permission(PermissionResult::Denied)
        ));
    }

    #[tokio::test]
    async fn print_handler_denies_dangerous_permission_request() {
        let handler_event = HandlerEvent::PermissionRequest {
            session_id: "test-session".to_string().into(),
            request_id: "req-2".to_string().into(),
            data: serde_json::from_value(json!({
                "tool": "bash",
                "command": "rm -rf /"
            }))
            .unwrap(),
        };
        let response = PrintHandler.on_event(handler_event).await;
        assert!(matches!(
            response,
            HandlerResponse::Permission(PermissionResult::Denied)
        ));
    }

    // --- print_session_event tests ---

    #[test]
    fn print_session_event_handles_message_delta() {
        // This won't panic and processes the delta content
        let event = make_session_event(
            "assistant.message_delta",
            json!({"deltaContent": "hello"}),
        );
        print_session_event(&event);
    }

    #[test]
    fn print_session_event_handles_message_delta_no_content() {
        let event = make_session_event("assistant.message_delta", json!({}));
        print_session_event(&event);
    }

    #[test]
    fn print_session_event_handles_message() {
        let event = make_session_event("assistant.message", json!({}));
        print_session_event(&event);
    }

    #[test]
    fn print_session_event_handles_unknown_type() {
        let event = make_session_event("unknown.event", json!({}));
        // Should not panic — the catch-all arm does nothing
        print_session_event(&event);
    }
}
