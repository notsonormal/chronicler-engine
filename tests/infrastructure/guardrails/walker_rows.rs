//! The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.

use crate::walkers::BannedName::{NameContains, NamePrefix, StaticItem};
use crate::walkers::{BannedNameRow, CallCountRow, CallSiteRow, TypeCheckRow};

// Row form, parsed by `scripts/generate_guardrails_doc.py` and documented in
// `docs/diataxis/reference/coding_standards/guardrails.md`: one struct literal
// per row, one `field: value,` line per field, the reason on the comments above
// the opening line. A row lands with the code it guards.
pub const BANNED_NAME_ROWS: &[BannedNameRow] = &[
    // Rule 1: every read and write names its Game. The active-Game pointer
    // lives in `ActiveGame`, so a `static` item or an `Atomic*` type in the storage
    // adapter would be a second hidden Game id.
    BannedNameRow {
        rule: 1,
        scope: "adapters/driven/storage/",
        banned: &[StaticItem, NamePrefix("Atomic")],
    },
    // Rule 6: the guard never spans a model call. `gate.rs` holds the guard for one
    // check and its writes, so the narrator or an Agent (an analysis Agent, an
    // Options Agent, their runner) named in that file would let a call run under it.
    BannedNameRow {
        rule: 6,
        scope: "application/generation/gate.rs",
        banned: &[NamePrefix("Narrator"), NameContains("Agent")],
    },
];

pub const CALL_SITE_ROWS: &[CallSiteRow] = &[
    // Rule 2: the active-Game pointer has one reader and one writer.
    // `GameCatalogue` is both — `CurrentGame` reads through it once per request
    // — and `ActiveGame::set` is `pub(super)`, so the compiler holds the write
    // side too.
    CallSiteRow {
        rule: 2,
        scope: "",
        callees: &["ActiveGame::get", "ActiveGame::set"],
        allowed: &["application/games/catalogue.rs"],
    },
    // Rule 3: only `GameWriter` writes game records. The unit of record writes is
    // opened with a capability token, so `gate.rs` is where a unit starts and no
    // other caller can open one; the storage seam contract gets its token from
    // `test_support`, because its crate cannot call a crate-only constructor.
    CallSiteRow {
        rule: 3,
        scope: "",
        callees: &["TxToken::new"],
        allowed: &["application/generation/gate.rs", "test_support/"],
    },
];

pub const TYPE_CHECK_ROWS: &[TypeCheckRow] = &[];

pub const CALL_COUNT_ROWS: &[CallCountRow] = &[
    // Rule 18: each writer takes the guard once, so a check and its writes are one
    // critical section. Look again when a second lock (for example one guard per
    // Game), an atomic with weak ordering in the write path, or a background Agent
    // save that checks "this Swipe is still active" outside `hold` arrives; then
    // the new primitive earns a `loom` seam contract.
    CallCountRow {
        rule: 18,
        scope: "application/generation/gate.rs",
        file: None,
        callee: "hold",
        max_calls: 1,
    },
];
