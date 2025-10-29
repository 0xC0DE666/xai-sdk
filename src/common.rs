pub mod interceptor {
    use tonic::Request;
    use tonic::Status;
    use tonic::metadata::MetadataValue;
    use tonic::service::Interceptor;

    pub fn auth(api_key: &str) -> impl Interceptor {
        move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let token = MetadataValue::try_from(&format!("Bearer {}", api_key)).map_err(|e| {
                Status::invalid_argument(format!("Failed to create metadata value: {}", e))
            })?;

            req.metadata_mut().insert("authorization", token);

            Ok(req)
        }
    }
}
