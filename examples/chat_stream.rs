use anyhow::{Context, Result};
use std::env;
use tonic::metadata::MetadataValue;
use tonic::{Request, Streaming};
use xai_sdk::{
    Content, GetChatCompletionChunk, GetCompletionsRequest, Message, MessageRole, chat, content,
};

#[tokio::main]
async fn main() -> Result<()> {
}
