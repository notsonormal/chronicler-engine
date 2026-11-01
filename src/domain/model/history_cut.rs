//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The history cut: the oldest Messages that leave the history, dropped whole and in blocks

use std::ops::Range;

/// The share of the history space that the kept history fills after a cut, in percent.
const KEPT_AFTER_CUT_PERCENT: usize = 75;

/// The Messages of one cut that left together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryBlock {
    /// The indexes of the Messages that this block dropped.
    pub dropped: Range<usize>,
    /// The index of the Message whose arrival made the cut drop. `None` for a drop
    /// that no arrival caused (the overflow guard).
    pub arrival: Option<usize>,
}

/// Where the history starts, and the blocks that went before it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryCut {
    len: usize,
    kept_from: usize,
    blocks: Vec<HistoryBlock>,
}

impl HistoryCut {
    /// Walks the history from its first Message. When the kept Messages pass
    /// `history_space`, it drops the oldest ones until the rest fills at most
    /// three quarters of the space. The same input always gives the same cut.
    pub fn of(entry_tokens: &[usize], history_space: usize) -> Self {
        let keep_after_cut = history_space * KEPT_AFTER_CUT_PERCENT / 100;
        let mut cut = Self {
            len: entry_tokens.len(),
            kept_from: 0,
            blocks: Vec::new(),
        };
        let mut kept_tokens = 0;

        for (arrival, tokens) in entry_tokens.iter().enumerate() {
            kept_tokens += tokens;
            if kept_tokens <= history_space {
                continue;
            }
            let block_start = cut.kept_from;
            while kept_tokens > keep_after_cut && cut.kept_from <= arrival {
                kept_tokens -= entry_tokens[cut.kept_from];
                cut.kept_from += 1;
            }
            cut.blocks.push(HistoryBlock {
                dropped: block_start..cut.kept_from,
                arrival: Some(arrival),
            });
        }
        cut
    }

    /// Drops `count` more of the oldest kept Messages as one block that no arrival
    /// caused. A count past the kept Messages drops all of them.
    pub fn drop_front(&mut self, count: usize) {
        let block_start = self.kept_from;
        self.kept_from = (self.kept_from + count).min(self.len);
        if self.kept_from > block_start {
            self.blocks.push(HistoryBlock {
                dropped: block_start..self.kept_from,
                arrival: None,
            });
        }
    }

    /// The index of the oldest Message the history keeps.
    pub fn kept_from(&self) -> usize {
        self.kept_from
    }

    /// The blocks that left, oldest first. They cover `0..kept_from` with no gap.
    pub fn blocks(&self) -> &[HistoryBlock] {
        &self.blocks
    }
}
