//! Utilities module.
//!
//! Provides extended functionality for xAI API enums and utility functions
//! to enhance the developer experience.
//!
//! ## Extended Enums
//!
//! This module re-exports all enums from the generated Protocol Buffer code
//! and adds convenience methods for common operations:
//!
//! ### Display and Parsing
//!
//! Many enums implement `Display` and `FromStr` for easy conversion:
//!
//! ```rust
//! use xai_sdk::utils::enums::*;
//!
//! // Display enum values as strings
//! let role = MessageRole::RoleUser;
//! println!("Role: {}", role); // "user"
//!
//! // Parse from strings
//! let parsed: MessageRole = "assistant".parse().unwrap();
//! assert_eq!(parsed, MessageRole::RoleAssistant);
//! ```
//!
//! ### Available Enums
//!
//! - [`MessageRole`] - User, assistant, system, function roles
//! - [`FinishReason`] - Why generation stopped (stop, length, etc.)
//! - [`Modality`] - Input/output modalities (text, image, etc.)
//! - [`ImageDetail`] - Image detail levels
//! - [`ImageFormat`] - Image file formats
//! - [`ImageQuality`] - Image quality settings (low, medium, high)
//! - [`ImageAspectRatio`] - Image aspect ratios (1:1, 16:9, etc.)
//! - [`ImageResolution`] - Image resolution settings
//! - [`RankingMetric`] - Similarity ranking methods
//! - [`ReasoningEffort`] - Reasoning effort levels
//! - [`SearchMode`] - Search operation modes
//! - [`ToolMode`] - Tool calling modes
//! - [`ToolCallType`] - Types of tool calls (client-side, server-side)
//! - [`ToolCallStatus`] - Tool call execution status
//! - [`ServerSideTool`] - Available server-side tools
//! - [`IncludeOption`] - Content inclusion options
//! - [`VideoAspectRatio`] - Video aspect ratios (1:1, 16:9, etc.)
//! - [`VideoResolution`] - Video resolution settings (480p, 720p)
//!
//! ## Implementation Details
//!
//! The extended functionality is implemented through:
//!
//! - `Display` trait implementations for human-readable output
//! - `FromStr` trait implementations for parsing from strings
//! - Additional convenience methods where appropriate
//!
//! All implementations handle deprecated enum variants gracefully
//! and provide clear error messages for invalid inputs.
pub mod enums {
    use std::fmt;
    use std::str::FromStr;

    // Re-export all enums
    pub use crate::xai_api::{
        DeferredStatus, EmbedEncodingFormat, FinishReason, FormatType, ImageAspectRatio,
        ImageDetail, ImageFormat, ImageQuality, ImageResolution, IncludeOption, MessageRole,
        Modality, RankingMetric, ReasoningEffort, SearchMode, ServerSideTool, ToolCallStatus,
        ToolCallType, ToolMode, VideoAspectRatio, VideoResolution,
    };

    impl fmt::Display for DeferredStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                DeferredStatus::InvalidDeferredStatus => "invalid_deferred_status",
                DeferredStatus::Done => "done",
                DeferredStatus::Expired => "expired",
                DeferredStatus::Pending => "pending",
            };
            f.write_str(s)
        }
    }

    impl FromStr for DeferredStatus {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_uppercase().as_str() {
                "INVALID_DEFERRED_STATUS" | "INVALID" => Ok(DeferredStatus::InvalidDeferredStatus),
                "DONE" => Ok(DeferredStatus::Done),
                "EXPIRED" => Ok(DeferredStatus::Expired),
                "PENDING" => Ok(DeferredStatus::Pending),
                _ => DeferredStatus::from_str_name(s)
                    .ok_or_else(|| format!("Invalid deferred status: '{s}'")),
            }
        }
    }

    impl fmt::Display for RankingMetric {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                RankingMetric::Unknown => "unknown",
                RankingMetric::L2Distance => "l2_distance",
                RankingMetric::CosineSimilarity => "cosine_similarity",
            };
            f.write_str(s)
        }
    }

    impl FromStr for RankingMetric {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_uppercase().as_str() {
                "UNKNOWN" => Ok(RankingMetric::Unknown),
                "L2_DISTANCE" | "L2" => Ok(RankingMetric::L2Distance),
                "COSINE_SIMILARITY" | "COSINE" => Ok(RankingMetric::CosineSimilarity),
                _ => RankingMetric::from_str_name(s)
                    .ok_or_else(|| format!("Invalid ranking metric: '{s}'")),
            }
        }
    }

    impl fmt::Display for ImageDetail {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ImageDetail::DetailInvalid => "invalid",
                ImageDetail::DetailAuto => "auto",
                ImageDetail::DetailLow => "low",
                ImageDetail::DetailHigh => "high",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ImageDetail {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ImageDetail::DetailInvalid),
                "auto" => Ok(ImageDetail::DetailAuto),
                "low" => Ok(ImageDetail::DetailLow),
                "high" => Ok(ImageDetail::DetailHigh),
                _ => ImageDetail::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid image detail: '{s}'")),
            }
        }
    }

    impl fmt::Display for ImageFormat {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ImageFormat::ImgFormatInvalid => "invalid",
                ImageFormat::ImgFormatBase64 => "base64",
                ImageFormat::ImgFormatUrl => "url",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ImageFormat {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ImageFormat::ImgFormatInvalid),
                "base64" => Ok(ImageFormat::ImgFormatBase64),
                "url" => Ok(ImageFormat::ImgFormatUrl),
                _ => ImageFormat::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid image format: '{s}'")),
            }
        }
    }

    impl fmt::Display for FinishReason {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                FinishReason::ReasonInvalid => "invalid",
                FinishReason::ReasonMaxLen => "max_len",
                FinishReason::ReasonMaxContext => "max_context",
                FinishReason::ReasonStop => "stop",
                FinishReason::ReasonToolCalls => "tool_calls",
                FinishReason::ReasonTimeLimit => "time_limit",
            };
            f.write_str(s)
        }
    }

    impl FromStr for FinishReason {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(FinishReason::ReasonInvalid),
                "max_len" => Ok(FinishReason::ReasonMaxLen),
                "max_context" => Ok(FinishReason::ReasonMaxContext),
                "stop" => Ok(FinishReason::ReasonStop),
                "tool_calls" => Ok(FinishReason::ReasonToolCalls),
                "time_limit" => Ok(FinishReason::ReasonTimeLimit),
                _ => FinishReason::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid finish reason: '{s}'")),
            }
        }
    }

    impl fmt::Display for MessageRole {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                MessageRole::InvalidRole => "invalid",
                MessageRole::RoleUser => "user",
                MessageRole::RoleAssistant => "assistant",
                MessageRole::RoleSystem => "system",
                MessageRole::RoleFunction => "function",
                MessageRole::RoleTool => "tool",
                MessageRole::RoleDeveloper => "developer",
            };
            f.write_str(s)
        }
    }

    impl FromStr for MessageRole {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(MessageRole::InvalidRole),
                "user" => Ok(MessageRole::RoleUser),
                "assistant" => Ok(MessageRole::RoleAssistant),
                "system" => Ok(MessageRole::RoleSystem),
                "function" => Ok(MessageRole::RoleFunction),
                "tool" => Ok(MessageRole::RoleTool),
                "developer" => Ok(MessageRole::RoleDeveloper),
                _ => MessageRole::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid message role: '{s}'")),
            }
        }
    }

    impl fmt::Display for ReasoningEffort {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ReasoningEffort::InvalidEffort => "invalid",
                ReasoningEffort::EffortLow => "low",
                ReasoningEffort::EffortMedium => "medium",
                ReasoningEffort::EffortHigh => "high",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ReasoningEffort {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ReasoningEffort::InvalidEffort),
                "low" => Ok(ReasoningEffort::EffortLow),
                "medium" => Ok(ReasoningEffort::EffortMedium),
                "high" => Ok(ReasoningEffort::EffortHigh),
                _ => ReasoningEffort::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid reasoning effort: '{s}'")),
            }
        }
    }

    impl fmt::Display for ToolMode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ToolMode::Invalid => "invalid",
                ToolMode::Auto => "auto",
                ToolMode::None => "none",
                ToolMode::Required => "required",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ToolMode {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ToolMode::Invalid),
                "auto" => Ok(ToolMode::Auto),
                "none" => Ok(ToolMode::None),
                "required" => Ok(ToolMode::Required),
                _ => ToolMode::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid tool mode: '{s}'")),
            }
        }
    }

    impl fmt::Display for FormatType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                FormatType::Invalid => "invalid",
                FormatType::Text => "text",
                FormatType::JsonObject => "json_object",
                FormatType::JsonSchema => "json_schema",
            };
            f.write_str(s)
        }
    }

    impl FromStr for FormatType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(FormatType::Invalid),
                "text" => Ok(FormatType::Text),
                "json_object" | "json" => Ok(FormatType::JsonObject),
                "json_schema" | "schema" => Ok(FormatType::JsonSchema),
                _ => FormatType::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid format type: '{s}'")),
            }
        }
    }

    impl fmt::Display for SearchMode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                SearchMode::InvalidSearchMode => "invalid",
                SearchMode::OffSearchMode => "off",
                SearchMode::OnSearchMode => "on",
                SearchMode::AutoSearchMode => "auto",
            };
            f.write_str(s)
        }
    }

    impl FromStr for SearchMode {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(SearchMode::InvalidSearchMode),
                "off" => Ok(SearchMode::OffSearchMode),
                "on" => Ok(SearchMode::OnSearchMode),
                "auto" => Ok(SearchMode::AutoSearchMode),
                _ => SearchMode::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid search mode: '{s}'")),
            }
        }
    }

    impl fmt::Display for EmbedEncodingFormat {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                EmbedEncodingFormat::FormatInvalid => "invalid",
                EmbedEncodingFormat::FormatFloat => "float",
                EmbedEncodingFormat::FormatBase64 => "base64",
            };
            f.write_str(s)
        }
    }

    impl FromStr for EmbedEncodingFormat {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(EmbedEncodingFormat::FormatInvalid),
                "float" => Ok(EmbedEncodingFormat::FormatFloat),
                "base64" => Ok(EmbedEncodingFormat::FormatBase64),
                _ => EmbedEncodingFormat::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid embed encoding format: '{s}'")),
            }
        }
    }

    impl fmt::Display for Modality {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                Modality::InvalidModality => "invalid",
                Modality::Text => "text",
                Modality::Image => "image",
                Modality::Embedding => "embedding",
            };
            f.write_str(s)
        }
    }

    impl FromStr for Modality {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(Modality::InvalidModality),
                "text" => Ok(Modality::Text),
                "image" => Ok(Modality::Image),
                "embedding" => Ok(Modality::Embedding),
                _ => Modality::from_str_name(s.to_ascii_uppercase().as_str())
                    .ok_or_else(|| format!("Invalid modality: '{s}'")),
            }
        }
    }

    impl fmt::Display for ServerSideTool {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ServerSideTool::Invalid => "invalid",
                ServerSideTool::WebSearch => "web_search",
                ServerSideTool::XSearch => "x_search",
                ServerSideTool::CodeExecution => "code_execution",
                ServerSideTool::ViewImage => "view_image",
                ServerSideTool::ViewXVideo => "view_x_video",
                ServerSideTool::CollectionsSearch => "collections_search",
                ServerSideTool::Mcp => "mcp",
                ServerSideTool::AttachmentSearch => "attachment_search",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ServerSideTool {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ServerSideTool::Invalid),
                "web_search" | "websearch" => Ok(ServerSideTool::WebSearch),
                "x_search" | "xsearch" => Ok(ServerSideTool::XSearch),
                "code_execution" | "codeexecution" => Ok(ServerSideTool::CodeExecution),
                "view_image" | "viewimage" => Ok(ServerSideTool::ViewImage),
                "view_x_video" | "viewxvideo" => Ok(ServerSideTool::ViewXVideo),
                "collections_search" | "collectionssearch" => Ok(ServerSideTool::CollectionsSearch),
                "mcp" => Ok(ServerSideTool::Mcp),
                "attachment_search" | "attachmentsearch" => Ok(ServerSideTool::AttachmentSearch),
                _ => ServerSideTool::from_str_name(s)
                    .ok_or_else(|| format!("Invalid server side tool: '{s}'")),
            }
        }
    }

    impl fmt::Display for ImageQuality {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ImageQuality::ImgQualityInvalid => "invalid",
                ImageQuality::ImgQualityLow => "low",
                ImageQuality::ImgQualityMedium => "medium",
                ImageQuality::ImgQualityHigh => "high",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ImageQuality {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ImageQuality::ImgQualityInvalid),
                "low" => Ok(ImageQuality::ImgQualityLow),
                "medium" => Ok(ImageQuality::ImgQualityMedium),
                "high" => Ok(ImageQuality::ImgQualityHigh),
                _ => ImageQuality::from_str_name(s)
                    .ok_or_else(|| format!("Invalid image quality: '{s}'")),
            }
        }
    }

    impl fmt::Display for ImageAspectRatio {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ImageAspectRatio::ImgAspectRatioInvalid => "invalid",
                ImageAspectRatio::ImgAspectRatio11 => "1:1",
                ImageAspectRatio::ImgAspectRatio34 => "3:4",
                ImageAspectRatio::ImgAspectRatio43 => "4:3",
                ImageAspectRatio::ImgAspectRatio916 => "9:16",
                ImageAspectRatio::ImgAspectRatio169 => "16:9",
                ImageAspectRatio::ImgAspectRatio23 => "2:3",
                ImageAspectRatio::ImgAspectRatio32 => "3:2",
                ImageAspectRatio::ImgAspectRatioAuto => "auto",
                ImageAspectRatio::ImgAspectRatio9195 => "9:19.5",
                ImageAspectRatio::ImgAspectRatio1959 => "19.5:9",
                ImageAspectRatio::ImgAspectRatio920 => "9:20",
                ImageAspectRatio::ImgAspectRatio209 => "20:9",
                ImageAspectRatio::ImgAspectRatio12 => "1:2",
                ImageAspectRatio::ImgAspectRatio21 => "2:1",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ImageAspectRatio {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "1:1" | "1x1" => Ok(ImageAspectRatio::ImgAspectRatio11),
                "3:4" | "3x4" => Ok(ImageAspectRatio::ImgAspectRatio34),
                "4:3" | "4x3" => Ok(ImageAspectRatio::ImgAspectRatio43),
                "9:16" | "9x16" => Ok(ImageAspectRatio::ImgAspectRatio916),
                "16:9" | "16x9" => Ok(ImageAspectRatio::ImgAspectRatio169),
                "2:3" | "2x3" => Ok(ImageAspectRatio::ImgAspectRatio23),
                "3:2" | "3x2" => Ok(ImageAspectRatio::ImgAspectRatio32),
                "9:19.5" | "9x19.5" => Ok(ImageAspectRatio::ImgAspectRatio9195),
                "19.5:9" | "19.5x9" => Ok(ImageAspectRatio::ImgAspectRatio1959),
                "9:20" | "9x20" => Ok(ImageAspectRatio::ImgAspectRatio920),
                "20:9" | "20x9" => Ok(ImageAspectRatio::ImgAspectRatio209),
                "1:2" | "1x2" => Ok(ImageAspectRatio::ImgAspectRatio12),
                "2:1" | "2x1" => Ok(ImageAspectRatio::ImgAspectRatio21),
                "auto" => Ok(ImageAspectRatio::ImgAspectRatioAuto),
                "invalid" => Ok(ImageAspectRatio::ImgAspectRatioInvalid),
                _ => ImageAspectRatio::from_str_name(s)
                    .ok_or_else(|| format!("Invalid image aspect ratio: '{s}'")),
            }
        }
    }

    impl fmt::Display for ImageResolution {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ImageResolution::ImgResolutionInvalid => "invalid",
                ImageResolution::ImgResolution1k => "1k",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ImageResolution {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ImageResolution::ImgResolutionInvalid),
                "1k" => Ok(ImageResolution::ImgResolution1k),
                _ => ImageResolution::from_str_name(s)
                    .ok_or_else(|| format!("Invalid image resolution: '{s}'")),
            }
        }
    }

    impl fmt::Display for IncludeOption {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                IncludeOption::Invalid => "invalid",
                IncludeOption::WebSearchCallOutput => "web_search_call_output",
                IncludeOption::XSearchCallOutput => "x_search_call_output",
                IncludeOption::CodeExecutionCallOutput => "code_execution_call_output",
                IncludeOption::CollectionsSearchCallOutput => "collections_search_call_output",
                IncludeOption::AttachmentSearchCallOutput => "attachment_search_call_output",
                IncludeOption::McpCallOutput => "mcp_call_output",
                IncludeOption::InlineCitations => "inline_citations",
                IncludeOption::VerboseStreaming => "verbose_streaming",
            };
            f.write_str(s)
        }
    }

    impl FromStr for IncludeOption {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(IncludeOption::Invalid),
                "web_search_call_output" | "websearch" => Ok(IncludeOption::WebSearchCallOutput),
                "x_search_call_output" | "xsearch" => Ok(IncludeOption::XSearchCallOutput),
                "code_execution_call_output" | "codeexecution" => {
                    Ok(IncludeOption::CodeExecutionCallOutput)
                }
                "collections_search_call_output" | "collectionssearch" => {
                    Ok(IncludeOption::CollectionsSearchCallOutput)
                }
                "attachment_search_call_output" | "attachmentsearch" => {
                    Ok(IncludeOption::AttachmentSearchCallOutput)
                }
                "mcp_call_output" | "mcp" => Ok(IncludeOption::McpCallOutput),
                "inline_citations" | "citations" => Ok(IncludeOption::InlineCitations),
                "verbose_streaming" | "verbose" => Ok(IncludeOption::VerboseStreaming),
                _ => IncludeOption::from_str_name(s)
                    .ok_or_else(|| format!("Invalid include option: '{s}'")),
            }
        }
    }

    impl fmt::Display for ToolCallType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ToolCallType::Invalid => "invalid",
                ToolCallType::ClientSideTool => "client_side_tool",
                ToolCallType::WebSearchTool => "web_search_tool",
                ToolCallType::XSearchTool => "x_search_tool",
                ToolCallType::CodeExecutionTool => "code_execution_tool",
                ToolCallType::CollectionsSearchTool => "collections_search_tool",
                ToolCallType::McpTool => "mcp_tool",
                ToolCallType::AttachmentSearchTool => "attachment_search_tool",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ToolCallType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "invalid" => Ok(ToolCallType::Invalid),
                "client_side_tool" | "clientside" => Ok(ToolCallType::ClientSideTool),
                "web_search_tool" | "websearch" => Ok(ToolCallType::WebSearchTool),
                "x_search_tool" | "xsearch" => Ok(ToolCallType::XSearchTool),
                "code_execution_tool" | "codeexecution" => Ok(ToolCallType::CodeExecutionTool),
                "collections_search_tool" | "collectionssearch" => {
                    Ok(ToolCallType::CollectionsSearchTool)
                }
                "attachment_search_tool" | "attachmentsearch" => {
                    Ok(ToolCallType::AttachmentSearchTool)
                }
                "mcp_tool" | "mcp" => Ok(ToolCallType::McpTool),
                _ => ToolCallType::from_str_name(s)
                    .ok_or_else(|| format!("Invalid tool call type: '{s}'")),
            }
        }
    }

    impl fmt::Display for ToolCallStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ToolCallStatus::InProgress => "in_progress",
                ToolCallStatus::Completed => "completed",
                ToolCallStatus::Incomplete => "incomplete",
                ToolCallStatus::Failed => "failed",
            };
            f.write_str(s)
        }
    }

    impl FromStr for ToolCallStatus {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "in_progress" | "inprogress" => Ok(ToolCallStatus::InProgress),
                "completed" => Ok(ToolCallStatus::Completed),
                "incomplete" => Ok(ToolCallStatus::Incomplete),
                "failed" => Ok(ToolCallStatus::Failed),
                _ => ToolCallStatus::from_str_name(s)
                    .ok_or_else(|| format!("Invalid tool call status: '{s}'")),
            }
        }
    }

    impl fmt::Display for VideoAspectRatio {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                VideoAspectRatio::Unspecified => "unspecified",
                VideoAspectRatio::VideoAspectRatio11 => "1:1",
                VideoAspectRatio::VideoAspectRatio169 => "16:9",
                VideoAspectRatio::VideoAspectRatio916 => "9:16",
                VideoAspectRatio::VideoAspectRatio43 => "4:3",
                VideoAspectRatio::VideoAspectRatio34 => "3:4",
                VideoAspectRatio::VideoAspectRatio32 => "3:2",
                VideoAspectRatio::VideoAspectRatio23 => "2:3",
            };
            f.write_str(s)
        }
    }

    impl FromStr for VideoAspectRatio {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "1:1" | "1x1" => Ok(VideoAspectRatio::VideoAspectRatio11),
                "16:9" | "16x9" => Ok(VideoAspectRatio::VideoAspectRatio169),
                "9:16" | "9x16" => Ok(VideoAspectRatio::VideoAspectRatio916),
                "4:3" | "4x3" => Ok(VideoAspectRatio::VideoAspectRatio43),
                "3:4" | "3x4" => Ok(VideoAspectRatio::VideoAspectRatio34),
                "3:2" | "3x2" => Ok(VideoAspectRatio::VideoAspectRatio32),
                "2:3" | "2x3" => Ok(VideoAspectRatio::VideoAspectRatio23),
                "unspecified" => Ok(VideoAspectRatio::Unspecified),
                _ => VideoAspectRatio::from_str_name(s)
                    .ok_or_else(|| format!("Invalid video aspect ratio: '{s}'")),
            }
        }
    }

    impl fmt::Display for VideoResolution {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                VideoResolution::Unspecified => "unspecified",
                VideoResolution::VideoResolution480p => "480p",
                VideoResolution::VideoResolution720p => "720p",
            };
            f.write_str(s)
        }
    }

    impl FromStr for VideoResolution {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "unspecified" => Ok(VideoResolution::Unspecified),
                "480p" => Ok(VideoResolution::VideoResolution480p),
                "720p" => Ok(VideoResolution::VideoResolution720p),
                _ => VideoResolution::from_str_name(s)
                    .ok_or_else(|| format!("Invalid video resolution: '{s}'")),
            }
        }
    }
}
