#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzzing JWT Decoder with arbitrary bytes (simulating corrupted headers/payloads)
    if let Ok(token) = std::str::from_utf8(data) {
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { lightx::core::verify_jwt(token).await });
    }

    // 2. Fuzzing JSON serialization (simulating malformed error payloads)
    if data.len() > 10 {
        let error_msg = String::from_utf8_lossy(&data[..10]).to_string();
        let app_error = lightx::core::AppError::ParameterError {
            field: "fuzz".into(),
            msg: error_msg.into(),
            file: "fuzz_target_1.rs",
            line: 42,
        };
        let _ = format!("{:?}", app_error);
    }
});
