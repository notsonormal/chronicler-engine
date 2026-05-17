//! [DOC: docs/reference/testing.md]

#[path = "game_service/advanced.rs"]
mod advanced;
#[path = "game_service/basic.rs"]
mod basic;
mod test_data;

use std::sync::Arc;

use chronicler_engine::engine::game_service::DefaultGameService;
use chronicler_engine::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::model::state::GameState;

use chronicler_engine::narrative::llm::MockBackend;

pub fn wait_for_generation_complete(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
    timeout_ms: u64,
) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    while start.elapsed() < timeout {
        if let Ok(Some(snap)) = ctx.snapshot_storage.load_latest() {
            let guard = GameState::from_snapshot(
                &snap,
                ctx.world.clone(),
                ctx.map.clone(),
                ctx.player.clone(),
                (*ctx.npcs).clone(),
            );
            if !guard.narrative.generation.status.is_generating() {
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}

pub fn latest_state(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
) -> GameState {
    let snap = ctx.snapshot_storage.load_latest().unwrap().unwrap();
    let mut state = GameState::from_snapshot(
        &snap,
        ctx.world.clone(),
        ctx.map.clone(),
        ctx.player.clone(),
        (*ctx.npcs).clone(),
    );
    if let Ok(msgs) = ctx.message_storage.load_messages() {
        if !msgs.is_empty() {
            state.narrative.messages = msgs;
        }
    }
    state
}

pub fn failing_service() -> DefaultGameService {
    DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    )
}

pub fn create_test_state_with_trigger_npc() -> GameState {
    crate::test_data::create_test_state_with_npcs(
        vec!["shopkeeper".to_string()],
        vec![NpcCard {
            id: "shopkeeper".into(),
            sheet: CharacterSheet {
                name: "Shopkeeper Sarah".into(),
                description: "A shrewd shopkeeper".into(),
                personality: "Business-minded".into(),
                scenario: "Runs the shop".into(),
                example_dialogue: "Welcome!".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![chronicler_engine::model::trigger::Trigger {
                condition: chronicler_engine::model::trigger::TriggerCondition::TimesMet(
                    chronicler_engine::model::trigger::ComparisonOperator::Eq,
                    0,
                ),
                action: chronicler_engine::model::trigger::TriggerAction {
                    name: "Greeting".into(),
                    narration_prompt: "The shopkeeper looks up with a smile.".into(),
                },
                repeat: false,
                room_id: None,
            }],
            relationships: vec![],
        }],
    )
}
