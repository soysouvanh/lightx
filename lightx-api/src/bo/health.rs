use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;

pub async fn execute_check(
    _ctx: &mut crate::RequestContext,
) -> Result<Response<Full<Bytes>>, AppError> {
    let json_body = serde_json::json!({
        "status": "ok",
        "message": "LightX API is running smoothly",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Serialization to raw bytes (Zero-Allocation JSON parsing logic)
    let bytes = Bytes::from(json_body.to_string());

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Full::new(bytes))
        .unwrap();

    Ok(response)
}
