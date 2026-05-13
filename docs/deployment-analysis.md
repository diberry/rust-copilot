# Deployment & Runtime Analysis

## How the Copilot SDK Works

The GitHub Copilot SDK (`github-copilot-sdk` v1.0.0-beta.4) is a **client-side SDK** — not a server framework. It:

- Spawns a local `copilot` CLI process
- Communicates via JSON-RPC 2.0 over stdio (Content-Length framed, same as LSP)
- Manages the CLI process lifecycle automatically
- Authenticates via GitHub OAuth tokens or BYOK (Bring Your Own Key)

## Intended Runtime Environments

The SDK is designed for:

| Environment | Fit | Notes |
|------------|-----|-------|
| CLI tools | ✅ Ideal | Terminal apps, developer utilities |
| Desktop apps | ✅ Ideal | Electron, Tauri, native apps |
| CI/CD pipelines | ✅ Good | GitHub Actions, build automation |
| Local agents | ✅ Good | Dev assistants running on your machine |
| Hosted services | ⚠️ Conditional | Requires BYOK mode (see below) |

## Azure Container Apps Considerations

Azure Container Apps is **not the typical deployment target** because:

1. **Authentication model mismatch** — The SDK authenticates via GitHub OAuth (user tokens), not service-to-service credentials
2. **Subprocess requirement** — It spawns `copilot --server --stdio` as a child process, requiring a persistent process model
3. **Session-based, not request/response** — Sessions are long-lived stateful conversations, not HTTP request handlers

### Making it Work with BYOK

If you need hosted deployment, **BYOK (Bring Your Own Key) mode** removes the GitHub auth dependency:

```rust
use github_copilot_sdk::types::ProviderConfig;

let mut provider = ProviderConfig::default();
provider.provider_type = Some("openai".to_string());
provider.base_url = "https://api.openai.com/v1".to_string();
provider.bearer_token = Some(std::env::var("OPENAI_API_KEY")?);

let mut config = SessionConfig::default();
config.model = Some("gpt-4o".to_string());  // Required for BYOK
config.provider = Some(provider);
```

Supported BYOK providers:
- **OpenAI** — `"openai"` type
- **Azure OpenAI** — `"azure"` type
- **Anthropic** — `"anthropic"` type
- **Ollama** — via `"openai"` type (local)
- **Microsoft Foundry Local** — local inference

## Deployment Options Comparison

| Platform | Auth Mode | Pros | Cons |
|----------|-----------|------|------|
| **Local dev machine** | GitHub OAuth | Zero config, just `cargo run` | Single user only |
| **GitHub Actions** | `GITHUB_TOKEN` | CI/CD automation, free tier | Ephemeral, no persistent sessions |
| **Azure Container Apps (BYOK)** | Own API key | Scalable, managed | No GitHub Copilot features, just LLM |
| **Azure Container Apps Jobs** | Own API key | Batch/pipeline workloads | Not long-running |
| **AKS / Azure VM** | GitHub OAuth or BYOK | Full control, persistent | More ops overhead |
| **Docker container** | Either | Portable, reproducible | Needs copilot CLI in image |

## Prerequisites for Any Deployment

1. **Rust 1.94.0+** (2024 edition)
2. **Copilot CLI** — one of:
   - Install separately and ensure it's in `PATH`
   - Use the `embedded-cli` Cargo feature (`COPILOT_CLI_VERSION=1.0.46 cargo build`)
   - Set `COPILOT_CLI_PATH` environment variable
3. **Authentication** — one of:
   - GitHub Copilot subscription + OAuth tokens
   - `GITHUB_TOKEN` / `GH_TOKEN` / `COPILOT_GITHUB_TOKEN` env var
   - BYOK configuration (no subscription needed)

## Recommendation

For this hello-world pipeline, **keep it as a local dev tool**. The SDK is in technical preview (beta) and oriented toward developer tooling. When ready to scale:

1. Start with **BYOK + Azure Container Apps Jobs** for batch workloads
2. Graduate to **AKS** if you need persistent sessions or full GitHub Copilot integration
3. Consider the **embedded-cli** feature for self-contained Docker images
