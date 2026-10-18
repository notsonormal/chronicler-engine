//! The storage seam of the new records: the story reads, the dock record, the Game's failure line and the record writes of a transaction.

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
    Swipe {
        text: text.to_string(),
        snapshot_id: None,
        room_report: None,
        event_header: None,
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
        state: None,
        maker: None,
    }
}

fn narration_swipe(text: &str, state: SwipeState) -> Swipe {
    Swipe {
        state: Some(state),
        maker: Some(Maker::Narrator),
        ..swipe(text)
    }
}

fn event_record() -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            npc_id: "npc_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: false,
        },
        name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

fn add_message(storage: &Storage, game: GameId, text: &str, room: &str) -> u64 {
    let mut message_id = 0;
    storage
        .transaction(game, tx_token(), |tx| {
            let message = Message::new(text, MessageType::Narration, None, None);
            message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &Swipe {
                    room_report: Some(room.to_string()),
                    ..narration_swipe(text, state_in(room))
                },
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
            let message = Message::new(text, MessageType::Narration, None, None);
            message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &Swipe {
                    room_report: Some(room.to_string()),
                    ..narration_swipe(text, state_in(room))
                },
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
            tx.set_swipe_state(at, &state_in("room2"), None)
        })
        .expect("the state update should succeed");

    let state = storage
        .stories()
        .swipe_state(game_id, at)
        .expect("the state read should succeed")
        .expect("the Swipe holds a state");
    assert_eq!(state.movement.current_room_id, "room2");
}

// CASE story-swipe-state-room-report: the state update writes the room report with the state
#[test]
fn the_state_update_writes_the_room_report_with_the_state() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let at = SwipeAt {
        message: message_id,
        swipe: 0,
    };

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_swipe_state(at, &state_in("room2"), Some("Room Two"))
        })
        .expect("the state update should succeed");

    let tail = storage
        .stories()
        .tail(game_id)
        .expect("the story read should succeed")
        .expect("the story holds a Message");
    assert_eq!(
        tail.last.room_report(),
        Some("Room Two"),
        "the room report is written with the state"
    );
}

// CASE story-swipe-state-none: a Swipe whose state never landed answers no state
#[test]
fn a_swipe_whose_state_never_landed_answers_none() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let (game_id, message_id) = game_with_message(&storage, "Story", "text", "room1");
    let other = SwipeAt {
        message: message_id,
        swipe: 1,
    };
    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.insert_swipe(other, &swipe("no state"))
        })
        .expect("the insert should succeed");

    let state = storage
        .stories()
        .swipe_state(game_id, other)
        .expect("the state read should succeed");

    assert!(
        state.is_none(),
        "a Swipe whose state never landed has no state"
    );
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

// CASE story-entries-entry-fields: an entry carries the active Swipe's text, room report, event header and count
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
                    room_report: Some("Room Two".to_string()),
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
        entries[0].room_report.as_deref(),
        Some("Room Two"),
        "the entry carries the active Swipe's room report"
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
    let mut message = Message::new("event text", MessageType::Narration, None, None);
    message.kind = Some(MessageKind::EventNarration(event_record()));
    storage
        .transaction(game_id, tx_token(), |tx| {
            let id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: id,
                    swipe: 0,
                },
                &Swipe {
                    event_header: Some(event_record().name),
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
        Some(MessageKind::EventNarration(event_record())),
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

// CASE story-set-swipe-text: an edited text lands and leaves the room report alone
#[test]
fn an_edited_text_lands_and_leaves_the_room_report_alone() {
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
    assert_eq!(
        entries[0].room_report.as_deref(),
        Some("room1"),
        "an edit changes the text only"
    );
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
        let message = Message::new("rolled back", MessageType::Narration, None, None);
        let id = tx.insert_message(&message)?;
        tx.insert_swipe(
            SwipeAt {
                message: id,
                swipe: 0,
            },
            &narration_swipe("rolled back", state_in("room2")),
        )?;
        tx.set_swipe_state(at, &state_in("room2"), None)
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

// CASE story-records-sqlite-only [failure]: the new records refuse the in-memory backend
#[test]
fn the_new_records_refuse_the_in_memory_backend() {
    let storage = Storage::new_in_memory();
    let game = GameId(1);

    assert!(
        storage.stories().tail(game).is_err(),
        "the story reads hold no in-memory record"
    );
    assert!(
        storage.docks().get(game).is_err(),
        "the dock read holds no in-memory record"
    );
}

// CASE story-kind-written-none: a Message of the old path carries no kind
#[test]
fn a_message_of_the_old_path_carries_no_kind() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();
    let game_id = game_without_messages(&storage, "Old path");
    let message = Message::new("old text", MessageType::Narration, None, None);
    let message_id = storage
        .insert_message(game_id, &message)
        .expect("the old insert should succeed");
    storage
        .insert_swipe(message_id, &swipe("old text"), 0)
        .expect("the old swipe insert should succeed");

    let entries = storage
        .stories()
        .entries(game_id, None, 10)
        .expect("the entries read should succeed");

    assert_eq!(
        entries[0].kind, None,
        "a Message today's code wrote carries no kind"
    );
}
