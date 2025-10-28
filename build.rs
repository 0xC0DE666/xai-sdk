fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir(&"src/")
        .compile_protos(
            &[
                "xai/proto/xai/api/v1/auth.proto",
                "xai/proto/xai/api/v1/chat.proto",
                "xai/proto/xai/api/v1/deferred.proto",
                "xai/proto/xai/api/v1/documents.proto",
                "xai/proto/xai/api/v1/embed.proto",
                "xai/proto/xai/api/v1/image.proto",
                "xai/proto/xai/api/v1/models.proto",
                "xai/proto/xai/api/v1/sample.proto",
                "xai/proto/xai/api/v1/tokenize.proto",
                "xai/proto/xai/api/v1/usage.proto",
            ],
            &["xai/proto"], // Include dir for imports
        )?;
    Ok(())
}
