pub mod channel {
    use crate::XAI_API_URL;
    use tonic::transport::{Channel, ClientTlsConfig};

    pub async fn new() -> Result<Channel, tonic::transport::Error> {
        Channel::from_static(XAI_API_URL)
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await
    }
}

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
