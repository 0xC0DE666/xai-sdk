fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir(&"src/")
        .compile_protos(
            &[
                "proto/xai/api/v1/auth.proto",
                "proto/xai/api/v1/chat.proto",
                "proto/xai/api/v1/deferred.proto",
                "proto/xai/api/v1/documents.proto",
                "proto/xai/api/v1/embed.proto",
                "proto/xai/api/v1/image.proto",
                "proto/xai/api/v1/models.proto",
                "proto/xai/api/v1/sample.proto",
                "proto/xai/api/v1/tokenize.proto",
                "proto/xai/api/v1/usage.proto",
            ],
            &["proto"], // Include dir for imports
        )?;
    Ok(())
}
