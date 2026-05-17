//! Sequential flow tests using direct service calls with mock backends.

mod test_data;

#[path = "helpers/game_service.rs"]
mod game_service_helpers;

#[path = "flow_mock/retry_event.rs"]
mod retry_event;
#[path = "flow_mock/retry_main.rs"]
mod retry_main;
#[path = "flow_mock/sequence.rs"]
mod sequence;
