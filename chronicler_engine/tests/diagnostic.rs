#![allow(clippy::uninlined_format_args)]

//! Diagnostic Signal Quality Benchmark
//!
//! Run via: cargo test --test diagnostic -- --nocapture
//! Or via: python scripts/diagnostic_benchmark.py

#![allow(dead_code)]

#[path = "diagnostic/backends.rs"]
mod backends;
#[path = "diagnostic/scenarios.rs"]
mod scenarios;

mod test_data;

use std::sync::Arc;

use chronicler_engine::engine::game_service::{
    DefaultGameService, GameService, GameServiceContext,
};

use chronicler_engine::model::state::GenerationStatus;
use chronicler_engine::narrative::llm::backend::LlmBackend;
use chronicler_engine::test_support::make_test_context;

#[derive(Debug, serde::Serialize)]
pub struct BenchmarkResult {
    pub scenario: String,
    pub category: String,
    pub injected_failure: String,
    pub error_message: String,
    pub generation_phase: String,
    pub scores: DiagnosticScores,
    pub root_cause_discoverable_from_ui: bool,
    pub root_cause_discoverable_from_debug_endpoint: bool,
    pub root_cause_discoverable_without_logs: bool,
    pub notes: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DiagnosticScores {
    pub error_specificity: u8,
    pub state_visibility: u8,
    pub log_independence: u8,
}

pub fn print_benchmark_result(result: &BenchmarkResult) {
    let json = serde_json::to_string(result).unwrap();
    println!("BENCHMARK_RESULT:{json}");
}

pub fn run_scenario(
    llm_backend: Arc<dyn LlmBackend>,
    quantifier_backend: Arc<
        dyn chronicler_engine::narrative::agents::quantifier::backends::QuantifierBackendTrait,
    >,
    _scenario_name: &str,
    _category: &str,
    _injected_failure: &str,
) -> (String, String, GameServiceContext) {
    let service = DefaultGameService::with_mock_quantifier(llm_backend, quantifier_backend);
    let state = test_data::create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(
        ctx.clone(),
        "look around".to_string(),
        "Test Player".to_string(),
    );

    let snapshot = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
    let error_message = match &snapshot.narrative.generation.status {
        GenerationStatus::Error(msg) => msg.clone(),
        GenerationStatus::Idle => "(no error, idle)".to_string(),
        GenerationStatus::Generating => "(still generating)".to_string(),
    };
    let phase = format!("{:?}", snapshot.narrative.generation.phase);

    (error_message, phase, ctx)
}
