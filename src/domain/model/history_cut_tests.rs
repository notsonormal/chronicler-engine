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
    assert_eq!(cut.blocks(), &[HistoryBlock { dropped: 0..3 }]);
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
fn test_drop_to_free_records_one_block() {
    let mut cut = HistoryCut::of(&[20; 6], 100);

    cut.drop_to_free(&[20; 6], 40);

    assert_eq!(cut.kept_from(), 5);
    assert_eq!(cut.blocks().last(), Some(&HistoryBlock { dropped: 3..5 }));
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

#[test]
fn test_dropped_within_one_space_takes_the_oldest_that_fit() {
    let cut = HistoryCut::of(&[40; 10], 100);
    assert_eq!(cut.kept_from(), 8);

    assert_eq!(cut.dropped_within_one_space(0), 0..2);
}

#[test]
fn test_dropped_within_one_space_takes_messages_that_fill_it_exactly() {
    let cut = HistoryCut::of(&[50, 50, 50], 100);
    assert_eq!(cut.kept_from(), 2);

    assert_eq!(cut.dropped_within_one_space(0), 0..2);
}

#[test]
fn test_dropped_within_one_space_ends_at_the_kept_history() {
    let cut = HistoryCut::of(&[40; 10], 100);

    assert_eq!(cut.dropped_within_one_space(7), 7..8);
}

#[test]
fn test_dropped_within_one_space_from_the_kept_history_is_empty() {
    let cut = HistoryCut::of(&[40; 10], 100);

    assert!(cut.dropped_within_one_space(8).is_empty());
}

#[test]
fn test_dropped_within_one_space_takes_one_message_larger_than_the_space() {
    let cut = HistoryCut::of(&[150, 10], 100);
    assert_eq!(cut.kept_from(), 1);

    assert_eq!(cut.dropped_within_one_space(0), 0..1);
}

#[test]
fn test_a_range_that_fits_one_call_is_one_call() {
    let cut = HistoryCut::of(&[10; 8], 20);

    assert_eq!(cut.writer_calls(0..3, 100), vec![0..3]);
}

#[test]
fn test_a_range_over_the_bound_splits_into_calls_that_each_fit_it() {
    let cut = HistoryCut::of(&[10; 8], 20);

    let calls = cut.writer_calls(0..6, 25);

    assert_eq!(calls, vec![0..2, 2..4, 4..6]);
}

#[test]
fn test_the_calls_cover_the_range_with_no_gap_and_no_overlap() {
    let tokens = [7, 30, 5, 5, 40, 1, 1, 1, 9];
    let cut = HistoryCut::of(&tokens, 10);

    let calls = cut.writer_calls(1..9, 35);

    let mut next = 1;
    for call in &calls {
        assert_eq!(call.start, next);
        assert!(call.end > call.start, "a call is not empty");
        next = call.end;
    }
    assert_eq!(next, 9, "no call ends before or past the range end");
}

#[test]
fn test_one_message_over_the_bound_is_a_call_alone() {
    let tokens = [5, 50, 5];
    let cut = HistoryCut::of(&tokens, 10);

    let calls = cut.writer_calls(0..3, 20);

    assert_eq!(calls, vec![0..1, 1..2, 2..3]);
}

#[test]
fn test_a_range_that_starts_late_splits_only_its_own_messages() {
    let cut = HistoryCut::of(&[10; 8], 20);

    assert_eq!(cut.writer_calls(3..6, 20), vec![3..5, 5..6]);
}

#[test]
fn test_an_empty_range_makes_no_call() {
    let cut = HistoryCut::of(&[10; 4], 20);

    assert!(cut.writer_calls(2..2, 100).is_empty());
}
