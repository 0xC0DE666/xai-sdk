# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `assemble_response` function to convert streaming chunks into complete `GetChatCompletionResponse`
- `process_stream` function with comprehensive documentation
- Example demonstrating stream processing and response assembly (`examples/assemble_response.rs`)
- Support for accumulating encrypted content from streaming deltas
- Multiple choice handling in stream assembly

### Changed
- Improved documentation for streaming functions
- Enhanced error handling in stream processing

## [0.1.0] - 2025-10-28

### Added
- Initial release of xAI SDK
- Complete gRPC client implementation for all xAI services
- Support for Grok language models (Grok-2, Grok-3)
- Streaming support for chat completions and text generation
- Type-safe Rust bindings generated from Protocol Buffers
- Comprehensive examples for all major features
- TLS encryption with automatic certificate validation
- Authentication support with API key management
- Support for all xAI services:
  - Chat completions (blocking and streaming)
  - Text generation and sampling
  - Embeddings generation
  - Image generation
  - Model listing and information
  - Deferred completions
  - Authentication services
- Configuration options for temperature, top-p, max tokens, and more
- Comprehensive error handling and connection management
