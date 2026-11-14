//! The rule of the Messages a story memory does not cover yet.

use super::state::message_types::{MessageType, StoryRow};
use super::story_memory::StoryMemory;
use crate::test_support::fixtures::default_story_row;

fn rows(ids: &[u64]) -> Vec<StoryRow> {
    ids.iter()
        .map(|id| StoryRow {
            id: *id,
            ..default_story_row(MessageType::Narration)
        })
        .collect()
}

fn ids(rows: &[StoryRow]) -> Vec<u64> {
    rows.iter().map(|row| row.id).collect()
}

fn memory_up_to(covers_up_to: u64) -> StoryMemory {
    StoryMemory {
        text: "The innkeeper owes the player a favour.".to_string(),
        covers_up_to,
    }
}

#[test]
fn test_with_no_memory_every_dropped_message_is_uncovered() {
    let dropped = rows(&[1, 2, 3]);

    assert_eq!(ids(StoryMemory::uncovered(None, &dropped)), vec![1, 2, 3]);
}

#[test]
fn test_a_memory_covers_the_messages_up_to_its_mark() {
    let dropped = rows(&[1, 2, 5, 8]);

    let uncovered = StoryMemory::uncovered(Some(&memory_up_to(2)), &dropped);

    assert_eq!(ids(uncovered), vec![5, 8]);
}

#[test]
fn test_a_mark_between_ids_covers_the_older_ones() {
    let dropped = rows(&[1, 4, 9]);

    let uncovered = StoryMemory::uncovered(Some(&memory_up_to(6)), &dropped);

    assert_eq!(ids(uncovered), vec![9]);
}

#[test]
fn test_a_memory_past_every_dropped_message_leaves_none_uncovered() {
    let dropped = rows(&[1, 2, 3]);

    assert!(StoryMemory::uncovered(Some(&memory_up_to(7)), &dropped).is_empty());
}

#[test]
fn test_a_cut_that_shrank_after_a_delete_leaves_none_uncovered() {
    let dropped = rows(&[1, 2]);

    assert!(StoryMemory::uncovered(Some(&memory_up_to(5)), &dropped).is_empty());
}
