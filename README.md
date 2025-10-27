# xAI gRPC Rust Client

A Rust client library for the xAI (Grok) API using gRPC protocol.

## Features

- Complete gRPC client implementation for all xAI services
- Raw text sampling with configurable parameters
- Chat completions with streaming support
- Image generation capabilities
- Embedding generation
- Model management and information
- Authentication and error handling

## Quick Start

### Prerequisites

1. Rust 1.70+ installed
2. xAI API key

### Running the Examples

1. Set your API key as an environment variable:
   ```bash
   export XAI_API_KEY="your-api-key-here"
   ```

2. Run the comprehensive example:
   ```bash
   cargo run --example raw_text_sample
   ```

3. Or run the simple example:
   ```bash
   cargo run --example simple_sample
   ```

4. Or use the convenience script:
   ```bash
   ./run_example.sh
   ```

#### Example Features

**Comprehensive Example (`raw_text_sample`):**
- Creative writing with high temperature
- Technical explanations with low temperature
- Multiple completions generation
- Proper error handling and logging
- Detailed output formatting

**Simple Example (`simple_sample`):**
- Basic text generation
- Minimal configuration
- Easy to understand code structure
- Perfect for getting started

### Example Output

```
🚀 xAI Raw Text Sampling Demo
=============================

📝 Example 1: Creative Writing
-------------------------------
📤 Sending request: Creative poem about Rust
🤖 Model: grok-2-latest
⚙️  Max tokens: Some(200), Temperature: Some(0.8), Top-p: Some(0.9)

✅ Response received!
🆔 Request ID: req_1234567890
🤖 Model used: grok-2-1212
📊 Usage: SamplingUsage { ... }

📄 Choice 1:
   Text: In the realm of systems, where memory's tight,
   Rust stands tall with its ownership might.
   No garbage collector, no runtime cost,
   Just fearless concurrency, nothing lost.
   
   Finish reason: REASON_STOP
```

## API Services

### Chat Service
- `GetCompletion` - Blocking text completion
- `GetCompletionChunk` - Streaming text completion
- `StartDeferredCompletion` - Async completion with polling
- `GetDeferredCompletion` - Retrieve async results

### Sample Service
- `SampleText` - Raw text sampling
- `SampleTextStreaming` - Streaming text sampling

### Models Service
- `ListLanguageModels` - List available language models
- `ListEmbeddingModels` - List embedding models
- `ListImageGenerationModels` - List image generation models

### Embed Service
- `Embed` - Generate embeddings from text or images

### Image Service
- `GenerateImage` - Create images from text prompts

### Auth Service
- `get_api_key_info` - Get API key information

## Configuration

The client supports various configuration options:

- **Temperature**: Controls randomness (0.0 to 2.0)
- **Top-p**: Nucleus sampling parameter (0.0 to 1.0)
- **Max tokens**: Maximum tokens to generate
- **Log probabilities**: Enable detailed token probability logging
- **Multiple completions**: Generate multiple responses per request

## Error Handling

The client includes comprehensive error handling:
- Connection errors
- Authentication failures
- API rate limiting
- Invalid parameters
- Network timeouts

## License

This project is licensed under the MIT License.
