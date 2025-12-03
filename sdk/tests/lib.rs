use xai_sdk::common::interceptor::{ClientInterceptor, auth, compose};
use xai_sdk::export::service::Interceptor;
use xai_sdk::{Request, Status};

#[test]
fn test_client_interceptor_new() {
    // Test creating ClientInterceptor from a closure
    let mut interceptor =
        ClientInterceptor::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("test-header", "test-value".parse().unwrap());
            Ok(req)
        });

    let request = Request::new(());
    let result = interceptor.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.metadata().get("test-header").unwrap(), "test-value");
}

#[test]
fn test_client_interceptor_from_boxed() {
    // Test From<Box<dyn Interceptor>> implementation
    let boxed: Box<dyn Interceptor> =
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("from-header", "from-value".parse().unwrap());
            Ok(req)
        });

    let mut interceptor = ClientInterceptor::from(boxed);
    let request = Request::new(());
    let result = interceptor.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.metadata().get("from-header").unwrap(), "from-value");
}

#[test]
fn test_client_interceptor_into() {
    // Test Into trait (via From)
    let boxed: Box<dyn Interceptor> =
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("into-header", "into-value".parse().unwrap());
            Ok(req)
        });

    let mut interceptor: ClientInterceptor = boxed.into();
    let request = Request::new(());
    let result = interceptor.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.metadata().get("into-header").unwrap(), "into-value");
}

#[test]
fn test_auth_interceptor() {
    // Test auth() function adds correct authorization header
    let api_key = "test-api-key-12345";
    let mut interceptor = auth(api_key);

    let request = Request::new(());
    let result = interceptor.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();

    // Check that authorization header exists
    let auth_header = request.metadata().get("authorization");
    assert!(auth_header.is_some());

    // Check that it has the correct format
    let auth_value = auth_header.unwrap().to_str().unwrap();
    assert_eq!(auth_value, format!("Bearer {}", api_key));
}

#[test]
fn test_auth_interceptor_different_keys() {
    // Test auth() with different API keys
    let keys = vec!["key1", "key2", "very-long-api-key-with-special-chars-12345"];

    for key in keys {
        let mut interceptor = auth(key);
        let request = Request::new(());
        let result = interceptor.call(request).unwrap();

        let auth_value = result
            .metadata()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth_value, format!("Bearer {}", key));
    }
}

#[test]
fn test_compose_single_interceptor() {
    // Test compose with a single interceptor
    let interceptors: Vec<Box<dyn Interceptor>> = vec![Box::new(
        |mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("single", "value".parse().unwrap());
            Ok(req)
        },
    )];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.metadata().get("single").unwrap(), "value");
}

#[test]
fn test_compose_multiple_interceptors() {
    // Test compose with multiple interceptors - should apply in order
    let interceptors: Vec<Box<dyn Interceptor>> = vec![
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut().insert("first", "1".parse().unwrap());
            Ok(req)
        }),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut().insert("second", "2".parse().unwrap());
            Ok(req)
        }),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut().insert("third", "3".parse().unwrap());
            Ok(req)
        }),
    ];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();

    // All headers should be present
    assert_eq!(request.metadata().get("first").unwrap(), "1");
    assert_eq!(request.metadata().get("second").unwrap(), "2");
    assert_eq!(request.metadata().get("third").unwrap(), "3");
}

#[test]
fn test_compose_interceptors_modify_same_header() {
    // Test that later interceptors can modify headers set by earlier ones
    let interceptors: Vec<Box<dyn Interceptor>> = vec![
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("trace-id", "original".parse().unwrap());
            Ok(req)
        }),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            // This should overwrite the previous trace-id
            req.metadata_mut()
                .insert("trace-id", "overwritten".parse().unwrap());
            Ok(req)
        }),
    ];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();

    // Should have the overwritten value
    assert_eq!(request.metadata().get("trace-id").unwrap(), "overwritten");
}

#[test]
fn test_compose_with_auth() {
    // Test compose with auth interceptor
    let interceptors: Vec<Box<dyn Interceptor>> = vec![
        Box::new(auth("test-key")),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("custom", "header".parse().unwrap());
            Ok(req)
        }),
    ];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();

    // Both headers should be present
    assert!(request.metadata().get("authorization").is_some());
    assert_eq!(request.metadata().get("custom").unwrap(), "header");
}

#[test]
fn test_compose_error_propagation() {
    // Test that errors from interceptors are propagated
    let interceptors: Vec<Box<dyn Interceptor>> = vec![
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("before-error", "value".parse().unwrap());
            Ok(req)
        }),
        Box::new(|_req: Request<()>| -> Result<Request<()>, Status> {
            Err(Status::invalid_argument("test error"))
        }),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            // This should never be called
            req.metadata_mut()
                .insert("after-error", "value".parse().unwrap());
            Ok(req)
        }),
    ];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    // Should return error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert_eq!(error.message(), "test error");
}

#[test]
fn test_compose_empty_vec() {
    // Test compose with empty vector - should pass through unchanged
    let interceptors: Vec<Box<dyn Interceptor>> = vec![];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    // Request should be unchanged
    let request = result.unwrap();
    assert_eq!(request.metadata().len(), 0);
}

#[test]
fn test_client_interceptor_reusable() {
    // Test that ClientInterceptor can be called multiple times
    let mut interceptor =
        ClientInterceptor::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            let count = req
                .metadata()
                .get("count")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            req.metadata_mut()
                .insert("count", (count + 1).to_string().parse().unwrap());
            Ok(req)
        });

    let mut request = Request::new(());
    request.metadata_mut().insert("count", "0".parse().unwrap());

    // Call multiple times
    request = interceptor.call(request).unwrap();
    request = interceptor.call(request).unwrap();
    request = interceptor.call(request).unwrap();

    let count = request.metadata().get("count").unwrap().to_str().unwrap();
    assert_eq!(count, "3");
}

#[test]
fn test_auth_with_compose_realistic() {
    // Test a realistic scenario: auth + trace-id + tenant-id
    let api_key = "real-api-key-123";
    let interceptors: Vec<Box<dyn Interceptor>> = vec![
        Box::new(auth(api_key)),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("x-trace-id", "trace-abc123".parse().unwrap());
            Ok(req)
        }),
        Box::new(|mut req: Request<()>| -> Result<Request<()>, Status> {
            req.metadata_mut()
                .insert("x-tenant-id", "tenant-42".parse().unwrap());
            Ok(req)
        }),
    ];

    let mut composed = compose(interceptors);
    let request = Request::new(());
    let result = composed.call(request);

    assert!(result.is_ok());
    let request = result.unwrap();

    // Verify all headers are present
    assert!(request.metadata().get("authorization").is_some());
    assert_eq!(
        request.metadata().get("x-trace-id").unwrap(),
        "trace-abc123"
    );
    assert_eq!(request.metadata().get("x-tenant-id").unwrap(), "tenant-42");

    // Verify auth header format
    let auth_value = request
        .metadata()
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(auth_value, format!("Bearer {}", api_key));
}
