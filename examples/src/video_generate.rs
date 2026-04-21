//! Text-to-video: 5 seconds, 480p, 1:1 (smallest footprint among supported presets).
//!
//! ```bash
//! export XAI_API_KEY="your-key"
//! cargo run -p examples --example video_generate
//! ```

use anyhow::{Context, Result};
use std::env;
use std::time::Duration;

use tokio::time::sleep;

use xai_sdk::Request;
use xai_sdk::api::{
    DeferredStatus, GenerateVideoRequest, GetDeferredVideoRequest, VideoAspectRatio,
    VideoResolution,
};
use xai_sdk::video;

/// Video model alias (see xAI docs / model list for current names).
const MODEL: &str = "grok-imagine-video";

const DURATION_SECS: i32 = 5;
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_POLLS: u32 = 120;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    let mut client = video::client::new(&api_key).await?;

    let gen_req = GenerateVideoRequest {
        prompt: "Geneate a video of a hybrid creature consisting of a genetic cross between a gorrila and a salt water crocodile.".into(),
        model: MODEL.into(),
        duration: Some(DURATION_SECS),
        // 1:1 at 480p minimizes pixel count vs wider aspect ratios at the same tier.
        aspect_ratio: Some(VideoAspectRatio::VideoAspectRatio11 as i32),
        resolution: Some(VideoResolution::VideoResolution480p as i32),
        ..Default::default()
    };

    let start = client
        .generate_video(Request::new(gen_req))
        .await
        .context("generate_video RPC failed")?
        .into_inner();

    let request_id = start.request_id;
    println!("Started video job request_id={request_id}");

    for poll in 1..=MAX_POLLS {
        sleep(POLL_INTERVAL).await;

        let poll_resp = client
            .get_deferred_video(Request::new(GetDeferredVideoRequest {
                request_id: request_id.clone(),
            }))
            .await
            .context("get_deferred_video RPC failed")?
            .into_inner();

        let status = match poll_resp.status {
            x if x == DeferredStatus::Done as i32 => DeferredStatus::Done,
            x if x == DeferredStatus::Pending as i32 => DeferredStatus::Pending,
            x if x == DeferredStatus::Failed as i32 => DeferredStatus::Failed,
            x if x == DeferredStatus::Expired as i32 => DeferredStatus::Expired,
            x if x == DeferredStatus::InvalidDeferredStatus as i32 => {
                DeferredStatus::InvalidDeferredStatus
            }
            other => anyhow::bail!("unknown DeferredStatus value {other}"),
        };

        match status {
            DeferredStatus::Pending => {
                if let Some(ref r) = poll_resp.response {
                    println!("poll {poll}: pending (progress {}%)", r.progress);
                } else {
                    println!("poll {poll}: pending");
                }
            }
            DeferredStatus::Done => {
                let video_resp = poll_resp
                    .response
                    .context("status DONE but response missing")?;

                if let Some(err) = video_resp.error {
                    anyhow::bail!("video error: {} — {}", err.code, err.message);
                }

                let v = video_resp
                    .video
                    .context("status DONE but video payload missing")?;

                println!(
                    "done: url={} duration_s={} respect_moderation={}",
                    v.url, v.duration, v.respect_moderation
                );
                return Ok(());
            }
            DeferredStatus::Failed => {
                let detail = poll_resp
                    .response
                    .and_then(|r| r.error)
                    .map(|e| format!("{}: {}", e.code, e.message))
                    .unwrap_or_else(|| "no error details".into());
                anyhow::bail!("job failed: {detail}");
            }
            DeferredStatus::Expired => {
                anyhow::bail!("job expired before completion");
            }
            DeferredStatus::InvalidDeferredStatus => {
                println!(
                    "poll {poll}: unexpected invalid status (raw {})",
                    poll_resp.status
                );
            }
        }
    }

    let secs = POLL_INTERVAL.as_secs() * u64::from(MAX_POLLS);
    anyhow::bail!("timed out after {MAX_POLLS} polls (~{secs}s)");
}
