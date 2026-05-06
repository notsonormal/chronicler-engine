use crate::error::EngineError;

#[test]
fn test_engine_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let engine_err: EngineError = io_err.into();
    match engine_err {
        EngineError::Io(msg) => {
            assert!(
                msg.contains("file not found"),
                "Expected error message to contain 'file not found', got: {msg}"
            );
        }
        other => panic!("Expected EngineError::Io, got: {other:?}"),
    }
}

#[test]
fn test_engine_error_display_variants() {
    let err = EngineError::Io("disk full".to_string());
    assert!(err.to_string().contains("disk full"));

    let err = EngineError::Parse("bad json".to_string());
    assert!(err.to_string().contains("bad json"));

    let err = EngineError::LlmEmptyResponse;
    assert!(err.to_string().contains("empty response"));

    let err = EngineError::ContextOverflow {
        requested: 100,
        max: 50,
    };
    assert!(err.to_string().contains("100"));
    assert!(err.to_string().contains("50"));
}
