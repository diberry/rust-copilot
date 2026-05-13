use std::sync::Arc;
use std::time::Duration;

use github_copilot_sdk::tool::ToolHandlerRouter;
use github_copilot_sdk::types::{MessageOptions, SessionConfig};
use github_copilot_sdk::{Client, ClientOptions, Error};
use rust_copilot::{GreetTool, PrintHandler};

/// Runs the agentic pipeline. Separated from main so that the caller
/// can ensure client cleanup regardless of success or failure.
async fn run(client: &Client) -> Result<(), Error> {
    // Set up the tool router with our custom greet tool
    let router = ToolHandlerRouter::new(
        vec![Box::new(GreetTool)],
        Arc::new(PrintHandler),
    );
    let tools = router.tools();
    let handler = Arc::new(router);

    // Create a session with tools and streaming enabled
    let config = {
        let mut cfg = SessionConfig::default();
        cfg.tools = Some(tools);
        cfg.streaming = Some(true);
        cfg.with_handler(handler)
    };

    println!("Creating session...");
    let session = client.create_session(config).await?;
    println!("Session {} created.\n", session.id());

    // Send a prompt that should trigger our custom tool
    println!("Sending prompt: \"Please greet Dina using the greet tool.\"\n");
    println!("--- Agent Response ---");
    session
        .send_and_wait(
            MessageOptions::new("Please greet Dina using the greet tool.")
                .with_wait_timeout(Duration::from_secs(60)),
        )
        .await?;
    println!("--- End Response ---\n");

    session.destroy().await?;
    println!("Done! 🎉");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🦀 Rust Copilot SDK — Hello World Agentic Pipeline");
    println!("===================================================\n");

    // Start the Copilot client (spawns the copilot CLI subprocess)
    println!("Starting Copilot client...");
    let client = Client::start(ClientOptions::default()).await?;
    println!("Client started.\n");

    // Run the pipeline, ensuring we stop the child process even on error
    let result = run(&client).await;
    if let Err(e) = client.stop().await {
        eprintln!("[WARN] Failed to stop client subprocess: {e:?}");
    }

    result
}
