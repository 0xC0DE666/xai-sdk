# xAI SDK

A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services including Grok language models, embeddings, image generation, and more.

## Features

- **Complete API Coverage**: Full gRPC client implementation for all xAI services
- **Type Safety**: Auto-generated Rust types from Protocol Buffers
- **Async/Await**: Built on Tokio for high-performance async operations
- **Multiple Models**: Support for all xAI language models (Grok-3, Grok-4, etc.)
- **Streaming Support**: Real-time streaming for chat completions and text generation
- **Response Assembly**: Convert streaming chunks into complete responses
- **Secure**: TLS encryption with automatic certificate validation

## Quick Start

### Prerequisites

1. Rust 1.70+ installed
2. xAI API key

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xai-sdk = "0.4"
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

## API Services

The SDK provides clients for all xAI services:

### Chat Service
- **`GetCompletion`** - Get chat completion
- **`GetCompletionChunk`** - Stream chat completion in chunks
- **`StartDeferredCompletion`** - Start defered chat completion
- **`GetDeferredCompletion`** - Retrieve defered completion
- **`GetStoredCompletion`** - Get stored chat completion
- **`DeleteStoredCompletion`** - Delete stored chat completion

### Sample Service  
- **`SampleText`** - Raw text generation
- **`SampleTextStreaming`** - Streaming text generation

### Models Service
- **`ListLanguageModels`** - List available language models
- **`ListEmbeddingModels`** - List embedding models
- **`ListImageGenerationModels`** - List image generation models

### Embed Service
- **`Embed`** - Generate embeddings from text or images

### Image Service
- **`GenerateImage`** - Create images from text prompts

### Auth Service
- **`get_api_key_info`** - Get API key information

## Client Modules

The SDK is organized into focused modules, each providing easy client creation:

### Available Modules
- **`auth`** - Authentication services
- **`chat`** - Chat completions and streaming
- **`documents`** - Document processing
- **`embed`** - Text and image embeddings
- **`image`** - Image generation
- **`models`** - Model listing and information
- **`sample`** - Text sampling and generation
- **`tokenize`** - Text tokenization

### Complete Example
Here's a complete example showing multiple services using the modular architecture:

```rust
use anyhow::Context;
use std::env;
use tonic::Request;
use xai_sdk::xai_api::{
    Content, GetCompletionsRequest, Message, MessageRole, SampleTextRequest, content,
};
use xai_sdk::{chat, models, sample};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key for authentication
    let api_key = env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;
    
    // Create authenticated clients for different services
    let mut models_client = models::client::new(api_key).await?;
    let mut sample_client = sample::client::new(api_key).await?;
    let mut chat_client = chat::client::new(api_key).await?;
    
    // List available models
    let models_request = Request::new(());
    let models_response = models_client.list_language_models(models_request).await?;
    println!("Available models: {:?}", models_response.into_inner().models);

    // Generate text
    let sample_request = Request::new(SampleTextRequest {
        prompt: vec!["Hello, world!".to_string()],
        model: "grok-2-latest".to_string(),
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
    println!("Chat response: {}", chat_response.into_inner().choices[0].message.unwrap().content);

    Ok(())
}
```

## Streaming Utilities

The SDK provides powerful utilities for working with streaming responses:

### Stream Consumer
A flexible callback system for processing streaming data:
- **`on_content_token(TokenContext, token: &str)`** - Called for each piece of response content
- **`on_reason_token(TokenContext, token: &str)`** - Called for each piece of reasoning content  
- **`on_chunk(chunk)`** - Called for each complete chunk received

The `TokenContext` provides:
- `total_choices` - Total number of choices in the stream
- `choice_index` - Index of the choice this token belongs to
- `reasoning_complete` - Whether the reasoning phase is complete
- `content_complete` - Whether the content phase is complete

### Stream Processing Functions
- **`chat::stream::process`** - Process streaming responses with custom callbacks
- **`chat::stream::assemble`** - Convert collected chunks into complete responses
- **`chat::stream::Consumer::with_stdout()`** - Pre-configured consumer for single-choice real-time output
- **`chat::stream::Consumer::with_buffered_stdout()`** - Pre-configured consumer for multi-choice buffered output

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
- **Tokio**: Async runtime for Rust

The code is generated from xAI's official Protocol Buffer definitions, ensuring compatibility and type safety.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes and new features.

## License

This project is licensed under the MIT License.
