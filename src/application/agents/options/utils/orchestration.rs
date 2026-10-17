//! [DOC: docs/diataxis/reference/narrative/options.md]
//! Options orchestration — LLM call + result processing + entry point.

use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::{AgentSettings, HookInput};
use crate::domain::model::template::DEFAULT_OPTION_COUNT;
use crate::error::EngineError;

use super::parser::parse_options;
use crate::application::agents::options::prompt::OptionsPromptBuilder;
use crate::application::agents::options::types::OptionsPromptContext;
use crate::application::ports::llm_provider::AGENT_OPTIONS;

/// Transport failures and zero-parse responses each get one retry, mirroring
/// the quantifier's low-confidence retry.
const MAX_OPTIONS_ATTEMPTS: u32 = 2;

pub fn generate_options(
    input: &HookInput,
    recorder: &LlmCallRecorder,
    settings: &AgentSettings,
) -> Result<Vec<String>, EngineError> {
    let world = input.world;
    let current_room = world.room.ok_or_else(|| {
        EngineError::RoomNotFound("current room not set in the hook input".into())
    })?;

    let context = OptionsPromptContext {
        room: current_room,
        npcs_in_area: world.present,
        recent_history: input.story,
        player_name: &world.persona.sheet.name,
        options_prompt_override: settings.preset.clone(),
        option_count: DEFAULT_OPTION_COUNT,
    };

    options_with_llm_call(&context, recorder, settings.max_tokens)
}

fn options_with_llm_call(
    context: &OptionsPromptContext<'_>,
    recorder: &LlmCallRecorder,
    max_tokens: Option<u32>,
) -> Result<Vec<String>, EngineError> {
    let (system_prompt, user_prompt) = OptionsPromptBuilder::new(context.clone()).build();

    tracing::info!("[Options] Calling provider: {}", recorder.provider_label());

    let mut last_error = None;
    for attempt in 1..=MAX_OPTIONS_ATTEMPTS {
        match options_call_attempt(context, recorder, &system_prompt, &user_prompt, max_tokens) {
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

fn options_call_attempt(
    context: &OptionsPromptContext<'_>,
    recorder: &LlmCallRecorder,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: Option<u32>,
) -> Result<Option<Vec<String>>, EngineError> {
    match recorder.complete(AGENT_OPTIONS, system_prompt, user_prompt, max_tokens) {
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
