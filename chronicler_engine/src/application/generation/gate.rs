//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! GenerationGate — per-game slot orchestration. Generation truth is persisted `GenerationStatus` only.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::application::errors::{ProcessActionResult};
use crate::application::generation::guard::GenerationGuard;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::generation::slot::GenerationSlot;
use crate::application::generation::slot::release_owned_slot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::game_state::GameState;
use crate::error::EngineError;

#[derive(Clone)]
pub struct GenerationGate {
    registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    next_generation_id: Arc<AtomicU64>,
}

impl Default for GenerationGate {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationGate {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            next_generation_id: Arc::new(AtomicU64::new(0)),
        }
    }

    fn next_generation_id(&self) -> u64 {
        self.next_generation_id
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    /// Reset a stale persisted `Generating` status when no slot owns this game.
    /// Mutates `state`; caller must persist if it proceeds.
    pub fn heal_stale(&self, game_id: u64, state: &mut GameState) {
        if !state.narrative.input_buffer.status.is_generating() {
            return;
        }

        let slot_generating = {
            let registry = self.registry.read().unwrap_or_else(|p| {
                tracing::warn!("Generation registry read lock poisoned during heal; recovering");
                p.into_inner()
            });
            registry
                .get(&game_id)
                .map(|slot| slot.is_generating())
                .unwrap_or(false)
        };

        if slot_generating {
            return;
        }

        tracing::warn!(
            "Found stale Generating status without active generation, resetting to Idle"
        );
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
    }

    pub fn try_claim(
        &self,
        game_id: u64,
        state: &mut GameState,
        persistence: &PersistenceGate,
    ) -> Result<(u64, u64, ProcessActionResult), EngineError> {
        let generation_id = self.next_generation_id();

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
        }

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        if let Err(e) = persistence.save_message_and_snapshot(state) {
            tracing::debug!("try_claim: save failed; releasing slot");
            release_owned_slot(&self.registry, game_id, generation_id);
            return Err(e);
        }
        tracing::debug!("try_claim: state saved, spawning blocking task");
        Ok((game_id, generation_id, ProcessActionResult::Started))
    }

    pub(crate) fn guard(&self, game_id: u64, generation_id: u64) -> GenerationGuard {
        GenerationGuard::new(game_id, generation_id, Arc::clone(&self.registry))
    }

    pub fn release_generation_slot(&self, game_id: u64, generation_id: u64) {
        release_owned_slot(&self.registry, game_id, generation_id);
    }

    /// Release any active generation slot for the given game and return its
    /// generation id. This is the caller-driven reset path; the persisted status
    /// is reset separately by the pipeline so the gate does not own persistence.
    pub fn release_generation_slot_for_game(&self, game_id: u64) -> Option<u64> {
        let mut registry = self.registry.write().unwrap_or_else(|p| {
            tracing::warn!(
                "Generation registry write lock poisoned during release for game; recovering"
            );
            p.into_inner()
        });
        match registry.get(&game_id) {
            Some(GenerationSlot::Generating { generation_id }) => {
                let id = *generation_id;
                registry.insert(game_id, GenerationSlot::Idle);
                Some(id)
            }
            _ => None,
        }
    }

    pub fn is_busy(&self, game_id: u64) -> bool {
        let registry = self.registry.read().unwrap_or_else(|p| {
            tracing::warn!("Generation registry read lock poisoned during is_busy; recovering");
            p.into_inner()
        });
        registry
            .get(&game_id)
            .map(|slot| slot.is_generating())
            .unwrap_or(false)
    }
}
