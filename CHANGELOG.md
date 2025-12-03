# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2025-12-03

### Changed
- **BREAKING**: All client creation functions now return `ClientInterceptor` instead of `impl Interceptor`
  - This provides a concrete, nameable type that can be stored in structs and used in trait implementations
  - All client modules (`auth`, `chat`, `sample`, `tokenize`, `models`, `image`, `embed`, `documents`) updated
  - Functions affected: `new()`, `with_channel()`, `with_interceptor()`, `with_channel_and_interceptor()`
- **BREAKING**: `common::interceptor::auth()` now returns `ClientInterceptor` instead of `impl Interceptor`
- **BREAKING**: `common::interceptor::compose()` now returns `ClientInterceptor` instead of `impl Interceptor`
- **BREAKING**: Renamed `BoxInterceptor` to `ClientInterceptor` for better semantic clarity
  - The type is specifically designed for use in client contexts where concrete types are needed
  - Updated all documentation and examples

### Added
- **`ClientInterceptor`**: New concrete interceptor type for use in client contexts
  - Implements `Interceptor` trait
  - Can be created from any `impl Interceptor + 'static` via `ClientInterceptor::new()`
  - Supports `From<Box<dyn Interceptor>>` for conversion from boxed interceptors
  - Provides a concrete type that can be stored in structs and used in return positions
- **Enhanced `ClientInterceptor::new()`**: Now accepts `impl Interceptor + 'static` and boxes internally
  - Eliminates the need for double-boxing (`BoxInterceptor::new(Box::new(...))`)
  - More ergonomic API: `ClientInterceptor::new(interceptor)` instead of `ClientInterceptor::new(Box::new(interceptor))`
- **Comprehensive test suite**: Added extensive tests organized by source file
  - `sdk/tests/common.rs`: Tests for interceptor functionality and channel creation
  - `sdk/tests/chat.rs`: Tests for streaming utilities, Consumer, and assembly functions
  - All tests follow the same file structure as the source code

### Fixed
- Fixed lifetime issues in `auth()` function by cloning the API key string
- Improved type safety by providing concrete return types instead of opaque `impl Trait` types

## [0.6.1] - 2025-11-23

### Changed
- **Internal refactoring**: All user-facing modules now use types from the `export` module instead of direct `tonic` imports
  - Ensures that only properly exported types are used throughout the codebase
  - Validates that all exported types work correctly
  - Improved code consistency and maintainability
- **Enhanced export module**: Added mirroring module structure (`transport`, `service`, `metadata`) to match `tonic`'s organization
  - `xai_sdk::transport::Channel`, `xai_sdk::transport::Error`, etc.
  - `xai_sdk::service::Interceptor`, `xai_sdk::service::interceptor::InterceptedService`
  - `xai_sdk::metadata::MetadataValue`

## [0.6.0] - 2025-11-23

### Added
- **Re-exported tonic types**: Added `export` module re-exporting commonly used `tonic` types
  - Users can now use `xai_sdk::Request`, `xai_sdk::Response`, `xai_sdk::Status`, and `xai_sdk::Streaming`
  - No longer need to add `tonic` as a dependency for basic usage
  - All examples updated to use re-exported types

## [0.5.0] - 2025-11-23

### Changed
- **Project Structure**: Converted from single-package to Cargo workspace structure
  - Main SDK library moved to `sdk/` crate
  - Examples moved to separate `examples/` crate
  - Improved separation of concerns and dependency management
- **Build System**: Updated build script to generate code in the new workspace structure
- **Workspace Configuration**: Centralized workspace metadata (edition, license, repository, rust-version)

### Internal
- Reorganized source files from `src/` to `sdk/src/`
- Examples now have their own `Cargo.toml` with proper dependency on the SDK
- Regenerated Protocol Buffer code in new location

## [0.4.0] - 2025-11-09

### Added
- **TokenContext**: New `TokenContext` struct providing contextual information about tokens in streaming responses
  - Includes `total_choices`, `choice_index`, `reasoning_status`, and `content_status` fields
  - Comprehensive rustdoc documentation
- **PhaseStatus Enum**: New `PhaseStatus` enum (`Init`, `Pending`, `Complete`) for tracking reasoning and content phase states
- **CompletionContext**: New `CompletionContext` struct providing contextual information to completion callbacks
  - Includes `choice_index` and `total_choices` fields to identify which choice completed
  - Passed to `on_reasoning_complete` and `on_content_complete` callbacks
- **Completion Callbacks**: New callbacks for detecting phase completion transitions
  - `Consumer::on_reasoning_complete(CompletionContext)` - Called once when reasoning phase completes for a choice
  - `Consumer::on_content_complete(CompletionContext)` - Called once when content phase completes for a choice
  - Builder methods `on_reasoning_complete()` and `on_content_complete()` for fluent API
- **Enhanced Completion Detection**: Improved logic for detecting when reasoning and content phases are complete using `finish_reason`
- **Per-Choice Completion Tracking**: Completion callbacks now correctly track state per choice, enabling proper handling of multiple concurrent choices

### Changed
- **BREAKING**: `Consumer::on_content_token` and `Consumer::on_reason_token` callbacks now receive `(TokenContext, token: &str)` instead of `(total_choices: usize, choice_idx: usize, token: &str)`
  - Context-first parameter order for better ergonomics
  - All related data grouped into a single struct for easier extension
- **BREAKING**: `TokenContext` fields changed from boolean flags to `PhaseStatus` enum:
  - `reasoning_complete: bool` → `reasoning_status: PhaseStatus`
  - `content_complete: bool` → `content_status: PhaseStatus`
- **BREAKING**: `Consumer::with_stdout()` now uses completion callbacks to print newlines when reasoning or content phases complete

### Fixed
- Fixed completion tracking to work correctly with multiple concurrent choices (per-choice state tracking)
- Fixed completion flag logic to accurately detect reasoning and content completion using `finish_reason`
- Fixed documentation typos and improved consistency

## [0.3.2] - 2025-10-31

### Changed
- **BREAKING**: Removed wildcard re-export of `xai_api` types. Types from the `xai_api` module (e.g., `Message`, `Content`, `GetCompletionsRequest`, etc.) must now be imported explicitly via `xai_sdk::xai_api::...` instead of being available directly from `xai_sdk`
- Updated all examples to use explicit `xai_api` imports
- Updated README documentation with correct import examples

### Fixed
- Improved API clarity by making generated types explicitly scoped to `xai_api` module

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
