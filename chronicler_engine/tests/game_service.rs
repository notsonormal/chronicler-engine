//! [DOC: docs/reference/testing.md]

#[path = "game_service/advanced.rs"]
mod advanced;
#[path = "game_service/basic.rs"]
mod basic;
mod test_data;

#[path = "helpers/game_service.rs"]
mod game_service_helpers;

use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::narrative::llm::MockBackend;

pub fn failing_service() -> DefaultGameService {
    DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    )
}
