# xAI SDK

A comprehensive Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services including Grok language models, embeddings, image generation, and more.

## Features

- **Complete API Coverage**: Full gRPC client implementation for all xAI services
- **Type Safety**: Auto-generated Rust types from Protocol Buffers
- **Async/Await**: Built on Tokio for high-performance async operations
- **Multiple Models**: Support for all xAI language models (Grok-2, Grok-3, etc.)
- **Streaming Support**: Real-time streaming for chat completions and text generation
- **Response Assembly**: Convert streaming chunks into complete responses
- **Secure**: TLS encryption with automatic certificate validation
- **Production Ready**: Comprehensive error handling and connection management

## Quick Start

### Prerequisites

1. Rust 1.70+ installed
2. xAI API key

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xai-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Running the Examples

1. Set your API key as an environment variable:
   ```bash
   export XAI_API_KEY="your-api-key-here"
   ```

2. Run the text generation example:
   ```bash
   cargo run --example raw_text_sample
   ```

3. Run the chat completion example:
   ```bash
   cargo run --example chat_post
   ```

4. Run the streaming chat example:
   ```bash
   cargo run --example chat_stream
   ```

5. Run the response assembly example:
   ```bash
   cargo run --example assemble_response
   ```

6. Test your connection:
   ```bash
   cargo run --bin test_connection
   ```

## Usage Examples

### Text Generation

```rust
use xai_sdk::{sample, SampleTextRequest};
use tonic::Request;

let mut client = sample::create_client().await?;

let request = Request::new(SampleTextRequest {
    prompt: vec!["Write a haiku about Rust programming".to_string()],
    model: "grok-2-latest".to_string(),
    max_tokens: Some(50),
    temperature: Some(0.8),
    ..Default::default()
});

// Add authentication to the request
let authenticated_request = sample::add_auth(request, "your-api-key-here")?;

let response = client.sample_text(authenticated_request).await?;
println!("Generated: {}", response.into_inner().choices[0].text);
```

### Chat Completion

```rust
use xai_sdk::{chat, GetCompletionsRequest, Message, MessageRole, Content};
use tonic::Request;

let mut client = chat::create_client().await?;

let message = Message {
    role: MessageRole::RoleUser.into(),
    content: vec![Content {
        content: Some(content::Content::Text("Explain quantum computing".to_string())),
    }],
    ..Default::default()
};

let request = Request::new(GetCompletionsRequest {
    model: "grok-3-latest".to_string(),
    messages: vec![message],
    ..Default::default()
});

// Add authentication to the request
let authenticated_request = chat::add_auth(request, "your-api-key-here")?;

let response = client.get_completion(authenticated_request).await?;
println!("Response: {}", response.into_inner().choices[0].message.unwrap().content);
```

### Streaming Chat Completion

```rust
use xai_sdk::{chat, GetCompletionsRequest, Message, MessageRole, Content};
use tonic::Request;

let mut client = chat::create_client().await?;

let message = Message {
    role: MessageRole::RoleUser.into(),
    content: vec![Content {
        content: Some(content::Content::Text("Write a poem about Rust".to_string())),
    }],
    ..Default::default()
};

let request = Request::new(GetCompletionsRequest {
    model: "grok-3-latest".to_string(),
    messages: vec![message],
    ..Default::default()
});

// Add authentication to the request
let authenticated_request = chat::add_auth(request, "your-api-key-here")?;

// Process streaming response
let stream = client.get_completion_chunk(authenticated_request).await?.into_inner();
let consumer = chat::StreamConsumer::with_stdout();
let chunks = chat::process_stream(stream, consumer).await?;

// Assemble complete response
if let Some(response) = chat::assemble_response(chunks) {
    println!("Complete response: {}", 
        response.choices[0].message.as_ref().unwrap().content);
}
```

## API Services

The SDK provides clients for all xAI services:

### Chat Service
- **`GetCompletion`** - Blocking chat completion
- **`GetCompletionChunk`** - Streaming chat completion  
- **`StartDeferredCompletion`** - Async completion with polling
- **`GetDeferredCompletion`** - Retrieve async results

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

### Client Creation
Each module provides a simple `create_client()` function and an `add_auth()` helper:

```rust
use xai_sdk::{chat, sample, models, embed};
use tonic::Request;

// Create clients for different services
let chat_client = chat::create_client().await?;
let sample_client = sample::create_client().await?;
let models_client = models::create_client().await?;
let embed_client = embed::create_client().await?;

// Add authentication to requests
let request = Request::new(GetCompletionsRequest::default());
let authenticated_request = chat::add_auth(request, "your-api-key-here")?;
```

### Complete Example
Here's a complete example showing multiple services:

```rust
use xai_sdk::{chat, models, sample};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "your-api-key-here";
    
    // List available models
    let models_client = models::create_client().await?;
    let models_request = Request::new(());
    let authenticated_models_request = models::add_auth(models_request, api_key)?;
    let models_response = models_client.list_language_models(authenticated_models_request).await?;
    println!("Available models: {:?}", models_response.into_inner().models);

    // Generate text
    let sample_client = sample::create_client().await?;
    let sample_request = Request::new(sample::SampleTextRequest {
        prompt: vec!["Hello, world!".to_string()],
        model: "grok-2-latest".to_string(),
        ..Default::default()
    });
    let authenticated_sample_request = sample::add_auth(sample_request, api_key)?;
    let sample_response = sample_client.sample_text(authenticated_sample_request).await?;
    println!("Generated: {}", sample_response.into_inner().choices[0].text);

    // Chat completion
    let chat_client = chat::create_client().await?;
    // ... use chat_client for chat completions with authentication

    Ok(())
}
```

## Streaming Utilities

The SDK provides powerful utilities for working with streaming responses:

### StreamConsumer
A flexible callback system for processing streaming data:
- **`on_content_token`** - Called for each piece of response content
- **`on_reason_token`** - Called for each piece of reasoning content  
- **`on_chunk`** - Called for each complete chunk received

### Stream Processing Functions
- **`process_stream`** - Process streaming responses with custom callbacks
- **`assemble_response`** - Convert collected chunks into complete responses
- **`StreamConsumer::with_stdout`** - Pre-configured consumer for real-time output

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
