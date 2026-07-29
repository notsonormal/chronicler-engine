//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! GenerationGate — `is_generating` cache (ADR-030) + per-game slot orchestration.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::application::application_service::ProcessActionResult;
use crate::application::errors::ApplicationError;
use crate::application::generation_guard::GenerationGuard;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::utils::slot::GenerationSlot;
use crate::application::utils::slot::release_owned_slot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::game_state::GameState;
use crate::error::EngineError;

#[derive(Clone)]
pub struct GenerationGate {
    is_generating: Arc<AtomicBool>,
    registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    next_generation_id: Arc<AtomicU64>,
}

impl GenerationGate {
    pub fn new(is_generating: Arc<AtomicBool>) -> Self {
        Self {
            is_generating,
            registry: Arc::new(RwLock::new(HashMap::new())),
            next_generation_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn is_generating(&self) -> &Arc<AtomicBool> {
        &self.is_generating
    }

    fn next_generation_id(&self) -> u64 {
        self.next_generation_id
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    pub fn heal_stale(&self, game_id: u64, state: &mut GameState) {
        // Atomic is a hint; lock-held slot state is authoritative.
        if self.is_generating.load(Ordering::SeqCst) {
            return;
        }

        // Persisted `Generating` with no active slot. Runs outside the write lock — hold time matters.
        if state.narrative.input_buffer.status.is_generating() {
            tracing::warn!(
                "Found stale Generating status without active generation, resetting to Idle"
            );
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
        }

        // Registry slot may still claim Generating. Re-check under write lock — atomic is only a hint.
        let mut registry = self.registry.write().unwrap_or_else(|p| {
            tracing::warn!("Generation registry write lock poisoned during heal; recovering");
            p.into_inner()
        });
        if self.is_generating.load(Ordering::SeqCst) {
            // Real claim raced in, not stale — do not clear.
            return;
        }
        if let Some(slot) = registry.get(&game_id) {
            if slot.is_generating() {
                tracing::debug!("heal_stale: clearing stale registry slot for game_id={game_id}");
                registry.insert(game_id, GenerationSlot::Idle);
            }
        }
    }

    pub fn try_claim(
        &self,
        game_id: u64,
        state: &mut GameState,
        persistence: &PersistenceGate,
    ) -> Result<(u64, u64, ProcessActionResult), EngineError> {
        let generation_id = self.next_generation_id();

        // Slot insert + projection flip share the write lock to prevent clobber.
        {
            let mut registry = self.registry.write().unwrap_or_else(|p| {
                tracing::warn!("Generation registry write lock poisoned during claim; recovering");
                p.into_inner()
            });
            if let Some(slot) = registry.get(&game_id) {
                if slot.is_generating() {
                    return Ok((
                        game_id,
                        generation_id,
                        ProcessActionResult::ConcurrentGeneration,
                    ));
                }
            }
            registry.insert(game_id, GenerationSlot::Generating { generation_id });

            // Atomic is a derived projection of "any game generating" — unconditional store, no CAS.
            self.is_generating.store(true, Ordering::SeqCst);
        }

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        if let Err(e) = persistence.save_message_and_snapshot(state) {
            tracing::debug!("try_claim: save failed; releasing slot");
            // Release the claimed id (not current_game_id()) — a reset between claim and save may have changed it.
            release_owned_slot(&self.registry, &self.is_generating, game_id, generation_id);
            return Err(e);
        }
        tracing::debug!("try_claim: state saved, spawning blocking task");
        Ok((game_id, generation_id, ProcessActionResult::Started))
    }

    pub(crate) fn guard(&self, game_id: u64, generation_id: u64) -> GenerationGuard {
        GenerationGuard::new(
            game_id,
            generation_id,
            Arc::clone(&self.registry),
            Arc::clone(&self.is_generating),
        )
    }

    pub fn release_generation_slot(&self, game_id: u64, generation_id: u64) {
        // Use claimed id — concurrent reset cannot steer release at the wrong slot.
        release_owned_slot(&self.registry, &self.is_generating, game_id, generation_id);
    }

    pub fn reset_generating_status(
        &self,
        persistence_gate: &PersistenceGate,
    ) -> Result<(), ApplicationError> {
        let mut game_state = persistence_gate.load_or_fresh();
        game_state.narrative.input_buffer.status = GenerationStatus::Idle;
        persistence_gate.save_state(&game_state)?;
        Ok(())
    }
}
