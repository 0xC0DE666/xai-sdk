# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2025-10-31
Cleaned up readme.

## [0.3.1] - 2025-10-31
Added repo url to Cargo.toml.

## [0.3.0] - 2025-10-31

### Added
- **Enhanced Stream Consumer**: `Consumer` callbacks now receive `(total_choices, choice_idx, token)` parameters for better multi-choice handling
- **Buffered Output**: `Consumer::with_buffered_stdout()` for clean, non-interleaved output when streaming multiple choices
- **Interceptor Composition**: `common::interceptor::compose()` for combining multiple interceptors
- **Custom Interceptor Support**: `with_interceptor()` and `with_channel_and_interceptor()` methods on all client modules
- **Comprehensive Documentation**: Complete rustdoc documentation for all modules, functions, and types
- **Module-Level Documentation**: Added module descriptions explaining the purpose of each service client

### Changed
- **BREAKING**: `Consumer::on_content_token` and `Consumer::on_reason_token` callbacks now receive `(total_choices: usize, choice_idx: usize, token: &str)` instead of `(choice_idx: i32, token: &str)`
- **BREAKING**: Renamed `chat::stream::assemble_response()` to `chat::stream::assemble()`
- **Stream Consumer**: `Consumer::with_stdout()` now only prints tokens from the first choice to avoid output mangling
- **Documentation**: Fixed incorrect return types in `with_channel()` documentation across all modules
- **Type Consistency**: Stream consumer indices changed from `i32` to `usize` for better type safety

### Fixed
- Corrected return type documentation for `with_channel()` methods (they return directly, not wrapped in Result)
- Fixed module documentation formatting issues
- Improved accuracy of all rustdoc comments

## [0.2.0] - 2025-10-30

### Added
- **Modular Client Architecture**: Each service now has a dedicated `client` submodule with automatic authentication
- **Automatic Authentication**: Clients created with `module::client::new(api_key)` handle authentication automatically via interceptors
- **Common Utilities**: `common::channel::new()` for centralized channel creation and `common::interceptor::auth()` for authentication
- **Stream Processing**: `chat::stream::process()` and `chat::stream::assemble()` for handling streaming responses
- **StreamConsumer**: Flexible callback system for processing streaming data with `chat::stream::Consumer`
- **Channel Reuse**: `with_channel()` methods for creating clients with existing channels
- **Comprehensive Examples**: Updated all examples to use the new modular architecture
- Support for accumulating encrypted content from streaming deltas
- Multiple choice handling in stream assembly

### Changed
- **BREAKING**: Removed `XaiClient` unified client in favor of modular architecture
- **BREAKING**: All modules now use `module::client::new(api_key)` instead of `module::create_client()`
- **BREAKING**: Removed manual `add_auth()` functions - authentication is now automatic
- **BREAKING**: Streaming functions moved to `chat::stream::` namespace
- **Module Structure**: Each module now has a `client` submodule with `new()` and `with_channel()` methods
- **Documentation**: Completely updated README and examples to reflect new architecture
- **Import Organization**: Consistent import ordering across all modules
- **Error Handling**: Enhanced error handling in stream processing and client creation

### Removed
- `XaiClient` unified client struct
- Manual authentication helper functions (`add_auth()`)
- Old `create_client()` functions
- `client.rs` module (functionality moved to individual service modules)

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
