//! [DOC: docs/diataxis/reference/game_flow.md]
//! Shared action-pipeline state, constructors, and orchestration helpers.

use std::collections::HashMap;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::application::pipeline::phase_error::PhaseError;
use crate::application::pipeline::narration_generation;
use crate::application::pipeline::pipeline_run::PipelineRun;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::Storage;

use crate::domain::model::agent::Kind;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::map::MapDef;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::{ActionResult, FreeActionContext, GameState};
use crate::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, GenerationStatus, Step,
};

use crate::application::errors::ProcessActionResult;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::generation::gate::GenerationGate;
use crate::application::prompting::PromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::AgentRunner;
use crate::application::message_service::MessageService;
use crate::application::narration::Narrator;
use crate::application::ports::llm_provider::LlmProvider;
use crate::error::EngineError;

#[derive(Clone)]
pub struct ActionPipeline {
    pub(crate) narrator: Arc<Narrator>,
    pub(crate) recorder: Arc<LlmCallRecorder>,
    pub(crate) agent_runner: Arc<AgentRunner>,
    pub(crate) message_service: Arc<MessageService>,
    pub(crate) storage: Arc<Storage>,
    pub(crate) game_catalogue: Arc<GameCatalogue>,
    pub(crate) shutdown_token: CancellationToken,
}

impl ActionPipeline {
    pub fn with_storage(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_runner: AgentRunner,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        game_catalogue: Arc<GameCatalogue>,
    ) -> Self {
        tracing::info!("ActionPipeline: provider={}", recorder.provider_label());
        Self::with_backends(
            shutdown_token,
            recorder,
            agent_runner,
            message_service,
            storage,
            game_catalogue,
        )
    }

    pub fn with_backends(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_runner: AgentRunner,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        game_catalogue: Arc<GameCatalogue>,
    ) -> Self {
        let narrator = Arc::new(Narrator::new(PromptAssembler, Arc::clone(&recorder)));
        Self {
            narrator,
            recorder,
            agent_runner: Arc::new(agent_runner),
            message_service,
            storage,
            game_catalogue,
            shutdown_token,
        }
    }

    pub fn with_mock_quantifier(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        quantifier_provider: Arc<dyn LlmProvider>,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        game_catalogue: Arc<GameCatalogue>,
    ) -> Self {
        let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
        let runner = AgentRunner::with_analysis_agent(Box::new(agent));
        Self::with_backends(
            shutdown_token,
            recorder,
            runner,
            message_service,
            storage,
            game_catalogue,
        )
    }

    pub fn backend_info(&self) -> (String, String) {
        match self.recorder.provider() {
            Ok(p) => (p.name().to_string(), p.model().to_string()),
            Err(e) => (format!("<unresolved: {e}>"), "<unresolved>".to_string()),
        }
    }

    pub fn recorder(&self) -> &Arc<LlmCallRecorder> {
        &self.recorder
    }

    pub fn rebind_for_test(
        mut self,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        game_catalogue: Arc<GameCatalogue>,
        shutdown_token: CancellationToken,
    ) -> Self {
        self.message_service = message_service;
        self.storage = storage;
        self.game_catalogue = game_catalogue;
        self.shutdown_token = shutdown_token;
        self
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }

    pub fn reset_persisted_status(&self, game_id: GameId) -> Result<(), EngineError> {
        let mut game_state = self.message_service.load_or_fresh(game_id);
        game_state.narrative.input_buffer.status = GenerationStatus::Idle;
        game_state.narrative.input_buffer.step = Step::default();
        self.message_service.save_state(game_id, &game_state)?;
        Ok(())
    }

    fn heal_stale_if_changed(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
        game_state: &mut GameState,
    ) -> Result<(), EngineError> {
        let before = game_state.narrative.input_buffer.status.clone();
        generation_gate.heal_stale(game_id, game_state);
        if game_state.narrative.input_buffer.status != before {
            self.message_service.save_state(game_id, game_state)?;
        }
        Ok(())
    }

    pub fn heal_stale_status(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
    ) -> Result<(), EngineError> {
        let mut game_state = self.message_service.load_or_fresh(game_id);
        self.heal_stale_if_changed(generation_gate, game_id, &mut game_state)
    }

    pub(super) fn claim_and_spawn<E, BeforeClaim, PreSpawn, SpawnTask>(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
        before_claim: BeforeClaim,
        pre_spawn: PreSpawn,
        spawn_task: SpawnTask,
    ) -> Result<ProcessActionResult, E>
    where
        E: From<EngineError>,
        BeforeClaim: FnOnce(GameId, &mut GameState) -> Result<(), E>,
        PreSpawn: FnOnce() -> Result<(), E>,
        SpawnTask: FnOnce(&ActionPipeline) + Send + 'static,
    {
        if self.is_shutting_down() {
            return Ok(ProcessActionResult::ShuttingDown);
        }
        let mut game_state = self.message_service.load_or_fresh(game_id);

        // Heal before validating or claiming, so a panicked turn cannot strand the page.
        self.heal_stale_if_changed(generation_gate, game_id, &mut game_state)?;

        before_claim(game_id, &mut game_state)?;

        let (started_game_id, started_generation_id, claim_result) =
            generation_gate.try_claim(game_id, &mut game_state, self.message_service.as_ref())?;
        match claim_result {
            ProcessActionResult::ConcurrentGeneration => {
                return Ok(ProcessActionResult::ConcurrentGeneration);
            }
            ProcessActionResult::Started => {}
            ProcessActionResult::ShuttingDown => {
                return Ok(ProcessActionResult::ShuttingDown);
            }
        }

        // On `pre_spawn` failure no `guard` drops, so release the claimed slot
        // manually to keep the registry in sync with the returned `Err`.
        if let Err(e) = pre_spawn() {
            generation_gate.release_generation_slot(started_game_id, started_generation_id);
            return Err(e);
        }

        let gate = generation_gate.clone();
        let pipeline_arc = Arc::new(self.clone());
        let job = tokio::task::spawn_blocking(move || {
            tracing::debug!("spawn_blocking: task started");
            let _guard = gate.guard(started_game_id, started_generation_id);
            if pipeline_arc.is_shutting_down() {
                tracing::debug!("spawn_blocking: shutting down before executing task");
                return;
            }
            spawn_task(&pipeline_arc);
            tracing::debug!("spawn_blocking: task completed");
        });
        #[cfg(feature = "testing")]
        generation_gate.watch_job(job);
        #[cfg(not(feature = "testing"))]
        let _ = job;
        Ok(ProcessActionResult::Started)
    }

    #[instrument(
        skip(self, impersonated, guide_text, impersonate_direction),
        fields(input_length)
    )]
    pub fn run_from_input(
        &self,
        game_id: GameId,
        mut state: GameState,
        input: String,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) -> Result<(), PhaseError> {
        tracing::debug!("run_from_input: called");
        let run = PipelineRun::new(self, game_id);

        let (impersonated, guide_text, impersonate_direction) =
            if impersonated || guide_text.is_some() || impersonate_direction.is_some() {
                (impersonated, guide_text, impersonate_direction)
            } else {
                match state.narrative.retry_target.as_ref() {
                    Some(target) => (
                        target.impersonated(),
                        target.guide_text().map(|d| d.to_string()),
                        target.impersonate_direction().map(|d| d.to_string()),
                    ),
                    None => (false, None, None),
                }
            };
        let generation_inputs = narration_generation::GenerationInputs {
            input: input.clone(),
            guide: guide_text,
            impersonate: impersonated.then_some(narration_generation::ImpersonateInputs {
                direction: impersonate_direction,
            }),
        };

        if let Err(e) = run.phase_pre_main_snapshot(&mut state) {
            Self::finalize_phase_error(&run, Some(&mut state), e);
            return Ok(());
        }

        let outcome = match narration_generation::NarrationGeneration::new(&run, generation_inputs)
            .run(&mut state)
        {
            Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return Ok(());
            }
            Ok(outcome) => outcome,
        };
        let narration_text = outcome.narration_text;
        state.narrative.last_backend_name = Some(outcome.backend_name);
        state.narrative.last_model_name = Some(outcome.model_name);
        let (map, persona, npcs) = {
            let bundle = &outcome.bundle;
            (
                Arc::clone(&bundle.map),
                Arc::clone(&bundle.persona),
                bundle.npcs.clone(),
            )
        };

        let kind = match impersonated {
            true => Kind::Impersonate,
            false => Kind::Narration,
        };
        let quantifier_result = match run.phase_post_generation(
            &mut state,
            &input,
            &narration_text,
            &map,
            &persona,
            &npcs,
            kind,
        ) {
            Ok(r) => r,
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return Ok(());
            }
        };

        let turn_result = match Self::phase_engine_commit(
            state,
            &narration_text,
            &quantifier_result,
            &map,
            &persona,
            &npcs,
        ) {
            Ok(r) => r,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    None,
                    PhaseError::PersistFailed {
                        label: "engine commit",
                        source: e,
                    },
                );
                return Ok(());
            }
        };
        let mut post_commit_state = turn_result.post_commit_state;

        let trigger_request = turn_result
            .trigger_match
            .as_ref()
            .and_then(|trigger_match| {
                run.build_trigger_request(
                    &post_commit_state,
                    &narration_text,
                    &outcome.bundle,
                    trigger_match,
                )
            });
        if let Err(e) = run.persist_snapshot_or_err(&mut post_commit_state, "post-engine snapshot")
        {
            Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
            return Ok(());
        }
        if let Some(target) = post_commit_state.narrative.retry_target.take() {
            post_commit_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            let continuation_text = match run.phase_trigger_continuation_llm_call(
                &mut post_commit_state,
                &request,
                &map,
                &npcs,
            ) {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
                    return Ok(());
                }
                Ok(t) => t,
            };
            if !continuation_text.is_empty() {
                if let Err(e) = run.reconcile_post_trigger_npcs(
                    &mut post_commit_state,
                    &input,
                    &continuation_text,
                    &map,
                    &persona,
                    &npcs,
                ) {
                    Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
                    return Ok(());
                }
            }
        }

        // An impersonate turn is the player acting, so its offered set is not
        // rewritten.
        if !impersonated {
            let options_enabled = run.resolve_options_always_on(&outcome.bundle.world);
            self.rewrite_options_after_turn(
                game_id,
                &mut post_commit_state,
                &map,
                &persona,
                &npcs,
                options_enabled,
            );
        }

        run.phase_finalize(&mut post_commit_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    pub(crate) fn load_world_bundle(
        &self,
        started_for: GameId,
    ) -> Result<WorldBundle, EngineError> {
        self.storage.world_bundle_for(started_for.0)
    }

    pub(super) fn finalize_phase_error(
        run: &PipelineRun<'_>,
        state: Option<&mut GameState>,
        e: PhaseError,
    ) {
        let failure = match e {
            PhaseError::NarratorFailed(failure) => failure,
            PhaseError::FetchFailed(failure) => failure,
            PhaseError::PersistFailed { label, source } => {
                tracing::error!("{label}: {source}");
                PipelineRun::save_failure(label, &source)
            }
            PhaseError::TriggerMissing => GenerationFailure::new(
                GenerationFailureKind::Other,
                "Retry failed: missing trigger context",
            ),
            PhaseError::SnapshotMissing => GenerationFailure::new(
                GenerationFailureKind::Other,
                "World data unavailable for current game",
            ),
            PhaseError::Cancelled => {
                unreachable!("Cancelled must be handled before calling finalize_phase_error")
            }
        };

        match state {
            Some(state) => {
                state.narrative.input_buffer.status = GenerationStatus::Error(failure);
                state.narrative.input_buffer.step = Step::default();
                if let Err(e) = run
                    .pipeline
                    .message_service
                    .save_message_and_snapshot(run.started_for, state)
                {
                    tracing::error!("Failed to persist error state: {e}");
                }
            }
            None => run
                .pipeline
                .persist_generation_error(run.started_for, failure),
        }
    }

    pub(crate) fn phase_trigger_continuation(
        &self,
        game_id: GameId,
        state: &mut GameState,
        trigger: &StoredTriggerContext,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<String, PhaseError> {
        let run = PipelineRun::new(self, game_id);
        match run.phase_trigger_continuation_llm_call(state, trigger, map, npcs) {
            Err(PhaseError::Cancelled) => Err(run.handle_cancellation()),
            other => other,
        }
    }

    pub(crate) fn persist_generation_error(&self, game_id: GameId, failure: GenerationFailure) {
        let mut state = self.message_service.load_or_fresh(game_id);
        state.narrative.input_buffer.status = GenerationStatus::Error(failure);
        state.narrative.input_buffer.step = Step::default();
        if let Err(e) = self.message_service.save_state(game_id, &state) {
            tracing::error!("Critical: failed to persist generation error state: {e}");
        }
    }

    pub(crate) fn log_cancellation(&self, outcome: Result<(), PhaseError>) {
        if let Err(PhaseError::Cancelled) = outcome {
            tracing::debug!("Pipeline cancelled");
        }
    }

    pub(crate) fn retry_event_continuation(
        &self,
        game_id: GameId,
        state: &mut GameState,
    ) -> Result<(), PhaseError> {
        let run = PipelineRun::new(self, game_id);

        let Some(trigger) = state.narrative.last_trigger.clone() else {
            Self::finalize_phase_error(&run, Some(state), PhaseError::TriggerMissing);
            return Ok(());
        };
        let input_text = state
            .narrative
            .history
            .last_input_text()
            .unwrap_or_default();
        let WorldBundle {
            world,
            map,
            persona,
            npcs: npcs_map,
            ..
        } = match self.load_world_bundle(game_id) {
            Ok(b) => b,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    Some(state),
                    PhaseError::FetchFailed(GenerationFailure::from_engine_error(&e)),
                );
                return Ok(());
            }
        };
        let continuation_text =
            match self.phase_trigger_continuation(game_id, state, &trigger, &map, &npcs_map) {
                Ok(t) => t,
                Err(PhaseError::Cancelled) => return Err(PhaseError::Cancelled),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(state), e);
                    return Ok(());
                }
            };
        if !continuation_text.is_empty() {
            match run.reconcile_post_trigger_npcs(
                state,
                &input_text,
                &continuation_text,
                &map,
                &persona,
                &npcs_map,
            ) {
                Ok(()) => {}
                Err(PhaseError::Cancelled) => return Err(PhaseError::Cancelled),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(state), e);
                    return Ok(());
                }
            }
        }
        if let Some(target) = state.narrative.retry_target.take() {
            state.narrative.history.append(target);
        }
        // An event-only retry is never an impersonation, so it always rewrites the offered set.
        let options_enabled = run.resolve_options_always_on(&world);
        self.rewrite_options_after_turn(game_id, state, &map, &persona, &npcs_map, options_enabled);
        run.phase_finalize(state);
        Ok(())
    }

    pub(crate) fn retry_main_narration(
        &self,
        game_id: GameId,
        state: GameState,
        input_text: String,
    ) -> Result<(), PhaseError> {
        self.run_from_input(game_id, state, input_text, false, None, None)
    }

    fn phase_engine_commit(
        state: GameState,
        narration_text: &str,
        quantifier_result: &QuantifierResult,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<ActionResult, EngineError> {
        state.execute_freeaction_impl(
            &FreeActionContext {
                narration_text,
                quantifier_result,
            },
            map,
            persona,
            npcs,
        )
    }
}
