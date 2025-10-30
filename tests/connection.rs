use xai_sdk::common;

#[tokio::test]
async fn connects_to_xai_api() {
    // Attempt to create a channel using the SDK helper.
    // This validates TLS config and endpoint are well-formed.
    let result = common::channel::new().await;

    assert!(
        result.is_ok(),
        "Expected successful connection, got: {:?}",
        result
    );
}
