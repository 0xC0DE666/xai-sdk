fn main() {
    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .out_dir("src/")
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
            &["xai/proto"],
        )
        .unwrap();
}
