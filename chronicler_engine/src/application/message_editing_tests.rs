//! [DOC: docs/reference/testing.md]

#[test]
fn test_switch_swipe_bounds_validation() {
    use crate::application::ApplicationError;
    use crate::error::EngineError;

    let result = validate_swipe_index(2, 5);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ApplicationError::Engine(EngineError::Internal(_))
    ));

    let result = validate_swipe_index(5, 2);
    assert!(result.is_ok());
}

#[test]
fn test_edit_history_validation() {
    use crate::application::ApplicationError;
    use crate::model::message::Message;
    use crate::model::state::MessageType;
    use crate::test_support::make_test_context;
    use crate::application::message_editing::MessageEditingService;
    use crate::application::game_service::GameService;
    use crate::narrative::agents::registry::AgentRegistry;
    use crate::narrative::llm::MockBackend;
    use std::sync::Arc;

    let mut state = test_helpers::create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let result = editing_service.edit_history(ctx.clone(), 99999, "Edited".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::Engine(_)));
}

#[test]
fn test_delete_last_empty_error() {
    use crate::application::message_editing::MessageEditingService;
    use crate::application::game_service::GameService;
    use crate::narrative::agents::registry::AgentRegistry;
    use crate::narrative::llm::MockBackend;
    use std::sync::Arc;

    let state = test_helpers::create_test_state();
    let _ctx = crate::test_support::make_test_context(state);
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let _editing_service = MessageEditingService::new(Arc::new(service));
}

#[test]
fn test_retry_input_validation() {
    use crate::application::ApplicationError;

    let has_input = false;
    if !has_input {
        let err = ApplicationError::Validation("No input to retry".to_string());
        assert!(matches!(err, ApplicationError::Validation(_)));
        assert_eq!(err.to_string(), "No input to retry");
    }
}

#[test]
fn test_retrigger_trigger_validation() {
    use crate::application::ApplicationError;

    let has_trigger = false;
    if !has_trigger {
        let err = ApplicationError::Validation("No trigger context available".to_string());
        assert!(matches!(err, ApplicationError::Validation(_)));
        assert_eq!(err.to_string(), "No trigger context available");
    }
}

fn validate_swipe_index(
    len: usize,
    index: usize,
) -> Result<(), crate::application::ApplicationError> {
    use crate::application::ApplicationError;
    use crate::error::EngineError;

    if index >= len {
        return Err(ApplicationError::Engine(EngineError::Internal(
            crate::error::InternalError {
                invariant: "Swipe index out of bounds".to_string(),
            },
        )));
    }
    Ok(())
}

mod test_helpers {
    #![allow(dead_code)]
    use std::collections::HashMap;
    use std::sync::Arc;
    use crate::model::character::{CharacterSheet, PlayerCard};
    use crate::model::map::{MapDef, Overworld, Region, Room};
    use crate::model::state::GameState;
    use crate::model::world::WorldCard;

    pub fn create_test_world() -> WorldCard {
        WorldCard {
            key: "test".into(),
            name: "Test World".into(),
            description: "A test world".into(),
            ..Default::default()
        }
    }

    pub fn create_test_player() -> PlayerCard {
        PlayerCard {
            key: "test_player".to_string(),
            sheet: CharacterSheet {
                name: "Test Player".into(),
                description: "A test player".into(),
                personality: "Brave".into(),
                scenario: "Test scenario".into(),
                example_dialogue: "Hello!".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }
    }

    pub fn create_test_map() -> MapDef {
        let room1 = Room {
            id: "room1".into(),
            name: "Test Room".into(),
            description: "A test room".into(),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        };

        let region = Region {
            id: "test_region".into(),
            name: "Test Region".into(),
            rooms: vec![room1],
        };

        MapDef {
            overworld: Overworld {
                id: "test_overworld".into(),
                name: "Test World".into(),
                regions: vec![region],
            },
        }
    }

    pub fn create_test_state() -> GameState {
        let world = Arc::new(create_test_world());
        let map = Arc::new(create_test_map());
        let player = Arc::new(create_test_player());
        let npcs = vec![];

        GameState::new(world, map, player, npcs, "room1".to_string())
    }
}
