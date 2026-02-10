# xAI SDK

A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services including Grok language models, embeddings, image generation, and more.

## Features

- **Complete API Coverage**: Full gRPC client implementation for all xAI services
- **Type Safety**: Auto-generated Rust types from Protocol Buffers
- **Multiple Models**: Support for all xAI language models (Grok-3, Grok-4, etc.)
- **Streaming Support**: Real-time streaming for chat completions and text generation
- **Response Assembly**: Convert streaming chunks into complete responses
- **Secure**: TLS encryption with automatic certificate validation

## Quick Start

### Prerequisites

1. Rust 1.88+ installed
2. xAI API key

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xai-sdk = "0.8.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Running the Examples

1. Set your API key as an environment variable:
   ```bash
   export XAI_API_KEY="your-api-key-here"
   ```

2. Run the authentication info example:
   ```bash
   cargo run --example auth_info
   ```

3. Run the raw text sampling example:
   ```bash
   cargo run --example raw_text_sample
   ```

4. Run the chat completion example (supports multiple modes):
   ```bash
   # Blocking completion
   cargo run --example chat -- --complete
   
   # Streaming completion
   cargo run --example chat -- --stream
   
   # Streaming with assembly
   cargo run --example chat -- --assemble
   ```

5. Run the multi-client example (demonstrates using multiple services with shared channel):
   ```bash
   cargo run --example multi_client
   ```

6. Run the interceptor composition example:
   ```bash
   cargo run --example interceptor_compose
   ```

7. Run the tool calls example:
   ```bash
   cargo run --example tool_calls
   ```

## API Services

The SDK provides clients for all xAI services:

### Chat Service
- **`get_completion`** - Get chat completion
- **`get_completion_chunk`** - Stream chat completion in chunks
- **`start_deferred_completion`** - Start deferred chat completion
- **`get_deferred_completion`** - Retrieve deferred completion
- **`get_stored_completion`** - Get stored chat completion
- **`delete_stored_completion`** - Delete stored chat completion

### Sample Service
- **`sample_text`** - Raw text generation
- **`sample_text_streaming`** - Streaming text generation

### Models Service
- **`list_language_models`** - List available language models
- **`list_embedding_models`** - List embedding models
- **`list_image_generation_models`** - List image generation models

### Embed Service
- **`embed`** - Generate embeddings from text or images

### Image Service
- **`generate_image`** - Create images from text prompts

### Video Service
- **`generate_video`** - Create videos with deferred processing
- **`get_deferred_video`** - Retrieve generated videos

### Auth Service
- **`get_api_key_info`** - Get API key information

### Billing Service
- **`set_billing_info`** - Set billing information for a team
- **`get_billing_info`** - Get billing information for a team
- **`list_payment_methods`** - List payment methods on file
- **`set_default_payment_method`** - Set default payment method
- **`get_amount_to_pay`** - Preview current billing period amount
- **`analyze_billing_items`** - Analyze historical billing usage
- **`list_invoices`** - List invoices for a team
- **`list_prepaid_balance_changes`** - List prepaid credit balance changes
- **`top_up_or_get_existing_pending_change`** - Top up prepaid credits
- **`get_spending_limits`** - Get spending limits
- **`set_soft_spending_limit`** - Set soft spending limit

## Client Modules

The SDK is organized into focused modules, each providing easy client creation:

### Available Modules
- **`auth`** - Authentication services
- **`billing`** - Billing and payment management
- **`chat`** - Chat completions and streaming
- **`documents`** - Document processing
- **`embed`** - Text and image embeddings
- **`image`** - Image generation
- **`models`** - Model listing and information
- **`sample`** - Text sampling and generation
- **`tokenize`** - Text tokenization
- **`video`** - Video generation with deferred processing

### Complete Example
Here's a complete example showing multiple services using the modular architecture:

```rust
use anyhow::{Context, Result};
use std::env;
use xai_sdk::Request;
use xai_sdk::api::{
    Content, GetCompletionsRequest, GetModelRequest, Message, MessageRole, SampleTextRequest,
    content,
};
use xai_sdk::{chat, models, sample};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key for authentication
    let api_key = env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated clients for different services
    let mut models_client = models::client::new(api_key.clone()).await?;
    let mut sample_client = sample::client::new(api_key.clone()).await?;
    let mut chat_client = chat::client::new(api_key).await?;

    // List available models
    let models_request = Request::new(());
    let models_response = models_client.list_language_models(models_request).await?;
    println!("Available models: {:?}", models_response.into_inner().models);

    // Get details for a specific model
    let model_request = Request::new(GetModelRequest {
        name: "grok-3-latest".to_string(),
    });
    let model_response = models_client.get_language_model(model_request).await?;
    println!("Model details: {:?}", model_response.into_inner());

    // Generate text
    let sample_request = Request::new(SampleTextRequest {
        prompt: vec!["Hello, world!".to_string()],
        model: "grok-3-latest".to_string(),
        ..Default::default()
    });
    let sample_response = sample_client.sample_text(sample_request).await?;
    println!("Generated: {}", sample_response.into_inner().choices[0].text);

    // Chat completion
    let message = Message {
        role: MessageRole::RoleUser.into(),
        content: vec![Content {
            content: Some(content::Content::Text("Explain Rust ownership".to_string())),
        }],
        ..Default::default()
    };
    let chat_request = Request::new(GetCompletionsRequest {
        model: "grok-3-latest".to_string(),
        messages: vec![message],
        ..Default::default()
    });
    let chat_response = chat_client.get_completion(chat_request).await?;
    println!(
        "Chat response: {}",
        chat_response.into_inner().outputs[0]
            .message
            .as_ref()
            .unwrap()
            .content
    );

    Ok(())
}
```

## Streaming Utilities

The SDK provides powerful utilities for working with streaming responses:

### Stream Consumer
A flexible callback system for processing streaming data:
- **`on_chunk(chunk)`** - Called for each complete chunk received
- **`on_reasoning_token(&OutputContext, token: &str)`** - Called for each piece of reasoning content
- **`on_reasoning_complete(&OutputContext)`** - Called once when the reasoning phase completes for an output
- **`on_content_token(&OutputContext, token: &str)`** - Called for each piece of response content
- **`on_content_complete(&OutputContext)`** - Called once when the content phase completes for an output
- **`on_inline_citations(&OutputContext, &[InlineCitation])`** - Called when inline citations are present in a delta
- **`on_client_tool_calls(&OutputContext, &[ToolCall])`** - Called when client-side tool calls are present
- **`on_server_tool_calls(&OutputContext, &[ToolCall])`** - Called when server-side tool calls are present
- **`on_usage(&SamplingUsage)`** - Called once on the last chunk with usage statistics
- **`on_citations(&[String])`** - Called once on the last chunk with citations

The `OutputContext` provides:
- `total_outputs` - Total number of outputs in the stream
- `output_index` - Index of the output this context belongs to
- `reasoning_status` - Current status of the reasoning phase (`Init`, `Pending`, or `Complete`)
- `content_status` - Current status of the content phase (`Init`, `Pending`, or `Complete`)

### Stream Processing Functions
- **`chat::stream::process`** - Process streaming responses with custom callbacks
- **`chat::stream::assemble`** - Convert collected chunks into complete responses
- **`chat::stream::Consumer::with_stdout()`** - Pre-configured consumer for single-output real-time output
- **`chat::stream::Consumer::with_buffered_stdout()`** - Pre-configured consumer for multi-output buffered output

## Interceptors

The SDK provides a flexible interceptor system for customizing request handling:

### Authentication Interceptor
The `auth()` function creates an interceptor that adds Bearer token authentication:

```rust
use xai_sdk::common::interceptor::auth;

let interceptor = auth("your-api-key");
let client = chat::client::with_interceptor(interceptor).await?;
```

### Composing Interceptors
Combine multiple interceptors using `compose()`:

```rust
use xai_sdk::common::interceptor::{auth, compose};

let interceptors: Vec<Box<dyn xai_sdk::export::service::Interceptor + Send + Sync>> = vec![
    Box::new(auth("your-api-key")),
    Box::new(|mut req| {
        req.metadata_mut().insert("x-custom-header", "value".parse().unwrap());
        Ok(req)
    }),
];
let composed = compose(interceptors);
let client = chat::client::with_interceptor(composed).await?;
```

**Note**: All interceptors must be `Send + Sync` to ensure thread safety when used in async contexts.

### ClientInterceptor Type
All client functions return `ClientInterceptor`, a concrete type that can be:
- Stored in structs
- Used in trait implementations
- Passed between functions without type erasure issues
- Used across thread boundaries (`Send + Sync`) - safe to use in `tokio::spawn` and other async contexts

The `ClientInterceptor` can be created from any `impl Interceptor + Send + Sync + 'static`:

```rust
use xai_sdk::common::interceptor::ClientInterceptor;

let interceptor = ClientInterceptor::new(|mut req| {
    // Custom logic
    Ok(req)
});

// Can be used in spawned tasks
tokio::spawn(async move {
    let client = chat::client::with_interceptor(interceptor).await?;
    // Use client...
});
```

## Configuration

The SDK supports comprehensive configuration options:

- **Temperature**: Controls randomness (0.0 to 2.0)
- **Top-p**: Nucleus sampling parameter (0.0 to 1.0)  
- **Max tokens**: Maximum tokens to generate
- **Log probabilities**: Enable detailed token probability logging
- **Multiple completions**: Generate multiple responses per request
- **Stop sequences**: Custom stop conditions
- **Frequency/Presence penalties**: Control repetition and topic diversity

## Security

- **TLS Encryption**: Automatic HTTPS with certificate validation
- **Authentication**: Bearer token support for API key authentication
- **Secure by Default**: No manual TLS configuration required

## Error Handling

Comprehensive error handling for:
- Connection errors and timeouts
- Authentication failures
- API rate limiting
- Invalid parameters
- Network issues

## Development

This SDK is built using:

- **Protocol Buffers**: Auto-generated Rust types from xAI's `.proto` definitions
- **Tonic**: Modern gRPC framework for Rust with async/await support
- **Prost**: High-performance Protocol Buffer implementation

The code is generated from xAI's official Protocol Buffer definitions, ensuring compatibility and type safety.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes and new features.

## License

This project is licensed under the MIT License.
