//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options orchestration — LLM call + result processing + entry point.

use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::AgentContext;
use crate::domain::model::template::DEFAULT_OPTION_COUNT;
use crate::error::EngineError;

use super::parser::parse_options;
use crate::application::agents::options::prompt::OptionsPromptBuilder;
use crate::application::agents::options::types::OptionsPromptContext;
use crate::application::ports::llm_provider::AGENT_OPTIONS;

/// Transport failures and zero-parse responses each get one retry, mirroring
/// the quantifier's low-confidence retry.
const MAX_OPTIONS_ATTEMPTS: u32 = 2;

/// Build the options prompt from the current scene, call the options
/// backend, and parse the response into at most [`DEFAULT_OPTION_COUNT`]
/// option strings.
pub fn generate_options(
    ctx: &AgentContext,
    recorder: &LlmCallRecorder,
    options_prompt_override: Option<String>,
) -> Result<Vec<String>, EngineError> {
    let state = ctx.state;
    let current_room = ctx
        .current_room
        .ok_or_else(|| EngineError::RoomNotFound("current room not set in AgentContext".into()))?;

    let recent_history: Vec<_> = state
        .narrative
        .history()
        .iter()
        .rev()
        .take(4)
        .rev()
        .cloned()
        .collect();

    let context = OptionsPromptContext {
        room: current_room,
        npcs_in_area: &state.scene.npcs_in_area,
        recent_history: &recent_history,
        player_name: &ctx.persona.sheet.name,
        options_prompt_override,
        option_count: DEFAULT_OPTION_COUNT,
    };

    options_with_llm_call(&context, recorder)
}

fn options_with_llm_call(
    context: &OptionsPromptContext<'_>,
    recorder: &LlmCallRecorder,
) -> Result<Vec<String>, EngineError> {
    let (system_prompt, user_prompt) = OptionsPromptBuilder::new(context.clone()).build();

    tracing::info!(
        "[Options] Calling backend: {} model: {}",
        recorder.provider().name(),
        recorder.provider().model()
    );

    let mut last_error = None;
    for attempt in 1..=MAX_OPTIONS_ATTEMPTS {
        match options_call_attempt(context, recorder, &system_prompt, &user_prompt) {
            Ok(Some(options)) => return Ok(options),
            Ok(None) => {
                last_error = Some(EngineError::Parse(
                    "options response contained no parseable options".to_string(),
                ));
            }
            Err(e) => last_error = Some(e),
        }
        tracing::info!(
            "[Options] attempt {attempt}/{} exhausted",
            MAX_OPTIONS_ATTEMPTS
        );
    }

    Err(last_error
        .unwrap_or_else(|| EngineError::Parse("options generation produced no result".into())))
}

/// One LLM attempt. `Ok(None)` — transport succeeded but zero options parsed.
fn options_call_attempt(
    context: &OptionsPromptContext<'_>,
    recorder: &LlmCallRecorder,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<Option<Vec<String>>, EngineError> {
    match recorder.complete(AGENT_OPTIONS, system_prompt, user_prompt, None) {
        Ok(llm_result) => {
            let options = parse_options(&llm_result.text, context.option_count);
            tracing::info!("[Options] Parsed {} options", options.len());
            if options.is_empty() {
                Ok(None)
            } else {
                Ok(Some(options))
            }
        }
        Err(e) => {
            tracing::warn!("[Options] LLM call failed: {e}");
            Err(e)
        }
    }
}
