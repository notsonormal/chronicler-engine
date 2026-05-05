//! [DOC: docs/architecture/system.md]

use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room, get_current_room};
use crate::engine::trigger_eval::{
    evaluate_triggers, increment_times_met, mark_trigger_fired, set_currently_meeting,
};
use crate::error::EngineError;
use crate::model::character::NpcCard;

use crate::model::state::{GameState, LogType};
use crate::model::world::WorldCard;
use crate::narrative::prompt::{PromptBuilder, PromptContext};
use crate::narrative::quantifier::{NpcEvent, QuantifierResult, compute_npc_events};

/// [DOC: docs/architecture/system.md]
pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub user_input: &'a str,
    pub quantifier_result: &'a QuantifierResult,
    pub world: &'a WorldCard,
    pub player: &'a crate::model::character::PlayerCard,
    pub all_npcs: &'a [NpcCard],
    pub history: &'a [crate::model::state::LogEntry],
    pub llm_backend: &'a dyn crate::narrative::llm::LlmBackend,
}

/// [DOC: docs/architecture/system.md]
pub fn get_static_npcs(state: &GameState, room_npc_ids: &[String]) -> Vec<NpcCard> {
    room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect()
}

/// [DOC: docs/architecture/system.md]
pub fn handle_movement(state: &mut GameState, destination: Option<&str>, new_npc_ids: &[String]) {
    let Some(trigger) = destination else {
        return;
    };

    let previous_room_id = state.current_room_id.clone();

    let success = match attempt_semantic_walk(state, trigger) {
        Ok(_) => true,
        Err(_) => {
            let dynamic_room = create_dynamic_room(trigger, "A place you have never seen before.");
            state
                .dynamic_rooms
                .insert(dynamic_room.id.clone(), dynamic_room.clone());
            state.current_room_id = dynamic_room.id.clone();
            true
        }
    };

    if !success {
        return;
    }

    if previous_room_id != state.current_room_id {
        for npc_id in new_npc_ids {
            set_currently_meeting(&mut state.character_state, npc_id, true);
        }
    }

    if let Ok(current_room) = get_current_room(state) {
        state.add_log(
            String::new(),
            Some(current_room.name.clone()),
            LogType::Narration,
        );
    }
}

/// [DOC: docs/architecture/system.md]
pub fn apply_npc_events(state: &mut GameState, events: &[NpcEvent]) {
    for event in events {
        match event.event_type {
            crate::narrative::quantifier::NpcEventType::Entered => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, true);
                increment_times_met(&mut state.character_state, &event.npc_id);
            }
            crate::narrative::quantifier::NpcEventType::Left => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, false);
            }
        }
    }
}

/// [DOC: docs/system/triggers.md]
pub fn evaluate_and_narrate_triggers(
    state: &mut GameState,
    narration_text: &str,
    trigger_context: &PromptContext<'_>,
    llm_backend: &dyn crate::narrative::llm::LlmBackend,
) {
    let matching_triggers = evaluate_triggers(state);

    let Some((trigger_idx, (npc, trigger))) = matching_triggers.iter().enumerate().next() else {
        return;
    };

    state.generation_state.phase = crate::model::state::GenerationPhase::GeneratingEvent;

    let continuation_user_msg = format!(
        "Previous narration:\n{narration_text}\n\nTrigger event: {}\n\n\
         Continue the scene naturally, incorporating the trigger event into the narrative. \
         Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
        trigger.action.narration_prompt
    );

    let trigger_ctx = PromptContext {
        world: trigger_context.world,
        room: trigger_context.room,
        all_npcs: trigger_context.all_npcs,
        npcs_in_area: &state.npcs_in_area,
        player: trigger_context.player,
        user_message: &continuation_user_msg,
        history: &state.narration_history,
    };

    let settings = crate::settings::load_settings().unwrap_or_default();
    let narration_conn = settings.get_narration_connection();
    let max_context = narration_conn
        .map(|c| c.resolve_max_context_tokens())
        .unwrap_or(crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS);
    let max_tokens = narration_conn.and_then(|c| c.max_tokens);

    let mut pb = PromptBuilder::from_context(&trigger_ctx);
    pb.max_context_tokens = Some(max_context);
    pb.requested_max_tokens = max_tokens;
    pb.response_length = Some(&settings.response_length);

    let (system_prompt, user_prompt, fitted_max_tokens) = match pb.build_split() {
        Ok(parts) => parts,
        Err(e) => {
            log::error!(
                "Failed to build trigger continuation prompt for '{}': {e}",
                trigger.action.name
            );
            return;
        }
    };

    let continuation_text = match llm_backend.narrate_action_from_prompt(
        &system_prompt,
        &user_prompt,
        Some(fitted_max_tokens),
    ) {
        Ok(text) => text,
        Err(e) => {
            log::error!("Trigger narration failed: {e}");
            state.add_log(
                format!("[Trigger narration failed: {e}]"),
                None,
                LogType::System,
            );
            return;
        }
    };

    if continuation_text.trim().is_empty() {
        return;
    }
    state.add_log(
        String::new(),
        Some(trigger.action.name.clone()),
        LogType::Event,
    );
    state.add_log(continuation_text, None, LogType::Narration);
    if !trigger.repeat {
        mark_trigger_fired(&mut state.character_state, &npc.id, trigger_idx);
    }
}

/// [DOC: docs/architecture/system.md]
pub fn execute_freeaction_impl(
    state: &mut GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<(), EngineError> {
    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    handle_movement(
        state,
        ctx.quantifier_result.movement.destination.as_deref(),
        &ctx.quantifier_result.npcs.npc_ids,
    );

    let current_npcs: Vec<NpcCard> = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids: Vec<String> = current_npcs.iter().map(|n| n.id.clone()).collect();

    let room_data = get_current_room(state)
        .map_err(|_| EngineError::RoomNotFound("current room not found".to_string()))?
        .clone();

    // Now state is no longer borrowed — we can safely mutate it
    // [DOC: docs/system/triggers.md §Mutation Order Invariant]
    // Order is load-bearing: narration logged first (step 1), then triggers evaluated
    // which read history for context (step 2), then NPC events applied (step 3).
    state.add_log(ctx.narration_text.to_string(), None, LogType::Narration);
    state.npcs_in_area = current_npcs.clone();

    let trigger_context = PromptContext {
        world: ctx.world,
        room: &room_data,
        all_npcs: ctx.all_npcs,
        npcs_in_area: &current_npcs,
        player: ctx.player,
        user_message: ctx.user_input,
        history: ctx.history,
    };

    evaluate_and_narrate_triggers(state, ctx.narration_text, &trigger_context, ctx.llm_backend);

    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
    apply_npc_events(state, &events.events);

    Ok(())
}

#[cfg(test)]
mod execute_freeaction_impl_tests {
    use super::*;
    use crate::model::state::LogType;
    use crate::narrative::quantifier::{
        MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    };
    use crate::test_support::{TestGameState, TestNpc, TestPlayer, TestWorld};
    use std::sync::Arc;

    fn make_quantifier_result_no_movement() -> QuantifierResult {
        QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["carla".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult {
                movement_type: None,
                destination: None,
                confidence: QuantifierConfidence::Low,
            },
        }
    }

    fn make_quantifier_result_with_movement(destination: &str) -> QuantifierResult {
        QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["carla".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some(destination.to_string()),
                confidence: QuantifierConfidence::High,
            },
        }
    }

    #[test]
    fn test_execute_freeaction_impl_no_movement() {
        let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());
        let all_npcs = vec![TestNpc::named("carla", "Carla")];
        let history = vec![];

        let result = execute_freeaction_impl(
            &mut state,
            &FreeActionContext {
                narration_text: "You examine the room.",
                user_input: "examine the room",
                quantifier_result: &make_quantifier_result_no_movement(),
                world: &world,
                player: &player,
                all_npcs: &all_npcs,
                history: &history,
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        assert!(result.is_ok());
        // Narration should be logged
        assert_eq!(state.narration_history.len(), 1);
        assert_eq!(state.narration_history[0].log_type, LogType::Narration);
        // NPCs in area should be updated
        assert_eq!(state.npcs_in_area.len(), 1);
        assert_eq!(state.npcs_in_area[0].id, "carla");
    }

    #[test]
    fn test_execute_freeaction_impl_with_movement() {
        let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());
        let all_npcs = vec![TestNpc::named("carla", "Carla")];
        let history = vec![];

        // quantifier result with movement to a new room
        let result = execute_freeaction_impl(
            &mut state,
            &FreeActionContext {
                narration_text: "You walk to the tavern.",
                user_input: "walk to the tavern",
                quantifier_result: &make_quantifier_result_with_movement("nonexistent_room"),
                world: &world,
                player: &player,
                all_npcs: &all_npcs,
                history: &history,
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        assert!(result.is_ok());
        // Narration logged
        assert!(!state.narration_history.is_empty());
        // Room changed to a dynamic room (since destination doesn't exist)
        assert!(state.current_room_id.starts_with("dynamic_"));
        assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
    }

    #[test]
    fn test_execute_freeaction_impl_updates_npcs_in_area() {
        let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());
        let all_npcs = vec![TestNpc::named("carla", "Carla")];

        // Start with empty npcs_in_area
        assert!(state.npcs_in_area.is_empty());

        let result = execute_freeaction_impl(
            &mut state,
            &FreeActionContext {
                narration_text: "You look around.",
                user_input: "look around",
                quantifier_result: &make_quantifier_result_no_movement(),
                world: &world,
                player: &player,
                all_npcs: &all_npcs,
                history: &[],
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        assert!(result.is_ok());
        // npcs_in_area should now contain carla
        assert_eq!(state.npcs_in_area.len(), 1);
        assert_eq!(state.npcs_in_area[0].id, "carla");
    }

    #[test]
    fn test_execute_freeaction_impl_triggers_evaluated() {
        let npc = TestNpc::with_times_met_trigger(
            "carla",
            "Carla",
            crate::model::trigger::ComparisonOperator::Eq,
            0,
        );

        let mut state = TestGameState::with_npc_raw("room1", npc.clone());
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());

        // NPC has TimesMet Eq 0 trigger - should fire because times_met starts at 0
        // Note: evaluate_and_narrate_triggers calls LLM internally, so this test
        // will use whatever backend is configured (mock in test env)
        let result = execute_freeaction_impl(
            &mut state,
            &FreeActionContext {
                narration_text: "You enter the room.",
                user_input: "enter",
                quantifier_result: &make_quantifier_result_no_movement(),
                world: &world,
                player: &player,
                all_npcs: &[], // empty won't match triggers
                history: &[],
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        // Result should be ok (even if trigger LLM call fails, fn handles gracefully)
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_freeaction_impl_npc_events_entered() {
        let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        // NPC already in area (simulating re-encounter after leaving)
        state.npcs_in_area = vec![]; // Empty - NPC is not currently in area
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());
        let all_npcs = vec![TestNpc::named("carla", "Carla")];

        let result = execute_freeaction_impl(
            &mut state,
            &FreeActionContext {
                narration_text: "You see Carla.",
                user_input: "look around",
                quantifier_result: &make_quantifier_result_no_movement(),
                world: &world,
                player: &player,
                all_npcs: &all_npcs,
                history: &[],
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        assert!(result.is_ok());
        // NPC enters - times_met should increment
        let times_met = state
            .character_state
            .npcs
            .get("carla")
            .map(|s| s.times_met)
            .unwrap_or(0);
        assert_eq!(times_met, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::trigger_eval::{get_times_met, is_currently_meeting};
    use crate::test_support::{TestGameState, TestMap, TestNpc};

    fn make_test_state() -> GameState {
        TestGameState::with_npc_in_named_room_raw(
            "test_room",
            "Test Room",
            TestNpc::named("carla", "Carla"),
        )
    }

    #[test]
    fn test_get_static_npcs_returns_npcs() {
        let state = make_test_state();
        let room_npcs = vec!["carla".to_string()];
        let result = get_static_npcs(&state, &room_npcs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "carla");
    }

    #[test]
    fn test_get_static_npcs_empty_for_unknown() {
        let state = make_test_state();
        let room_npcs = vec!["unknown".to_string()];
        let result = get_static_npcs(&state, &room_npcs);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_npc_events_entered() {
        let mut state = make_test_state();
        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Entered,
        }];

        apply_npc_events(&mut state, &events);

        assert!(is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_apply_npc_events_left() {
        let mut state = make_test_state();
        set_currently_meeting(&mut state.character_state, "carla", true);

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Left,
        }];

        apply_npc_events(&mut state, &events);

        assert!(!is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_apply_npc_events_increments_times_met() {
        let mut state = make_test_state();
        let initial_times = get_times_met(&state.character_state, "carla");

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Entered,
        }];

        apply_npc_events(&mut state, &events);

        assert_eq!(
            get_times_met(&state.character_state, "carla"),
            initial_times + 1
        );
    }

    #[test]
    fn test_handle_movement_no_destination() {
        let mut state = make_test_state();
        let original_room = state.current_room_id.clone();

        handle_movement(&mut state, None, &["carla".to_string()]);

        // Room should not change when destination is None
        assert_eq!(state.current_room_id, original_room);
    }

    #[test]
    fn test_handle_movement_same_room_no_increment() {
        let mut state = make_test_state();
        // Already in test_room, moving to same room
        state.current_room_id = "test_room".to_string();
        let initial_times = get_times_met(&state.character_state, "carla");

        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        // times_met should not increment when room doesn't change
        assert_eq!(
            get_times_met(&state.character_state, "carla"),
            initial_times
        );
    }

    #[test]
    fn test_handle_movement_creates_dynamic_room() {
        let mut state = make_test_state();
        let original_room = state.current_room_id.clone();

        // Attempt to move to a non-existent room
        handle_movement(&mut state, Some("nonexistent_room"), &[]);

        // Should create a dynamic room
        assert_ne!(state.current_room_id, original_room);
        assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
    }

    #[test]
    fn test_handle_movement_success_adds_room_log() {
        let mut state = make_test_state();

        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        assert!(!state.narration_history.is_empty());
        let last_entry = state.narration_history.last().unwrap();
        assert_eq!(last_entry.log_type, LogType::Narration);
        assert_eq!(last_entry.sender, Some("Test Room".to_string()));
    }

    #[test]
    fn test_handle_movement_sets_currently_meeting() {
        let mut state = make_test_state();

        handle_movement(&mut state, Some("new_room"), &["carla".to_string()]);

        // Should set currently_meeting for NPCs in new room
        assert!(is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_evaluate_and_narrate_triggers_adds_event_header() {
        let llm_backend = crate::narrative::llm::MockBackend::default();

        let mut state = make_test_state();
        let npc_with_trigger = TestNpc::with_times_met_trigger(
            "carla",
            "Carla",
            crate::model::trigger::ComparisonOperator::Eq,
            0,
        );
        state
            .npcs
            .insert("carla".to_string(), npc_with_trigger.clone());

        let mut room = TestMap::room_named("test_room", "Test Room");
        room.npcs.push("carla".to_string());

        let world = state.world.clone();
        let player = state.player.clone();
        let history = state.narration_history.clone();

        let trigger_context = crate::narrative::prompt::PromptContext {
            world: &world,
            room: &room,
            all_npcs: &[],
            npcs_in_area: &[],
            player: &player,
            user_message: "test",
            history: &history,
        };

        evaluate_and_narrate_triggers(
            &mut state,
            "You enter the room.",
            &trigger_context,
            &llm_backend,
        );

        // Should have at least 2 entries: event header + narration
        assert!(
            state.narration_history.len() >= 2,
            "Expected event header + narration, got {:?}",
            state.narration_history
        );

        // First trigger-related entry should be the event header
        let event_entry = &state.narration_history[0];
        assert_eq!(event_entry.log_type, LogType::Event);
        assert_eq!(event_entry.sender, Some("Carla Introduction".to_string()));
        assert_eq!(event_entry.text, "");

        // Second entry should be the narration
        let narration_entry = &state.narration_history[1];
        assert_eq!(narration_entry.log_type, LogType::Narration);
    }
}
