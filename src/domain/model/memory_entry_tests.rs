//! The rules of the memory entries: the cover mark, the Messages no entry covers, and the list after a writer call.

use super::memory_entry::{MemoryEntries, MemoryEntry};
use super::state::message_types::{MessageType, StoryRow};
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

fn entry(from: u64, to: u64) -> MemoryEntry {
    MemoryEntry {
        from,
        to,
        time: "Evening".to_string(),
        place: "The gate".to_string(),
        took_part: vec!["Mara".to_string()],
        others: Vec::new(),
        what_happened: format!("Messages {from} to {to} happened."),
    }
}

fn entries_covering(ranges: &[(u64, u64)]) -> MemoryEntries {
    MemoryEntries::new(ranges.iter().map(|(from, to)| entry(*from, *to)).collect())
}

#[test]
fn test_no_entry_has_no_cover_mark_and_no_last_entry() {
    let none = MemoryEntries::default();

    assert_eq!(none.covers_up_to(), None);
    assert!(none.last().is_none());
    assert!(none.entries().is_empty());
}

#[test]
fn test_the_cover_mark_is_the_to_of_the_newest_entry() {
    let list = entries_covering(&[(1, 4), (5, 9)]);

    assert_eq!(list.covers_up_to(), Some(9));
    assert_eq!(list.last(), Some(&entry(5, 9)));
}

#[test]
fn test_with_no_entry_every_dropped_message_is_uncovered() {
    let dropped = rows(&[1, 2, 3]);

    assert_eq!(
        ids(MemoryEntries::default().uncovered(&dropped)),
        vec![1, 2, 3]
    );
}

#[test]
fn test_the_entries_cover_the_messages_up_to_their_mark() {
    let dropped = rows(&[1, 2, 5, 8]);

    let uncovered = entries_covering(&[(1, 2)]).uncovered(&dropped);

    assert_eq!(ids(uncovered), vec![5, 8]);
}

#[test]
fn test_a_mark_between_ids_covers_the_older_ones() {
    let dropped = rows(&[1, 4, 9]);

    let uncovered = entries_covering(&[(1, 6)]).uncovered(&dropped);

    assert_eq!(ids(uncovered), vec![9]);
}

#[test]
fn test_entries_past_every_dropped_message_leave_none_uncovered() {
    let dropped = rows(&[1, 2, 3]);

    assert!(entries_covering(&[(1, 7)]).uncovered(&dropped).is_empty());
}

#[test]
fn test_a_cut_that_shrank_after_a_delete_leaves_none_uncovered() {
    let dropped = rows(&[1, 2]);

    assert!(entries_covering(&[(1, 5)]).uncovered(&dropped).is_empty());
}

#[test]
fn test_the_list_after_a_writer_call_ends_with_the_entries_it_wrote() {
    let before = entries_covering(&[(1, 4)]);

    let after = before.with_written(&[entry(5, 6), entry(7, 9)]);

    assert_eq!(after.entries(), &[entry(1, 4), entry(5, 6), entry(7, 9)]);
    assert_eq!(after.covers_up_to(), Some(9));
    assert_eq!(
        before.entries(),
        &[entry(1, 4)],
        "the list it came from stays"
    );
}
