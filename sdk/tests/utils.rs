use std::str::FromStr;
use xai_sdk::utils::enums::*;

// Tests for Display implementations

#[test]
fn test_deferred_status_display() {
    assert_eq!(DeferredStatus::InvalidDeferredStatus.to_string(), "invalid_deferred_status");
    assert_eq!(DeferredStatus::Done.to_string(), "done");
    assert_eq!(DeferredStatus::Expired.to_string(), "expired");
    assert_eq!(DeferredStatus::Pending.to_string(), "pending");
}

#[test]
fn test_deferred_status_from_str() {
    assert_eq!(DeferredStatus::from_str("invalid").unwrap(), DeferredStatus::InvalidDeferredStatus);
    assert_eq!(DeferredStatus::from_str("INVALID").unwrap(), DeferredStatus::InvalidDeferredStatus);
    assert_eq!(DeferredStatus::from_str("done").unwrap(), DeferredStatus::Done);
    assert_eq!(DeferredStatus::from_str("DONE").unwrap(), DeferredStatus::Done);
    assert_eq!(DeferredStatus::from_str("expired").unwrap(), DeferredStatus::Expired);
    assert_eq!(DeferredStatus::from_str("EXPIRED").unwrap(), DeferredStatus::Expired);
    assert_eq!(DeferredStatus::from_str("pending").unwrap(), DeferredStatus::Pending);
    assert_eq!(DeferredStatus::from_str("PENDING").unwrap(), DeferredStatus::Pending);
    assert!(DeferredStatus::from_str("invalid_status").is_err());
}

#[test]
fn test_ranking_metric_display() {
    assert_eq!(RankingMetric::Unknown.to_string(), "unknown");
    assert_eq!(RankingMetric::L2Distance.to_string(), "l2_distance");
    assert_eq!(RankingMetric::CosineSimilarity.to_string(), "cosine_similarity");
}

#[test]
fn test_ranking_metric_from_str() {
    assert_eq!(RankingMetric::from_str("unknown").unwrap(), RankingMetric::Unknown);
    assert_eq!(RankingMetric::from_str("UNKNOWN").unwrap(), RankingMetric::Unknown);
    assert_eq!(RankingMetric::from_str("l2_distance").unwrap(), RankingMetric::L2Distance);
    assert_eq!(RankingMetric::from_str("L2_DISTANCE").unwrap(), RankingMetric::L2Distance);
    assert_eq!(RankingMetric::from_str("l2").unwrap(), RankingMetric::L2Distance);
    assert_eq!(RankingMetric::from_str("L2").unwrap(), RankingMetric::L2Distance);
    assert_eq!(RankingMetric::from_str("cosine_similarity").unwrap(), RankingMetric::CosineSimilarity);
    assert_eq!(RankingMetric::from_str("COSINE_SIMILARITY").unwrap(), RankingMetric::CosineSimilarity);
    assert_eq!(RankingMetric::from_str("cosine").unwrap(), RankingMetric::CosineSimilarity);
    assert_eq!(RankingMetric::from_str("COSINE").unwrap(), RankingMetric::CosineSimilarity);
    assert!(RankingMetric::from_str("invalid").is_err());
}

#[test]
fn test_image_detail_display() {
    assert_eq!(ImageDetail::DetailInvalid.to_string(), "invalid");
    assert_eq!(ImageDetail::DetailAuto.to_string(), "auto");
    assert_eq!(ImageDetail::DetailLow.to_string(), "low");
    assert_eq!(ImageDetail::DetailHigh.to_string(), "high");
}

#[test]
fn test_image_detail_from_str() {
    assert_eq!(ImageDetail::from_str("invalid").unwrap(), ImageDetail::DetailInvalid);
    assert_eq!(ImageDetail::from_str("INVALID").unwrap(), ImageDetail::DetailInvalid);
    assert_eq!(ImageDetail::from_str("auto").unwrap(), ImageDetail::DetailAuto);
    assert_eq!(ImageDetail::from_str("AUTO").unwrap(), ImageDetail::DetailAuto);
    assert_eq!(ImageDetail::from_str("low").unwrap(), ImageDetail::DetailLow);
    assert_eq!(ImageDetail::from_str("LOW").unwrap(), ImageDetail::DetailLow);
    assert_eq!(ImageDetail::from_str("high").unwrap(), ImageDetail::DetailHigh);
    assert_eq!(ImageDetail::from_str("HIGH").unwrap(), ImageDetail::DetailHigh);
    assert!(ImageDetail::from_str("invalid_detail").is_err());
}

#[test]
fn test_image_format_display() {
    assert_eq!(ImageFormat::ImgFormatInvalid.to_string(), "invalid");
    assert_eq!(ImageFormat::ImgFormatBase64.to_string(), "base64");
    assert_eq!(ImageFormat::ImgFormatUrl.to_string(), "url");
}

#[test]
fn test_image_format_from_str() {
    assert_eq!(ImageFormat::from_str("invalid").unwrap(), ImageFormat::ImgFormatInvalid);
    assert_eq!(ImageFormat::from_str("INVALID").unwrap(), ImageFormat::ImgFormatInvalid);
    assert_eq!(ImageFormat::from_str("base64").unwrap(), ImageFormat::ImgFormatBase64);
    assert_eq!(ImageFormat::from_str("BASE64").unwrap(), ImageFormat::ImgFormatBase64);
    assert_eq!(ImageFormat::from_str("url").unwrap(), ImageFormat::ImgFormatUrl);
    assert_eq!(ImageFormat::from_str("URL").unwrap(), ImageFormat::ImgFormatUrl);
    assert!(ImageFormat::from_str("invalid_format").is_err());
}

#[test]
fn test_finish_reason_display() {
    assert_eq!(FinishReason::ReasonInvalid.to_string(), "invalid");
    assert_eq!(FinishReason::ReasonMaxLen.to_string(), "max_len");
    assert_eq!(FinishReason::ReasonMaxContext.to_string(), "max_context");
    assert_eq!(FinishReason::ReasonStop.to_string(), "stop");
    assert_eq!(FinishReason::ReasonToolCalls.to_string(), "tool_calls");
    assert_eq!(FinishReason::ReasonTimeLimit.to_string(), "time_limit");
}

#[test]
fn test_finish_reason_from_str() {
    assert_eq!(FinishReason::from_str("invalid").unwrap(), FinishReason::ReasonInvalid);
    assert_eq!(FinishReason::from_str("INVALID").unwrap(), FinishReason::ReasonInvalid);
    assert_eq!(FinishReason::from_str("max_len").unwrap(), FinishReason::ReasonMaxLen);
    assert_eq!(FinishReason::from_str("MAX_LEN").unwrap(), FinishReason::ReasonMaxLen);
    assert_eq!(FinishReason::from_str("max_context").unwrap(), FinishReason::ReasonMaxContext);
    assert_eq!(FinishReason::from_str("MAX_CONTEXT").unwrap(), FinishReason::ReasonMaxContext);
    assert_eq!(FinishReason::from_str("stop").unwrap(), FinishReason::ReasonStop);
    assert_eq!(FinishReason::from_str("STOP").unwrap(), FinishReason::ReasonStop);
    assert_eq!(FinishReason::from_str("tool_calls").unwrap(), FinishReason::ReasonToolCalls);
    assert_eq!(FinishReason::from_str("TOOL_CALLS").unwrap(), FinishReason::ReasonToolCalls);
    assert_eq!(FinishReason::from_str("time_limit").unwrap(), FinishReason::ReasonTimeLimit);
    assert_eq!(FinishReason::from_str("TIME_LIMIT").unwrap(), FinishReason::ReasonTimeLimit);
    assert!(FinishReason::from_str("invalid_reason").is_err());
}

#[test]
fn test_message_role_display() {
    assert_eq!(MessageRole::InvalidRole.to_string(), "invalid");
    assert_eq!(MessageRole::RoleUser.to_string(), "user");
    assert_eq!(MessageRole::RoleAssistant.to_string(), "assistant");
    assert_eq!(MessageRole::RoleSystem.to_string(), "system");
    assert_eq!(MessageRole::RoleFunction.to_string(), "function");
    assert_eq!(MessageRole::RoleTool.to_string(), "tool");
    assert_eq!(MessageRole::RoleDeveloper.to_string(), "developer");
}

#[test]
fn test_message_role_from_str() {
    assert_eq!(MessageRole::from_str("invalid").unwrap(), MessageRole::InvalidRole);
    assert_eq!(MessageRole::from_str("INVALID").unwrap(), MessageRole::InvalidRole);
    assert_eq!(MessageRole::from_str("user").unwrap(), MessageRole::RoleUser);
    assert_eq!(MessageRole::from_str("USER").unwrap(), MessageRole::RoleUser);
    assert_eq!(MessageRole::from_str("assistant").unwrap(), MessageRole::RoleAssistant);
    assert_eq!(MessageRole::from_str("ASSISTANT").unwrap(), MessageRole::RoleAssistant);
    assert_eq!(MessageRole::from_str("system").unwrap(), MessageRole::RoleSystem);
    assert_eq!(MessageRole::from_str("SYSTEM").unwrap(), MessageRole::RoleSystem);
    assert_eq!(MessageRole::from_str("function").unwrap(), MessageRole::RoleFunction);
    assert_eq!(MessageRole::from_str("FUNCTION").unwrap(), MessageRole::RoleFunction);
    assert_eq!(MessageRole::from_str("tool").unwrap(), MessageRole::RoleTool);
    assert_eq!(MessageRole::from_str("TOOL").unwrap(), MessageRole::RoleTool);
    assert_eq!(MessageRole::from_str("developer").unwrap(), MessageRole::RoleDeveloper);
    assert_eq!(MessageRole::from_str("DEVELOPER").unwrap(), MessageRole::RoleDeveloper);
    // Note: from_str_name might parse some variants, so use a definitely invalid string
    assert!(MessageRole::from_str("definitely_not_a_role_xyz123").is_err());
}

#[test]
fn test_reasoning_effort_display() {
    assert_eq!(ReasoningEffort::InvalidEffort.to_string(), "invalid");
    assert_eq!(ReasoningEffort::EffortLow.to_string(), "low");
    assert_eq!(ReasoningEffort::EffortMedium.to_string(), "medium");
    assert_eq!(ReasoningEffort::EffortHigh.to_string(), "high");
}

#[test]
fn test_reasoning_effort_from_str() {
    assert_eq!(ReasoningEffort::from_str("invalid").unwrap(), ReasoningEffort::InvalidEffort);
    assert_eq!(ReasoningEffort::from_str("INVALID").unwrap(), ReasoningEffort::InvalidEffort);
    assert_eq!(ReasoningEffort::from_str("low").unwrap(), ReasoningEffort::EffortLow);
    assert_eq!(ReasoningEffort::from_str("LOW").unwrap(), ReasoningEffort::EffortLow);
    assert_eq!(ReasoningEffort::from_str("medium").unwrap(), ReasoningEffort::EffortMedium);
    assert_eq!(ReasoningEffort::from_str("MEDIUM").unwrap(), ReasoningEffort::EffortMedium);
    assert_eq!(ReasoningEffort::from_str("high").unwrap(), ReasoningEffort::EffortHigh);
    assert_eq!(ReasoningEffort::from_str("HIGH").unwrap(), ReasoningEffort::EffortHigh);
    // Note: from_str_name might parse some variants, so use a definitely invalid string
    assert!(ReasoningEffort::from_str("definitely_not_an_effort_xyz123").is_err());
}

#[test]
fn test_tool_mode_display() {
    assert_eq!(ToolMode::Invalid.to_string(), "invalid");
    assert_eq!(ToolMode::Auto.to_string(), "auto");
    assert_eq!(ToolMode::None.to_string(), "none");
    assert_eq!(ToolMode::Required.to_string(), "required");
}

#[test]
fn test_tool_mode_from_str() {
    assert_eq!(ToolMode::from_str("invalid").unwrap(), ToolMode::Invalid);
    assert_eq!(ToolMode::from_str("INVALID").unwrap(), ToolMode::Invalid);
    assert_eq!(ToolMode::from_str("auto").unwrap(), ToolMode::Auto);
    assert_eq!(ToolMode::from_str("AUTO").unwrap(), ToolMode::Auto);
    assert_eq!(ToolMode::from_str("none").unwrap(), ToolMode::None);
    assert_eq!(ToolMode::from_str("NONE").unwrap(), ToolMode::None);
    assert_eq!(ToolMode::from_str("required").unwrap(), ToolMode::Required);
    assert_eq!(ToolMode::from_str("REQUIRED").unwrap(), ToolMode::Required);
    assert!(ToolMode::from_str("invalid_mode").is_err());
}

#[test]
fn test_format_type_display() {
    assert_eq!(FormatType::Invalid.to_string(), "invalid");
    assert_eq!(FormatType::Text.to_string(), "text");
    assert_eq!(FormatType::JsonObject.to_string(), "json_object");
    assert_eq!(FormatType::JsonSchema.to_string(), "json_schema");
}

#[test]
fn test_format_type_from_str() {
    assert_eq!(FormatType::from_str("invalid").unwrap(), FormatType::Invalid);
    assert_eq!(FormatType::from_str("INVALID").unwrap(), FormatType::Invalid);
    assert_eq!(FormatType::from_str("text").unwrap(), FormatType::Text);
    assert_eq!(FormatType::from_str("TEXT").unwrap(), FormatType::Text);
    assert_eq!(FormatType::from_str("json_object").unwrap(), FormatType::JsonObject);
    assert_eq!(FormatType::from_str("JSON_OBJECT").unwrap(), FormatType::JsonObject);
    assert_eq!(FormatType::from_str("json").unwrap(), FormatType::JsonObject);
    assert_eq!(FormatType::from_str("JSON").unwrap(), FormatType::JsonObject);
    assert_eq!(FormatType::from_str("json_schema").unwrap(), FormatType::JsonSchema);
    assert_eq!(FormatType::from_str("JSON_SCHEMA").unwrap(), FormatType::JsonSchema);
    assert_eq!(FormatType::from_str("schema").unwrap(), FormatType::JsonSchema);
    assert_eq!(FormatType::from_str("SCHEMA").unwrap(), FormatType::JsonSchema);
    assert!(FormatType::from_str("invalid_format").is_err());
}

#[test]
fn test_search_mode_display() {
    assert_eq!(SearchMode::InvalidSearchMode.to_string(), "invalid");
    assert_eq!(SearchMode::OffSearchMode.to_string(), "off");
    assert_eq!(SearchMode::OnSearchMode.to_string(), "on");
    assert_eq!(SearchMode::AutoSearchMode.to_string(), "auto");
}

#[test]
fn test_search_mode_from_str() {
    assert_eq!(SearchMode::from_str("invalid").unwrap(), SearchMode::InvalidSearchMode);
    assert_eq!(SearchMode::from_str("INVALID").unwrap(), SearchMode::InvalidSearchMode);
    assert_eq!(SearchMode::from_str("off").unwrap(), SearchMode::OffSearchMode);
    assert_eq!(SearchMode::from_str("OFF").unwrap(), SearchMode::OffSearchMode);
    assert_eq!(SearchMode::from_str("on").unwrap(), SearchMode::OnSearchMode);
    assert_eq!(SearchMode::from_str("ON").unwrap(), SearchMode::OnSearchMode);
    assert_eq!(SearchMode::from_str("auto").unwrap(), SearchMode::AutoSearchMode);
    assert_eq!(SearchMode::from_str("AUTO").unwrap(), SearchMode::AutoSearchMode);
    assert!(SearchMode::from_str("invalid_mode").is_err());
}

#[test]
fn test_embed_encoding_format_display() {
    assert_eq!(EmbedEncodingFormat::FormatInvalid.to_string(), "invalid");
    assert_eq!(EmbedEncodingFormat::FormatFloat.to_string(), "float");
    assert_eq!(EmbedEncodingFormat::FormatBase64.to_string(), "base64");
}

#[test]
fn test_embed_encoding_format_from_str() {
    assert_eq!(EmbedEncodingFormat::from_str("invalid").unwrap(), EmbedEncodingFormat::FormatInvalid);
    assert_eq!(EmbedEncodingFormat::from_str("INVALID").unwrap(), EmbedEncodingFormat::FormatInvalid);
    assert_eq!(EmbedEncodingFormat::from_str("float").unwrap(), EmbedEncodingFormat::FormatFloat);
    assert_eq!(EmbedEncodingFormat::from_str("FLOAT").unwrap(), EmbedEncodingFormat::FormatFloat);
    assert_eq!(EmbedEncodingFormat::from_str("base64").unwrap(), EmbedEncodingFormat::FormatBase64);
    assert_eq!(EmbedEncodingFormat::from_str("BASE64").unwrap(), EmbedEncodingFormat::FormatBase64);
    assert!(EmbedEncodingFormat::from_str("invalid_format").is_err());
}

#[test]
fn test_modality_display() {
    assert_eq!(Modality::InvalidModality.to_string(), "invalid");
    assert_eq!(Modality::Text.to_string(), "text");
    assert_eq!(Modality::Image.to_string(), "image");
    assert_eq!(Modality::Embedding.to_string(), "embedding");
}

#[test]
fn test_modality_from_str() {
    assert_eq!(Modality::from_str("invalid").unwrap(), Modality::InvalidModality);
    assert_eq!(Modality::from_str("INVALID").unwrap(), Modality::InvalidModality);
    assert_eq!(Modality::from_str("text").unwrap(), Modality::Text);
    assert_eq!(Modality::from_str("TEXT").unwrap(), Modality::Text);
    assert_eq!(Modality::from_str("image").unwrap(), Modality::Image);
    assert_eq!(Modality::from_str("IMAGE").unwrap(), Modality::Image);
    assert_eq!(Modality::from_str("embedding").unwrap(), Modality::Embedding);
    assert_eq!(Modality::from_str("EMBEDDING").unwrap(), Modality::Embedding);
    // Note: from_str_name might parse some variants, so use a definitely invalid string
    assert!(Modality::from_str("definitely_not_a_modality_xyz123").is_err());
}

// Test round-trip: Display -> FromStr
#[test]
fn test_display_from_str_roundtrip() {
    // Test that we can convert to string and back
    let status = DeferredStatus::Done;
    let s = status.to_string();
    assert_eq!(DeferredStatus::from_str(&s).unwrap(), status);

    let metric = RankingMetric::L2Distance;
    let s = metric.to_string();
    assert_eq!(RankingMetric::from_str(&s).unwrap(), metric);

    let detail = ImageDetail::DetailHigh;
    let s = detail.to_string();
    assert_eq!(ImageDetail::from_str(&s).unwrap(), detail);

    let format = ImageFormat::ImgFormatBase64;
    let s = format.to_string();
    assert_eq!(ImageFormat::from_str(&s).unwrap(), format);

    let reason = FinishReason::ReasonStop;
    let s = reason.to_string();
    assert_eq!(FinishReason::from_str(&s).unwrap(), reason);

    let role = MessageRole::RoleAssistant;
    let s = role.to_string();
    assert_eq!(MessageRole::from_str(&s).unwrap(), role);

    let effort = ReasoningEffort::EffortHigh;
    let s = effort.to_string();
    assert_eq!(ReasoningEffort::from_str(&s).unwrap(), effort);

    let mode = ToolMode::Auto;
    let s = mode.to_string();
    assert_eq!(ToolMode::from_str(&s).unwrap(), mode);

    let format_type = FormatType::JsonObject;
    let s = format_type.to_string();
    assert_eq!(FormatType::from_str(&s).unwrap(), format_type);

    let search = SearchMode::AutoSearchMode;
    let s = search.to_string();
    assert_eq!(SearchMode::from_str(&s).unwrap(), search);

    let encoding = EmbedEncodingFormat::FormatFloat;
    let s = encoding.to_string();
    assert_eq!(EmbedEncodingFormat::from_str(&s).unwrap(), encoding);

    let modality = Modality::Text;
    let s = modality.to_string();
    assert_eq!(Modality::from_str(&s).unwrap(), modality);
}
