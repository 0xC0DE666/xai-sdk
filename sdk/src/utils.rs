/// Module containing re-exported enums from xai_api with extended functionality.
pub mod enums {
    use std::fmt;
    use std::str::FromStr;

    // Re-export all enums
    pub use crate::xai_api::{
        DeferredStatus, EmbedEncodingFormat, FinishReason, FormatType, ImageDetail, ImageFormat,
        MessageRole, Modality, RankingMetric, ReasoningEffort, SearchMode, ToolMode,
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
}
