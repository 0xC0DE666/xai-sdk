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
use xai_sdk::{SampleClient, SampleTextRequest};
use tonic::Request;

let mut client = SampleClient::connect("https://api.x.ai:443").await?;

let request = Request::new(SampleTextRequest {
    prompt: vec!["Write a haiku about Rust programming".to_string()],
    model: "grok-2-latest".to_string(),
    max_tokens: Some(50),
    temperature: Some(0.8),
    ..Default::default()
});

let response = client.sample_text(request).await?;
println!("Generated: {}", response.into_inner().choices[0].text);
```

### Chat Completion

```rust
use xai_sdk::{ChatClient, GetCompletionsRequest, Message, MessageRole, Content};
use tonic::Request;

let mut client = ChatClient::connect("https://api.x.ai:443").await?;

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

let response = client.get_completion(request).await?;
println!("Response: {}", response.into_inner().choices[0].message.unwrap().content);
```

### Streaming Chat Completion

```rust
use xai_sdk::{ChatClient, GetCompletionsRequest, Message, MessageRole, Content, chat};
use tonic::Request;

let mut client = ChatClient::connect("https://api.x.ai:443").await?;

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

// Process streaming response
let stream = client.get_completion_chunk(request).await?.into_inner();
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
