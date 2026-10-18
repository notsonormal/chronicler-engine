//! [DOC: docs/diataxis/reference/game_flow.md]
//! GenerationGate — per-game slot orchestration. The registry is the live "is generating" truth; the persisted `GenerationStatus` carries the durable phase.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::application::errors::{ProcessActionResult};
use crate::application::generation::guard::GenerationGuard;
use crate::application::message_service::MessageService;
use crate::application::generation::slot::GenerationSlot;
use crate::application::generation::slot::release_owned_slot;
use crate::domain::model::game::GameId;
use crate::domain::model::state::generation_status::{GenerationStatus, Step};
use crate::domain::model::state::game_state::GameState;
use crate::error::EngineError;

/// A stop and a reset end the *slot* of a Game at once, while the Job's thread
/// lives on until its refusals return. A test that asserts what a Job's late
/// answer wrote therefore waits here, and not for the slot.
#[cfg(feature = "testing")]
#[derive(Default)]
pub struct JobTasks {
    handles: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}

#[cfg(feature = "testing")]
impl JobTasks {
    pub fn watch(&self, handle: tokio::task::JoinHandle<()>) {
        let mut handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        handles.retain(|handle| !handle.is_finished());
        handles.push(handle);
    }

    pub async fn wait(&self, timeout: std::time::Duration) -> bool {
        tokio::time::timeout(timeout, self.wait_for_each())
            .await
            .is_ok()
    }

    async fn wait_for_each(&self) {
        loop {
            let pending: Vec<tokio::task::JoinHandle<()>> = {
                let mut handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
                handles.drain(..).collect()
            };
            if pending.is_empty() {
                return;
            }
            for handle in pending {
                // A Job that panicked failed its own test already (the fake
                // drops with its stray calls on record); this only orders the
                // test after the task.
                if let Err(join_error) = handle.await {
                    tracing::debug!("a Job task ended without a result: {join_error}");
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct GenerationGate {
    registry: Arc<RwLock<HashMap<GameId, GenerationSlot>>>,
    next_generation_id: Arc<AtomicU64>,
    #[cfg(feature = "testing")]
    jobs: Arc<JobTasks>,
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
            #[cfg(feature = "testing")]
            jobs: Arc::new(JobTasks::default()),
        }
    }

    #[cfg(feature = "testing")]
    pub fn watch_job(&self, handle: tokio::task::JoinHandle<()>) {
        self.jobs.watch(handle);
    }

    #[cfg(feature = "testing")]
    pub async fn wait_for_jobs(&self, timeout_ms: u64) -> bool {
        self.jobs
            .wait(std::time::Duration::from_millis(timeout_ms))
            .await
    }

    fn next_generation_id(&self) -> u64 {
        self.next_generation_id
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    /// Mutates `state`; the caller persists the reset.
    pub fn heal_stale(&self, game_id: GameId, state: &mut GameState) {
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
        state.narrative.input_buffer.step = Step::default();
    }

    pub fn try_claim(
        &self,
        game_id: GameId,
        state: &mut GameState,
        message_service: &MessageService,
    ) -> Result<(GameId, u64, ProcessActionResult), EngineError> {
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
        state.narrative.input_buffer.step = Step::Narrating;

        if let Err(e) = message_service.save_message_and_snapshot(game_id, state) {
            tracing::debug!("try_claim: save failed; releasing slot");
            release_owned_slot(&self.registry, game_id, generation_id);
            return Err(e);
        }
        tracing::debug!("try_claim: state saved, spawning blocking task");
        Ok((game_id, generation_id, ProcessActionResult::Started))
    }

    pub(crate) fn guard(&self, game_id: GameId, generation_id: u64) -> GenerationGuard {
        GenerationGuard::new(game_id, generation_id, Arc::clone(&self.registry))
    }

    pub fn release_generation_slot(&self, game_id: GameId, generation_id: u64) {
        release_owned_slot(&self.registry, game_id, generation_id);
    }

    /// Releases the slot; the pipeline resets the persisted status.
    pub fn release_generation_slot_for_game(&self, game_id: GameId) -> Option<u64> {
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

    pub fn is_busy(&self, game_id: GameId) -> bool {
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
