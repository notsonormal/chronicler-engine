//! Unit tests for StatePatch merge semantics

use crate::domain::model::agent::{Confidence, StatePatch};

fn scene_patch(
    npc_ids: Vec<&str>,
    destination: Option<&str>,
    confidence: Confidence,
) -> StatePatch {
    StatePatch {
        npc_ids: npc_ids.into_iter().map(String::from).collect(),
        movement_destination: destination.map(String::from),
        confidence,
    }
}

#[test]
fn test_merge_npc_ids_union_preserves_order() {
    let a = scene_patch(vec!["alice", "bob"], None, Confidence::High);
    let b = scene_patch(vec!["charlie", "bob"], None, Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch { npc_ids, .. } = merged;
    assert_eq!(npc_ids, vec!["alice", "bob", "charlie"]);
}

#[test]
fn test_merge_npc_ids_empty_first() {
    let a = scene_patch(vec![], None, Confidence::High);
    let b = scene_patch(vec!["alice", "bob"], None, Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch { npc_ids, .. } = merged;
    assert_eq!(npc_ids, vec!["alice", "bob"]);
}

#[test]
fn test_merge_npc_ids_empty_second() {
    let a = scene_patch(vec!["alice", "bob"], None, Confidence::High);
    let b = scene_patch(vec![], None, Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch { npc_ids, .. } = merged;
    assert_eq!(npc_ids, vec!["alice", "bob"]);
}

#[test]
fn test_merge_movement_destination_keeps_first() {
    let a = scene_patch(vec![], Some("room_a"), Confidence::High);
    let b = scene_patch(vec![], Some("room_b"), Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch {
        movement_destination,
        ..
    } = merged;
    assert_eq!(movement_destination, Some("room_a".to_string()));
}

#[test]
fn test_merge_movement_destination_falls_back_to_second() {
    let a = scene_patch(vec![], None, Confidence::High);
    let b = scene_patch(vec![], Some("room_b"), Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch {
        movement_destination,
        ..
    } = merged;
    assert_eq!(movement_destination, Some("room_b".to_string()));
}

#[test]
fn test_merge_confidence_takes_minimum_high_medium() {
    let a = scene_patch(vec![], None, Confidence::High);
    let b = scene_patch(vec![], None, Confidence::Medium);

    let merged = a.merge(b);

    let StatePatch { confidence, .. } = merged;
    assert_eq!(confidence, Confidence::Medium);
}

#[test]
fn test_merge_confidence_takes_minimum_medium_low() {
    let a = scene_patch(vec![], None, Confidence::Medium);
    let b = scene_patch(vec![], None, Confidence::Low);

    let merged = a.merge(b);

    let StatePatch { confidence, .. } = merged;
    assert_eq!(confidence, Confidence::Low);
}

#[test]
fn test_merge_confidence_takes_minimum_high_low() {
    let a = scene_patch(vec![], None, Confidence::High);
    let b = scene_patch(vec![], None, Confidence::Low);

    let merged = a.merge(b);

    let StatePatch { confidence, .. } = merged;
    assert_eq!(confidence, Confidence::Low);
}

#[test]
fn test_merge_confidence_both_high() {
    let a = scene_patch(vec![], None, Confidence::High);
    let b = scene_patch(vec![], None, Confidence::High);

    let merged = a.merge(b);

    let StatePatch { confidence, .. } = merged;
    assert_eq!(confidence, Confidence::High);
}

#[test]
fn test_merge_all_fields() {
    let a = scene_patch(vec!["alice"], Some("room_a"), Confidence::High);
    let b = scene_patch(vec!["bob"], Some("room_b"), Confidence::Low);

    let merged = a.merge(b);

    let StatePatch {
        npc_ids,
        movement_destination,
        confidence,
    } = merged;
    assert_eq!(npc_ids, vec!["alice", "bob"]);
    assert_eq!(movement_destination, Some("room_a".to_string()));
    assert_eq!(confidence, Confidence::Low);
}

#[test]
fn test_merge_multiple_chains() {
    let patch1 = scene_patch(vec!["a"], Some("x"), Confidence::High);
    let patch2 = scene_patch(vec!["b"], Some("y"), Confidence::Medium);
    let patch3 = scene_patch(vec!["c"], None, Confidence::Low);

    let merged = patch1.merge(patch2).merge(patch3);

    let StatePatch {
        npc_ids,
        movement_destination,
        confidence,
    } = merged;
    assert_eq!(npc_ids, vec!["a", "b", "c"]);
    assert_eq!(movement_destination, Some("x".to_string()));
    assert_eq!(confidence, Confidence::Low);
}
