//! The rules of the history cut.

use super::history_cut::{HistoryBlock, HistoryCut};

fn kept_tokens(tokens: &[usize], cut: &HistoryCut) -> usize {
    tokens[cut.kept_from()..].iter().sum()
}

fn assert_blocks_cover_the_dropped_front(cut: &HistoryCut) {
    let mut next = 0;
    for block in cut.blocks() {
        assert_eq!(block.dropped.start, next, "the blocks leave no gap");
        assert!(
            block.dropped.end > block.dropped.start,
            "a block is not empty"
        );
        next = block.dropped.end;
    }
    assert_eq!(
        next,
        cut.kept_from(),
        "the blocks end where the history starts"
    );
}

#[test]
fn test_a_history_that_fits_drops_nothing() {
    let cut = HistoryCut::of(&[10, 10, 10], 100);

    assert_eq!(cut.kept_from(), 0);
    assert!(cut.blocks().is_empty());
}

#[test]
fn test_a_history_equal_to_the_space_drops_nothing() {
    let cut = HistoryCut::of(&[40, 30, 30], 100);

    assert_eq!(cut.kept_from(), 0);
    assert!(cut.blocks().is_empty());
}

#[test]
fn test_the_first_pass_of_the_space_drops_to_three_quarters() {
    let tokens = [20; 6];

    let cut = HistoryCut::of(&tokens, 100);

    assert_eq!(
        cut.kept_from(),
        3,
        "six messages of 20 pass 100, and 60 fit in 75"
    );
    assert_eq!(kept_tokens(&tokens, &cut), 60);
    assert_eq!(
        cut.blocks(),
        &[HistoryBlock {
            dropped: 0..3,
            arrival: Some(5),
        }]
    );
}

#[test]
fn test_one_more_message_after_a_cut_keeps_the_same_start() {
    let mut tokens = vec![20; 6];
    let before = HistoryCut::of(&tokens, 100);

    tokens.push(20);
    let after = HistoryCut::of(&tokens, 100);

    assert_eq!(after.kept_from(), before.kept_from());
    assert_eq!(after.blocks(), before.blocks());
}

#[test]
fn test_the_start_moves_again_when_the_space_is_passed_again() {
    let tokens = [20; 9];

    let cut = HistoryCut::of(&tokens, 100);

    assert_eq!(
        cut.blocks().len(),
        2,
        "120 passes the space, then 60 grows to 120 again"
    );
    assert_eq!(cut.blocks()[1].arrival, Some(8));
    assert_eq!(kept_tokens(&tokens, &cut), 60);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_the_same_input_gives_the_same_cut() {
    let tokens = [7, 31, 12, 55, 3, 40, 22, 18, 9, 60];

    assert_eq!(HistoryCut::of(&tokens, 120), HistoryCut::of(&tokens, 120));
}

#[test]
fn test_one_message_larger_than_the_space_drops_all() {
    let cut = HistoryCut::of(&[500], 100);

    assert_eq!(cut.kept_from(), 1);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_a_large_message_drops_the_ones_before_it_too() {
    let cut = HistoryCut::of(&[10, 10, 500, 10], 100);

    assert_eq!(
        cut.kept_from(),
        3,
        "the large Message goes, the one after it stays"
    );
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_a_space_of_zero_drops_all() {
    let cut = HistoryCut::of(&[5, 5, 5], 0);

    assert_eq!(cut.kept_from(), 3);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_an_empty_history_drops_nothing() {
    let cut = HistoryCut::of(&[], 0);

    assert_eq!(cut.kept_from(), 0);
    assert!(cut.blocks().is_empty());
}

#[test]
fn test_drop_to_free_records_one_block_with_no_arrival() {
    let mut cut = HistoryCut::of(&[20; 6], 100);

    cut.drop_to_free(&[20; 6], 40);

    assert_eq!(cut.kept_from(), 5);
    assert_eq!(
        cut.blocks().last(),
        Some(&HistoryBlock {
            dropped: 3..5,
            arrival: None,
        })
    );
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_drop_to_free_without_a_cut_before_it_starts_at_the_first_message() {
    let mut cut = HistoryCut::of(&[10, 10, 10], 100);

    cut.drop_to_free(&[10, 10, 10], 5);

    assert_eq!(cut.kept_from(), 1);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_drop_to_free_drops_the_fewest_messages_that_reach_the_excess() {
    let mut cut = HistoryCut::of(&[10, 10, 10], 100);

    cut.drop_to_free(&[10, 10, 10], 11);

    assert_eq!(cut.kept_from(), 2);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_drop_to_free_past_the_kept_messages_drops_all_and_no_more() {
    let mut cut = HistoryCut::of(&[10, 10, 10], 100);

    cut.drop_to_free(&[10, 10, 10], 500);

    assert_eq!(cut.kept_from(), 3);
    assert_blocks_cover_the_dropped_front(&cut);
}

#[test]
fn test_drop_to_free_of_nothing_records_no_block() {
    let mut cut = HistoryCut::of(&[10, 10, 10], 100);

    cut.drop_to_free(&[10, 10, 10], 0);

    assert!(cut.blocks().is_empty());
}
