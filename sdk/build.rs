fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to rerun this build script if any proto files change
    println!("cargo:rerun-if-changed=xai/proto");

    // Also rerun if this build script changes
    println!("cargo:rerun-if-changed=build.rs");

    let includes = &["xai/proto"];
    let protos = &[
        "xai/proto/xai/api/v1/auth.proto",
        "xai/proto/xai/api/v1/batch.proto",
        "xai/proto/xai/api/v1/chat.proto",
        "xai/proto/xai/api/v1/deferred.proto",
        "xai/proto/xai/api/v1/documents.proto",
        "xai/proto/xai/api/v1/embed.proto",
        "xai/proto/xai/api/v1/files.proto",
        "xai/proto/xai/api/v1/image.proto",
        "xai/proto/xai/api/v1/models.proto",
        "xai/proto/xai/api/v1/sample.proto",
        "xai/proto/xai/api/v1/tokenize.proto",
        "xai/proto/xai/api/v1/usage.proto",
        "xai/proto/xai/api/v1/video.proto",
        //
        "xai/proto/xai/management_api/v1/billing.proto",
    ];

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .emit_rerun_if_changed(true)
        .out_dir(&"src/")
        .compile_well_known_types(false)
        .compile_protos(protos, includes)?;

    Ok(())
}
