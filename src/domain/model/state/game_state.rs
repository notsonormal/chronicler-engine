//! [DOC: docs/diataxis/reference/game_flow.md]
//! Main game state and builder

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::quantifier::{NpcEvent, QuantifierResult};
use crate::domain::model::state::movement::MovementState;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::{StoredTriggerContext, TriggerRef};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::trigger::NpcEncounterLog;
use crate::domain::model::utils::game_rules::{
    apply_game_rules, assert_state_consistency, GameRuleRequest, GameRulesOutcome, TriggerMatch,
};
use crate::domain::model::world::WorldCard;
use crate::error::Result;
use super::message_types::MessageType;
use super::narrative_state::{ArrivingLocationWrite, NarrativeState};
use super::scene_state::SceneState;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GameState {
    pub narrative: NarrativeState,
    pub swipe: SwipeState,
}

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
            narrative: self.narrative.unwrap_or_default(),
            swipe: SwipeState {
                movement: MovementState {
                    current_room_id: self.starting_room,
                    dynamic_rooms: HashMap::new(),
                },
                scene: self.scene.unwrap_or_default(),
                npc_encounter_log: self.npc_encounter_log.unwrap_or_default(),
                trigger_context: None,
            },
        }
    }
}

impl GameState {
    pub fn from_snapshot(
        snapshot: &crate::domain::model::state::game_state_snapshot::GameStateSnapshot,
    ) -> Self {
        Self {
            narrative: NarrativeState::from_snapshot(&snapshot.narrative),
            swipe: SwipeState {
                movement: snapshot.movement.clone(),
                scene: snapshot.scene.clone(),
                npc_encounter_log: snapshot.npc_encounter_log.clone(),
                trigger_context: None,
            },
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
        self.swipe.init_scenario_npcs(scenario, npcs);
    }

    fn push_message(
        &mut self,
        text: String,
        message_type: MessageType,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) {
        let room_report = self.narrative.pending_location.take();
        let event_header = self.narrative.pending_event.take();

        if message_type == MessageType::Narration || message_type == MessageType::Input {
            if let Some(ref mut target) = self.narrative.retry_target {
                let target_is_event = target.event_header().is_some();
                let new_is_event = event_header.is_some();
                if target_is_event == new_is_event {
                    let swipe = Swipe {
                        text: text.clone(),
                        snapshot_id: None,
                        room_report: room_report.clone(),
                        event_header: event_header.clone(),
                        impersonated: target.impersonated(),
                        guide_text: target.guide_text().map(|d| d.to_string()),
                        impersonate_direction: target
                            .impersonate_direction()
                            .map(|d| d.to_string()),
                        state: None,
                        maker: None,
                    };
                    target.swipes.push(swipe);
                    target.set_active_swipe(target.swipes.len() - 1);
                    return;
                }
            }
        }

        let mut message = Message::new(text, message_type, room_report, event_header);
        if impersonated || guide_text.is_some() || impersonate_direction.is_some() {
            message.set_stored_inputs(impersonated, guide_text, impersonate_direction);
        }
        self.narrative.history.append(message);
    }

    pub fn add_message(&mut self, text: String, message_type: MessageType) {
        self.push_message(text, message_type, false, None, None);
    }

    /// The Swipe stores the generation inputs so a redo re-applies them.
    pub fn add_message_with_inputs(
        &mut self,
        text: String,
        message_type: MessageType,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) {
        self.push_message(
            text,
            message_type,
            impersonated,
            guide_text,
            impersonate_direction,
        );
    }

    pub fn inject_scenario_logs(
        &mut self,
        world: &WorldCard,
        player: &PersonaCard,
        map: &crate::domain::model::map::MapDef,
    ) {
        let Some(scenario) = world.default_scenario() else {
            return;
        };
        if scenario.text.is_empty() {
            return;
        }

        let room_name = map
            .get_room_by_id(&scenario.starting_room_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| scenario.starting_room_id.clone());

        self.narrative.pending_location = Some(room_name);
        let text = TemplateVars::new(&player.sheet.name).render(&scenario.text);
        self.add_message(text, MessageType::Narration);
    }
}

pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub quantifier_result: &'a QuantifierResult,
}

pub struct ActionResult {
    pub post_commit_state: GameState,
    pub narration: String,
    pub trigger_match: Option<TriggerMatch>,
}

impl GameState {
    pub fn apply_npc_events(
        &mut self,
        events: &[NpcEvent],
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<()> {
        apply_game_rules(
            &mut self.swipe,
            GameRuleRequest::NpcEvents(events),
            map,
            npcs,
        )?;
        Ok(())
    }

    pub fn commit_trigger_narration(
        &mut self,
        trigger: &StoredTriggerContext,
        continuation_text: &str,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<()> {
        let outcome = apply_game_rules(
            &mut self.swipe,
            GameRuleRequest::TriggerNarration {
                trigger: &TriggerRef::from(trigger),
                text: continuation_text,
            },
            map,
            npcs,
        )?;
        if !outcome.trigger_narration_landed {
            return Ok(());
        }

        self.narrative.last_trigger = Some(trigger.clone());
        self.narrative.pending_event = Some(trigger.trigger_name.clone());
        self.add_message(continuation_text.to_string(), MessageType::Narration);

        // The check runs after the event Narration lands, because the caller reports its
        // failure with this state.
        assert_state_consistency(&self.swipe, map, npcs)
    }

    pub fn execute_freeaction_impl(
        mut self,
        ctx: &FreeActionContext<'_>,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<ActionResult> {
        let outcome = apply_game_rules(
            &mut self.swipe,
            GameRuleRequest::Analysis {
                quantifier_result: ctx.quantifier_result,
                persona_name: &persona.sheet.name,
            },
            map,
            npcs,
        )?;
        self.write_rules_story(&outcome);

        Ok(ActionResult {
            post_commit_state: self,
            narration: ctx.narration_text.to_string(),
            trigger_match: outcome.trigger_match,
        })
    }

    fn write_rules_story(&mut self, outcome: &GameRulesOutcome) {
        if let Some(room) = outcome.unknown_location.as_ref() {
            self.add_message(
                format!("[System] Entered unknown location: {}", room.id),
                MessageType::System,
            );
        }
        self.write_room_report(outcome.arrived_room_name.clone());
    }

    /// The Quantifier reports a movement only after the Narration was saved, so the room it
    /// names is written back onto that Narration.
    fn write_room_report(&mut self, room_name: Option<String>) {
        let Some(room_name) = room_name else {
            return;
        };
        self.narrative.pending_location = None;
        if let Some(arriving_message) = self.arriving_message_mut() {
            arriving_message.set_room_report(room_name.clone());
            if !arriving_message.is_unpersisted() {
                self.narrative.arriving_location_write = Some(ArrivingLocationWrite {
                    message_id: arriving_message.id,
                    swipe_index: arriving_message.active_swipe_index,
                    room_name,
                });
            }
        }
    }

    /// A redo appends its Swipe to the Message it retries, so that Message
    /// carries the redo's Narration.
    fn arriving_message_mut(&mut self) -> Option<&mut Message> {
        if let Some(target) = self.narrative.retry_target.as_mut() {
            return Some(target);
        }
        self.narrative
            .history
            .iter_mut()
            .rev()
            .find(|message| message.is_input_or_narration())
    }
}
