//! The storage seam of the new records: the story reads, the dock record, the Game's failure line and the record writes of a transaction.

use chronicler_engine::adapters::driven::storage::db::DbPool;
use chronicler_engine::adapters::driven::storage::{Storage, TestOverride, TxWrite};
use chronicler_engine::domain::model::dock::{DockRecord, Notice};
use chronicler_engine::domain::model::game::{GameId, NewGame};
use chronicler_engine::domain::model::message::{Maker, Message, Swipe, SwipeAt};
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind,
};
use chronicler_engine::domain::model::state::message_types::{MessageKind, MessageType};
use chronicler_engine::domain::model::state::movement::MovementState;
use chronicler_engine::domain::model::state::scene_state::SceneState;
use chronicler_engine::domain::model::state::swipe_state::SwipeState;
use chronicler_engine::domain::model::state::trigger_context::{EventRecord, TriggerRef};
use chronicler_engine::domain::model::trigger::NpcEncounterLog;
use chronicler_engine::error::EngineError;
use chronicler_engine::test_support::tx_token;

const WORLD_KEY: &str = "record_world";

fn new_game(name: &str) -> NewGame {
    NewGame {
        world_name: "Record World".to_string(),
        world_key: WORLD_KEY.to_string(),
        persona_key: "record_persona".to_string(),
        persona_name: "Record Persona".to_string(),
        name: name.to_string(),
        narrator_mode: NarratorMode::Novel,
        narrative_perspective: NarrativePerspective::Third,
        narrative_tense: NarrativeTense::Past,
        system_prompt_preset_id: "system_default".to_string(),
        quantifier_prompt_preset_id: "quantifier_default".to_string(),
        impersonate_prompt_preset_id: "impersonate_default".to_string(),
        options_prompt_preset_id: "options_default".to_string(),
        options_always_on: false,
    }
}

fn state_in(room: &str) -> SwipeState {
    SwipeState {
        movement: MovementState {
            current_room_id: room.to_string(),
            dynamic_rooms: Default::default(),
        },
        scene: SceneState::default(),
        npc_encounter_log: NpcEncounterLog::default(),
        trigger_context: None,
    }
}

fn swipe(text: &str) -> Swipe {
    Swipe::new(text, Maker::Narrator, state_in("room1"))
}

fn narration_swipe(text: &str, state: SwipeState) -> Swipe {
    Swipe::new(text, Maker::Narrator, state)
}

/// The Message row only: its Swipes are written one by one.
fn narration_message(text: &str) -> Message {
    Message::new(MessageType::Narration, MessageKind::Narration, swipe(text))
}

fn event_record() -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            npc_id: "npc_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: false,
        },
        header_name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

fn add_message(storage: &Storage, game: GameId, text: &str, room: &str) -> u64 {
    let mut message_id = 0;
    storage
        .transaction(game, tx_token(), |tx| {
            let message = narration_message(text);
            message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &narration_swipe(text, state_in(room)),
            )
        })
        .expect("the unit should succeed");
    message_id
}

fn game_with_message(storage: &Storage, name: &str, text: &str, room: &str) -> (GameId, u64) {
    let mut message_id = 0;
    let mut game_id = GameId(0);
    storage
        .transaction_new_game(&new_game(name), tx_token(), |new_game_id, tx| {
            game_id = new_game_id;
            let message = narration_message(text);
            message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &narration_swipe(text, state_in(room)),
            )
        })
        .expect("the create unit should succeed");
    (game_id, message_id)
}

fn game_without_messages(storage: &Storage, name: &str) -> GameId {
    storage
        .transaction_new_game(&new_game(name), tx_token(), |game_id, _tx| Ok(game_id))
        .expect("the create unit should succeed")
}

// CASE story-tail-last-message: `tail` answers the last Message with all its Swipes and the count
#[test]
fn tail_answers_the_last_message_with_all_its_swipes_and_the_count() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, first_id) = game_with_message(&storage, "Story", "first text", "room1");
    add_message(&storage, game_id, "second text", "room2");

    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed")
        .expect("the story holds a Message");

    assert_eq!(tail.count, 2, "both Messages of the story are counted");
    assert_eq!(tail.last.id, first_id + 1, "the last Message is the newest");
    assert_eq!(tail.last.text(), "second text");
    assert_eq!(tail.last.swipes.len(), 1, "all its Swipes are read");
}

// CASE story-tail-none: a Game with no Message answers no tail
#[test]
fn tail_answers_none_for_a_game_with_no_message() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Empty");

    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed");

    assert!(tail.is_none(), "no Message is no story");
}

// CASE story-tail-state-before-last: the state before the last Message comes with the Swipe it belongs to
#[test]
fn tail_answers_the_state_before_the_last_message_with_its_swipe() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, first_id) = game_with_message(&storage, "Story", "first text", "room1");
    add_message(&storage, game_id, "second text", "room2");

    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed")
        .expect("the story holds a Message");
    let (at, state) = tail
        .state_before_last
        .clone()
        .expect("the Message before the last carries its state");

    assert_eq!(
        at,
        SwipeAt {
            message: first_id,
            swipe: 0
        },
        "the state comes with the Swipe it belongs to"
    );
    assert_eq!(
        state.movement.current_room_id, "room1",
        "the state before the last Message is the first Message's state"
    );
}

// CASE story-swipe-state: the state a unit wrote reads back
#[test]
fn the_state_a_unit_wrote_reads_back() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let at = SwipeAt {
        message: message_id,
        swipe: 0,
    };

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_swipe_state(at, &state_in("room2"))
        })
        .expect("the state update should succeed");

    let state = storage
        .stories()
        .swipe_state(game_id, at)
        .expect("the state read should succeed")
        .expect("the Swipe holds a state");
    assert_eq!(state.movement.current_room_id, "room2");
}

// CASE story-swipe-state-none: a Swipe the story does not hold answers no state
#[test]
fn a_swipe_the_story_does_not_hold_answers_none() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");

    let state = storage
        .stories()
        .swipe_state(
            game_id,
            SwipeAt {
                message: message_id,
                swipe: 1,
            },
        )
        .expect("the state read should succeed");

    assert!(state.is_none(), "a missing Swipe has no state");
}

// CASE story-swipe-state-other-game [failure]: a read of another Game's Swipe is refused
#[test]
fn a_state_read_of_another_game_is_refused() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (_game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let other_game = game_without_messages(&storage, "Other");

    let result = storage.stories().swipe_state(
        other_game,
        SwipeAt {
            message: message_id,
            swipe: 0,
        },
    );

    assert!(
        matches!(result, Err(EngineError::MessageNotFound(_))),
        "a read names its Game, so another Game's Message is not found"
    );
}

// CASE story-has-text: a story whose Message holds text answers that it has some
#[test]
fn a_story_with_text_answers_that_it_has_some() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, _message_id) = game_with_message(&storage, "Story", "text", "room1");

    let has_text = storage
        .stories()
        .has_text(game_id)
        .expect("the story text read should succeed");

    assert!(has_text, "the Message holds text");
}

// CASE story-has-text-none: a story whose Messages hold no text answers that it has none
#[test]
fn a_story_of_empty_text_answers_that_it_has_none() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, _message_id) = game_with_message(&storage, "Story", "", "room1");

    let has_text = storage
        .stories()
        .has_text(game_id)
        .expect("the story text read should succeed");

    assert!(!has_text, "no Message of the story holds text");
}

// CASE story-write-other-game-message [failure]: a write to another Game's Message is refused
#[test]
fn a_write_to_another_game_message_is_refused() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (_game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let other_game = game_without_messages(&storage, "Other");

    let result = storage.transaction(other_game, tx_token(), |tx| {
        tx.set_active_swipe(SwipeAt {
            message: message_id,
            swipe: 0,
        })
    });

    assert!(
        matches!(result, Err(EngineError::MessageNotFound(_))),
        "a write names its Game, so another Game's Message is refused"
    );
}

// CASE story-window-newest-entries: the window read takes the newest entries, oldest first
#[test]
fn the_window_read_takes_the_newest_entries_oldest_first() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, _first_id) = game_with_message(&storage, "Story", "first text", "room1");
    add_message(&storage, game_id, "second text", "room2");
    add_message(&storage, game_id, "third text", "room3");

    let window = storage
        .stories()
        .window(game_id, 2)
        .expect("the window read should succeed");

    let texts: Vec<&str> = window
        .entries
        .iter()
        .map(|(entry, _)| entry.text.as_str())
        .collect();
    assert_eq!(
        texts,
        ["second text", "third text"],
        "the newest two, oldest first"
    );
    let rooms: Vec<&str> = window
        .entries
        .iter()
        .map(|(_, state)| state.movement.current_room_id.as_str())
        .collect();
    assert_eq!(
        rooms,
        ["room2", "room3"],
        "each entry carries the state of its active Swipe"
    );
}

// CASE story-window-retrigger-allowed: the window read carries whether the end of the story allows a Retrigger
#[test]
fn the_window_read_carries_whether_the_end_of_the_story_allows_a_retrigger() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, _first_id) = game_with_message(&storage, "Story", "first text", "room1");
    let window = storage
        .stories()
        .window(game_id, 50)
        .expect("the window read should succeed");
    assert!(
        !window.retrigger_allowed,
        "a story whose last state holds no trigger context allows no Retrigger"
    );

    let triggered = SwipeState {
        trigger_context: Some(event_record().trigger),
        ..state_in("room1")
    };
    storage
        .transaction(game_id, tx_token(), |tx| {
            let message = narration_message("second text");
            let message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &narration_swipe("second text", triggered),
            )
        })
        .expect("the unit should succeed");

    let window = storage
        .stories()
        .window(game_id, 50)
        .expect("the window read should succeed");
    assert!(
        window.retrigger_allowed,
        "a last Narration with a trigger context allows a Retrigger"
    );
}

// CASE story-entries-window: the entries read takes the newest Messages before a Message
#[test]
fn the_entries_read_takes_the_newest_messages_before_a_message() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, first_id) = game_with_message(&storage, "Story", "first text", "room1");
    add_message(&storage, game_id, "second text", "room2");
    add_message(&storage, game_id, "third text", "room3");

    let entries = storage
        .stories()
        .entries(game_id, Some(first_id + 2), 1)
        .expect("the entries read should succeed");

    assert_eq!(entries.len(), 1, "the limit picks one Message");
    assert_eq!(entries[0].text, "second text", "the newest before the cut");
}

// CASE story-entries-entry-fields: an entry carries the active Swipe's text, event header and count
#[test]
fn an_entry_carries_the_active_swipe_and_its_headers() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 1,
                },
                &Swipe {
                    event_header: Some("Event".to_string()),
                    ..swipe("other text")
                },
            )?;
            tx.set_active_swipe(SwipeAt {
                message: message_id,
                swipe: 1,
            })
        })
        .expect("the unit should succeed");

    let entries = storage
        .stories()
        .entries(game_id, None, 10)
        .expect("the entries read should succeed");

    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].text, "other text",
        "the entry reads the active Swipe's text"
    );
    assert_eq!(
        entries[0].event_header.as_deref(),
        Some("Event"),
        "the entry carries the active Swipe's event header"
    );
    assert_eq!(
        entries[0].swipe_count, 2,
        "the entry counts the Swipes of the Message"
    );
}

// CASE story-entries-kind: an inserted Message kind reads back through the entries read
#[test]
fn an_inserted_message_kind_reads_back_through_the_entries_read() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, _first_id) = game_with_message(&storage, "Story", "text", "room1");
    let message = Message::new(
        MessageType::Narration,
        MessageKind::EventNarration(event_record()),
        swipe("event text"),
    );
    storage
        .transaction(game_id, tx_token(), |tx| {
            let id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: id,
                    swipe: 0,
                },
                &Swipe {
                    event_header: Some(event_record().header_name),
                    ..narration_swipe("event text", state_in("room2"))
                },
            )
        })
        .expect("the unit should succeed");

    let entries = storage
        .stories()
        .entries(game_id, None, 10)
        .expect("the entries read should succeed");

    assert_eq!(
        entries[1].kind,
        MessageKind::EventNarration(event_record()),
        "the inserted kind reads back"
    );
}

// CASE story-delete-message: a deleted Message leaves the story with its Swipes
#[test]
fn a_deleted_message_leaves_the_story() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, first_id) = game_with_message(&storage, "Story", "first text", "room1");
    add_message(&storage, game_id, "second text", "room2");

    storage
        .transaction(game_id, tx_token(), |tx| tx.delete_message(first_id))
        .expect("the delete unit should succeed");

    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed")
        .expect("the second Message stays");
    assert_eq!(tail.count, 1, "the deleted Message is gone");
    assert_eq!(tail.last.text(), "second text");
}

// CASE story-set-swipe-text: an edited text lands
#[test]
fn an_edited_text_lands() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let at = SwipeAt {
        message: message_id,
        swipe: 0,
    };

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_swipe_text(at, "edited text")
        })
        .expect("the edit unit should succeed");

    let entries = storage
        .stories()
        .entries(game_id, None, 10)
        .expect("the entries read should succeed");
    assert_eq!(entries[0].text, "edited text");
}

// CASE story-write-injection [failure]: a unit whose later record write fails leaves no row
#[test]
fn the_failed_state_update_rolls_the_whole_unit_back() {
    let (storage, handle) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let at = SwipeAt {
        message: message_id,
        swipe: 0,
    };
    handle.set_write(
        TxWrite::SetSwipeState,
        TestOverride::internal("the state update fails"),
    );

    let result = storage.transaction(game_id, tx_token(), |tx| {
        let message = narration_message("rolled back");
        let id = tx.insert_message(&message)?;
        tx.insert_swipe(
            SwipeAt {
                message: id,
                swipe: 0,
            },
            &narration_swipe("rolled back", state_in("room2")),
        )?;
        tx.set_swipe_state(at, &state_in("room2"))
    });

    assert!(
        matches!(result, Err(EngineError::Internal(_))),
        "the injected failure fails the unit"
    );
    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed")
        .expect("the first Message stays");
    assert_eq!(tail.count, 1, "the inserted Message rolled back");
    assert_eq!(
        storage
            .stories()
            .swipe_state(game_id, at)
            .expect("the state read should succeed")
            .expect("the first state stays")
            .movement
            .current_room_id,
        "room1",
        "the refused state update changed nothing"
    );
}

// CASE story-game-failure-set: the failure line lands on the Game and reads back
#[test]
fn the_failure_line_lands_on_the_game_and_reads_back() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Failing");
    let failure = GenerationFailure::new(
        GenerationFailureKind::UnreadableAnswer,
        "the model wrote nothing",
    );

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_game_failure(Some(&failure))
        })
        .expect("the failure unit should succeed");

    let stored = storage
        .get_game(game_id.0)
        .expect("the Game read should succeed")
        .expect("the Game row stays");
    assert_eq!(stored.failure, Some(failure), "the line reads back");
}

// CASE story-game-failure-clear: a cleared failure line reads back as none
#[test]
fn a_cleared_failure_line_reads_back_as_none() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Failing");
    let failure = GenerationFailure::new(
        GenerationFailureKind::UnreadableAnswer,
        "the model wrote nothing",
    );
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_game_failure(Some(&failure))
        })
        .expect("the failure unit should succeed");

    storage
        .transaction(game_id, tx_token(), |tx| tx.set_game_failure(None))
        .expect("the clearing unit should succeed");

    let cleared = storage
        .get_game(game_id.0)
        .expect("the Game read should succeed")
        .expect("the Game row stays");
    assert_eq!(cleared.failure, None, "the cleared line reads back as none");
}

// CASE story-dock-roundtrip: the dock record lands whole and reads back
#[test]
fn the_dock_record_lands_whole_and_reads_back() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Docked");
    let record = DockRecord {
        offered: vec!["look around".to_string(), "open the gate".to_string()],
        notice: None,
    };

    storage
        .transaction(game_id, tx_token(), |tx| tx.put_dock(&record))
        .expect("the dock unit should succeed");

    let stored = storage
        .docks()
        .get(game_id)
        .expect("the dock read should succeed")
        .expect("the dock record stays");
    assert_eq!(stored, record);
}

// CASE story-dock-notice: the dock record carries its notice
#[test]
fn the_dock_record_carries_its_notice() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Docked");
    let record = DockRecord {
        offered: Vec::new(),
        notice: Some(Notice {
            reason: "Options generation failed".to_string(),
            raw: "the model wrote no option".to_string(),
        }),
    };

    storage
        .transaction(game_id, tx_token(), |tx| tx.put_dock(&record))
        .expect("the dock unit should succeed");

    let stored = storage
        .docks()
        .get(game_id)
        .expect("the dock read should succeed")
        .expect("the dock record stays");
    assert_eq!(stored.notice, record.notice);
}

// CASE story-dock-clear: a cleared dock record keeps its row with no offered set and no notice
#[test]
fn a_cleared_dock_record_keeps_its_row() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Docked");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.put_dock(&DockRecord {
                offered: vec!["look around".to_string()],
                notice: Some(Notice {
                    reason: "Options generation failed".to_string(),
                    raw: "the model wrote no option".to_string(),
                }),
            })
        })
        .expect("the dock unit should succeed");

    storage
        .transaction(game_id, tx_token(), |tx| tx.clear_dock())
        .expect("the clearing unit should succeed");

    let stored = storage
        .docks()
        .get(game_id)
        .expect("the dock read should succeed")
        .expect("a Game holds one dock record");
    assert_eq!(
        stored,
        DockRecord::default(),
        "the record stays with no offered set and no notice"
    );
}

// CASE story-active-swipe: the active Swipe of a Message of the Game reads back, and another Game's Message answers none
#[test]
fn the_active_swipe_of_a_message_reads_back() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let other_game = game_without_messages(&storage, "Other");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 1,
                },
                &swipe("second"),
            )?;
            tx.set_active_swipe(SwipeAt {
                message: message_id,
                swipe: 1,
            })
        })
        .expect("the unit should succeed");

    assert_eq!(
        storage.stories().active_swipe(game_id, message_id).unwrap(),
        Some(SwipeAt {
            message: message_id,
            swipe: 1
        }),
        "the read names the active Swipe"
    );
    assert_eq!(
        storage
            .stories()
            .active_swipe(other_game, message_id)
            .unwrap(),
        None,
        "another Game's Message is not found"
    );
    assert_eq!(
        storage.stories().active_swipe(game_id, 9999).unwrap(),
        None,
        "a Message the story does not hold is not found"
    );
}

// CASE story-messages: the whole story reads back oldest first, each Message with all its Swipes
#[test]
fn the_whole_story_reads_back_with_all_swipes() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, first_id) = game_with_message(&storage, "Story", "first", "room1");
    let second_id = add_message(&storage, game_id, "second", "room2");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.insert_swipe(
                SwipeAt {
                    message: second_id,
                    swipe: 1,
                },
                &swipe("second again"),
            )
        })
        .expect("the swipe insert should succeed");

    let messages = storage
        .stories()
        .messages(game_id)
        .expect("the messages read should succeed");

    assert_eq!(
        messages.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![first_id, second_id],
        "oldest first"
    );
    assert_eq!(messages[0].swipes.len(), 1);
    assert_eq!(
        messages[1]
            .swipes
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>(),
        vec!["second", "second again"],
        "every Swipe of a Message comes with it"
    );
}

// CASE story-swipe-stored-inputs: the inputs a Swipe was made from read back with it
#[test]
fn the_stored_inputs_of_a_swipe_read_back() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Story");
    storage
        .transaction(game_id, tx_token(), |tx| {
            let id = tx.insert_message(&narration_message("I ask about the artifact."))?;
            tx.insert_swipe(
                SwipeAt {
                    message: id,
                    swipe: 0,
                },
                &Swipe {
                    impersonated: true,
                    guide_text: Some("make it ominous".to_string()),
                    impersonate_direction: Some("as the player".to_string()),
                    ..swipe("I ask about the artifact.")
                },
            )
        })
        .expect("the unit should succeed");

    let tail = storage
        .stories()
        .tail(game_id)
        .unwrap()
        .expect("the story holds a Message");
    let inputs = tail.last.swipes[0].stored_inputs();

    assert!(inputs.impersonated);
    assert_eq!(inputs.guide_text.as_deref(), Some("make it ominous"));
    assert_eq!(
        inputs.impersonate_direction.as_deref(),
        Some("as the player")
    );
}

// CASE story-tail-active-swipe-out-of-bounds: an active index past the Swipes reads as the first Swipe
#[test]
fn an_active_swipe_index_past_the_swipes_reads_as_the_first() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "first swipe", "room1");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_active_swipe(SwipeAt {
                message: message_id,
                swipe: 99,
            })
        })
        .expect("the unit should succeed");

    let tail = storage
        .stories()
        .tail(game_id)
        .unwrap()
        .expect("the story holds a Message");

    assert_eq!(tail.last.text(), "first swipe");
    assert_eq!(tail.last.active_swipe_index, 0);
}

// CASE message-without-swipe [failure]: a Message row with no Swipe row gives a load error
#[test]
fn a_message_with_no_swipe_is_a_load_error() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Story");
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.insert_message(&narration_message("no swipe follows"))
        })
        .expect("the unit should succeed");

    let error = storage
        .stories()
        .tail(game_id)
        .expect_err("a Message with no Swipe cannot load");

    assert!(
        matches!(&error, EngineError::Internal(invariant) if invariant.invariant.contains("no swipe")),
        "the load error names the missing Swipe, got: {error}"
    );
}

// CASE db-error-context [failure]: a read that SQLite refuses gives `Database` with the context of its method
#[test]
fn a_failed_read_names_its_method_in_the_database_error() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    let storage = Storage::new_sqlite(pool.clone());
    let game_id = game_without_messages(&storage, "Story");
    pool.conn()
        .execute("DROP TABLE game_docks", [])
        .expect("the table should drop");
    pool.conn()
        .execute("DROP TABLE message_swipes", [])
        .expect("the table should drop");

    let dock_error = storage
        .docks()
        .get(game_id)
        .expect_err("the dock read cannot run without its table");
    let text_error = storage
        .stories()
        .has_text(game_id)
        .expect_err("the text read cannot run without the Swipe table");

    assert!(
        matches!(
            dock_error,
            EngineError::Database {
                context: "Failed to read the dock record",
                ..
            }
        ),
        "the dock read names itself, got: {dock_error}"
    );
    assert!(
        matches!(
            text_error,
            EngineError::Database {
                context: "Failed to read the story text",
                ..
            }
        ),
        "the text read names itself, got: {text_error}"
    );
}
