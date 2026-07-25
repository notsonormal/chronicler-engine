//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Main game state and builder

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::quantifier::{
    NpcEvent, NpcTransitionType, QuantifierResult, compute_npc_events,
};
use crate::domain::model::template::{TemplateVars, render_template};
use crate::domain::model::trigger::{NpcEncounterLog, Trigger};
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::error::{EngineError, Result};
#[cfg(feature = "diagnostics")]
use crate::error::internal_error;
use crate::domain::model::message::{Message, Swipe};
use super::message_types::MessageType;
use super::movement::MovementState;
use super::narrative_state::NarrativeState;
use super::scene_state::SceneState;

/// Mutable game state. World data lives on the orchestrator and threads through engine call sites as `&Arc<T>`/`&HashMap<_, _>` args.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GameState {
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
}

// Builder uses Option + Default so new GameState fields don't break callers.
pub struct GameStateBuilder {
    starting_room: String,
    narrative: Option<NarrativeState>,
    scene: Option<SceneState>,
    npc_encounter_log: Option<NpcEncounterLog>,
}

impl GameStateBuilder {
    pub fn new(starting_room: impl Into<String>) -> Self {
        Self {
            starting_room: starting_room.into(),
            narrative: None,
            scene: None,
            npc_encounter_log: None,
        }
    }

    pub fn with_narrative(mut self, narrative: NarrativeState) -> Self {
        self.narrative = Some(narrative);
        self
    }

    pub fn with_scene(mut self, scene: SceneState) -> Self {
        self.scene = Some(scene);
        self
    }

    pub fn with_npc_encounter_log(mut self, log: NpcEncounterLog) -> Self {
        self.npc_encounter_log = Some(log);
        self
    }

    pub fn build(self) -> GameState {
        GameState {
            movement: MovementState {
                current_room_id: self.starting_room,
                dynamic_rooms: HashMap::new(),
            },
            narrative: self.narrative.unwrap_or_default(),
            scene: self.scene.unwrap_or_default(),
            npc_encounter_log: self.npc_encounter_log.unwrap_or_default(),
        }
    }
}

impl GameState {
    pub fn from_snapshot(
        snapshot: &crate::domain::model::state::game_state_snapshot::GameStateSnapshot,
    ) -> Self {
        Self {
            movement: snapshot.movement.clone(),
            narrative: NarrativeState::from_snapshot(&snapshot.narrative),
            scene: snapshot.scene.clone(),
            npc_encounter_log: snapshot.npc_encounter_log.clone(),
        }
    }

    pub fn new(starting_room: impl Into<String>) -> Self {
        GameStateBuilder::new(starting_room).build()
    }

    pub fn init_scenario_npcs(
        &mut self,
        scenario: &crate::domain::model::scenario::StartingScenario,
        npcs: &HashMap<String, NpcCard>,
    ) {
        for npc_id in &scenario.npcs {
            if let Some(npc) = npcs.get(npc_id).cloned() {
                let encounter = self
                    .npc_encounter_log
                    .npcs
                    .entry(npc_id.clone())
                    .or_default();
                encounter.times_met = 1;
                encounter.currently_meeting = true;
                if !self.scene.npcs_in_area.iter().any(|n| n.id == *npc_id) {
                    self.scene.npcs_in_area.push(npc);
                }
            }
        }
    }

    fn push_message(&mut self, text: String, sender: Option<String>, message_type: MessageType) {
        let location_header = self.narrative.pending_location.take();
        let event_header = self.narrative.pending_event.take();

        if message_type == MessageType::Narration || message_type == MessageType::Dialogue {
            if let Some(ref mut target) = self.narrative.retry_target {
                let target_is_event = target.event_header().is_some();
                let new_is_event = event_header.is_some();
                if target_is_event == new_is_event {
                    let swipe = Swipe {
                        text: text.clone(),
                        snapshot_id: None,
                        location_header: location_header.clone(),
                        event_header: event_header.clone(),
                    };
                    target.swipes.push(swipe);
                    target.set_active_swipe(target.swipes.len() - 1);
                    return;
                }
            }
        }

        let message = Message::new(sender, text, message_type, location_header, event_header);
        self.narrative.history.append(message);
    }

    pub fn add_message(&mut self, text: String, sender: Option<String>, message_type: MessageType) {
        self.push_message(text, sender, message_type);
    }
}

pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub quantifier_result: &'a QuantifierResult,
}

pub struct TriggerMatch {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
}

pub struct ActionResult {
    pub next_state: GameState,
    pub narration: String,
    pub trigger_match: Option<TriggerMatch>,
}

impl GameState {
    pub fn attempt_movement(mut self, destination: &str, map: &MapDef) -> Result<Self> {
        match self.attempt_semantic_walk(map, destination) {
            Ok(_) => Ok(self),
            Err(e) => {
                tracing::debug!("Semantic walk failed for '{destination}': {e}");
                let dynamic_room =
                    Room::new_dynamic(destination, "A place you have never seen before.");
                self.add_message(
                    format!("[System] Entered unknown location: {}", dynamic_room.id),
                    None,
                    MessageType::System,
                );
                self.movement
                    .dynamic_rooms
                    .insert(dynamic_room.id.clone(), dynamic_room.clone());
                self.movement.current_room_id = dynamic_room.id.clone();
                Ok(self)
            }
        }
    }

    pub fn update_npc_encounters_on_room_change(
        mut self,
        previous_room_id: &str,
        new_npc_ids: &[String],
    ) -> Self {
        if previous_room_id != self.movement.current_room_id {
            for npc_id in new_npc_ids {
                self.npc_encounter_log.set_currently_meeting(npc_id, true);
            }
        }
        self
    }

    pub fn log_movement_completion(mut self, map: &MapDef) -> Self {
        let room = map
            .get_room_by_id(&self.movement.current_room_id)
            .or_else(|| {
                self.movement
                    .dynamic_rooms
                    .get(&self.movement.current_room_id)
            });
        if let Some(current_room) = room {
            self.narrative.pending_location = Some(current_room.name.clone());
        }
        self
    }

    pub fn handle_movement(
        self,
        destination: Option<&str>,
        new_npc_ids: &[String],
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<Self> {
        let Some(destination) = destination else {
            return Ok(self);
        };

        let previous_room_id = self.movement.current_room_id.clone();
        let state = self.attempt_movement(destination, map.as_ref())?;
        let state = state.update_npc_encounters_on_room_change(&previous_room_id, new_npc_ids);
        let state = state.log_movement_completion(map.as_ref());

        state.assert_state_consistency(map, npcs)?;
        Ok(state)
    }

    pub fn apply_npc_events(
        mut self,
        events: &[NpcEvent],
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<Self> {
        for event in events {
            match event.event_type {
                NpcTransitionType::Entered => {
                    self.npc_encounter_log
                        .set_currently_meeting(&event.npc_id, true);
                    self.npc_encounter_log.increment_times_met(&event.npc_id);
                }
                NpcTransitionType::Left => {
                    self.npc_encounter_log
                        .set_currently_meeting(&event.npc_id, false);
                }
            }
        }

        self.assert_state_consistency(map, npcs)?;
        Ok(self)
    }

    pub fn commit_trigger_narration(
        mut self,
        trigger: &StoredTriggerContext,
        continuation_text: &str,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<Self> {
        if continuation_text.trim().is_empty() {
            return Ok(self);
        }
        self.narrative.last_trigger = Some(trigger.clone());
        self.narrative.pending_event = Some(trigger.trigger_name.clone());
        self.add_message(continuation_text.to_string(), None, MessageType::Narration);
        if !trigger.trigger_repeat {
            self.npc_encounter_log
                .mark_trigger_fired(&trigger.npc_id, trigger.trigger_idx);
        }

        self.assert_state_consistency(map, npcs)?;
        Ok(self)
    }

    pub fn execute_freeaction_impl(
        &self,
        ctx: &FreeActionContext<'_>,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<ActionResult> {
        let previous_room_npcs: Vec<NpcCard> = self.scene.npcs_in_area.clone();
        let previous_npc_ids: Vec<String> =
            previous_room_npcs.iter().map(|n| n.id.clone()).collect();

        let mut next_state = self.clone().handle_movement(
            ctx.quantifier_result.movement.destination.as_deref(),
            &ctx.quantifier_result.npcs.npc_ids,
            map,
            npcs,
        )?;

        next_state.assert_state_consistency(map, npcs)?;

        next_state.scene.npcs_in_area = ctx
            .quantifier_result
            .npcs
            .npc_ids
            .iter()
            .filter_map(|id| npcs.get(id).cloned())
            .collect();
        let current_npc_ids = ctx.quantifier_result.npcs.npc_ids.clone();

        let trigger_match =
            next_state
                .evaluate_triggers(npcs)
                .into_iter()
                .next()
                .map(|(npc, trigger, idx)| TriggerMatch {
                    npc_id: npc.id,
                    trigger_idx: idx,
                    trigger_name: trigger.narration.name,
                    trigger_repeat: trigger.repeat,
                    trigger_narration_prompt: render_template(
                        &trigger.narration.narration_prompt,
                        &TemplateVars::new(&persona.sheet.name),
                    ),
                });

        let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
        next_state = next_state.apply_npc_events(&events.events, map, npcs)?;
        next_state.assert_state_consistency(map, npcs)?;

        Ok(ActionResult {
            next_state,
            narration: ctx.narration_text.to_string(),
            trigger_match,
        })
    }

    pub fn attempt_semantic_walk(&mut self, map: &MapDef, room_id: &str) -> Result<String> {
        let room_name = if let Some(room) = map.get_room_by_id(room_id) {
            room.name.clone()
        } else if let Some(room) = self.movement.dynamic_rooms.get(room_id) {
            room.name.clone()
        } else {
            return Err(EngineError::Navigation(
                "You don't see a way to go there.".to_string(),
            ));
        };

        self.movement.current_room_id = room_id.to_string();
        Ok(format!("You go to: {room_name}."))
    }

    pub fn evaluate_triggers(
        &self,
        npcs: &HashMap<String, NpcCard>,
    ) -> Vec<(NpcCard, Trigger, usize)> {
        let current_room_id = &self.movement.current_room_id;
        let mut results = Vec::new();

        for npc in npcs.values() {
            for (index, trigger) in npc.triggers.iter().enumerate() {
                if trigger
                    .room_id
                    .as_deref()
                    .is_some_and(|r| r != current_room_id)
                {
                    tracing::debug!(
                        npc_id = %npc.id,
                        trigger = %trigger.narration.name,
                        reason = "room_mismatch",
                        "Trigger skipped"
                    );
                    continue;
                }

                if !self
                    .npc_encounter_log
                    .check_condition(&npc.id, &trigger.requirement)
                {
                    tracing::debug!(
                        npc_id = %npc.id,
                        trigger = %trigger.narration.name,
                        reason = "condition_not_met",
                        "Trigger skipped"
                    );
                    continue;
                }

                if !trigger.repeat && self.npc_encounter_log.is_trigger_fired(&npc.id, index) {
                    tracing::debug!(
                        npc_id = %npc.id,
                        trigger = %trigger.narration.name,
                        reason = "already_fired",
                        "Trigger skipped"
                    );
                    continue;
                }

                results.push((npc.clone(), trigger.clone(), index));
            }
        }

        results
    }

    #[cfg(feature = "diagnostics")]
    pub fn assert_state_consistency(
        &self,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<()> {
        self.assert_room_exists(map)?;
        self.assert_npc_consistency(npcs)?;
        self.assert_npc_encounter_log_consistency(npcs)?;
        self.assert_log_invariants()?;
        Ok(())
    }

    #[cfg(not(feature = "diagnostics"))]
    pub fn assert_state_consistency(
        &self,
        _map: &Arc<MapDef>,
        _npcs: &HashMap<String, NpcCard>,
    ) -> Result<()> {
        Ok(())
    }

    #[cfg(feature = "diagnostics")]
    fn assert_room_exists(&self, map: &Arc<MapDef>) -> Result<()> {
        let current_room = map
            .get_room_by_id(&self.movement.current_room_id)
            .or_else(|| {
                self.movement
                    .dynamic_rooms
                    .get(&self.movement.current_room_id)
            });
        if current_room.is_none() {
            return Err(EngineError::Internal(internal_error(format!(
                "current_room_id '{}' not found in map or dynamic_rooms",
                self.movement.current_room_id
            ))));
        }
        Ok(())
    }

    #[cfg(feature = "diagnostics")]
    fn assert_npc_consistency(&self, npcs: &HashMap<String, NpcCard>) -> Result<()> {
        for npc in &self.scene.npcs_in_area {
            if !npcs.contains_key(&npc.id) {
                return Err(EngineError::Internal(internal_error(format!(
                    "npcs_in_area contains NPC '{}' which is not in the npcs map",
                    npc.id
                ))));
            }
        }
        Ok(())
    }

    #[cfg(feature = "diagnostics")]
    fn assert_npc_encounter_log_consistency(&self, npcs: &HashMap<String, NpcCard>) -> Result<()> {
        for npc_id in self.npc_encounter_log.npcs.keys() {
            if !npcs.contains_key(npc_id) {
                return Err(EngineError::Internal(internal_error(format!(
                    "npc_encounter_log references unknown NPC '{npc_id}'"
                ))));
            }
        }
        Ok(())
    }

    #[cfg(feature = "diagnostics")]
    fn assert_log_invariants(&self) -> Result<()> {
        let ai_idx = self.narrative.history.last_ai_response_index();
        let input_idx = self.narrative.history.last_input_index();

        if let (Some(ai), Some(input)) = (ai_idx, input_idx) {
            if ai <= input {
                return Err(EngineError::Internal(internal_error(
                    "last AI response is not after last player input",
                )));
            }
        }
        Ok(())
    }
}
