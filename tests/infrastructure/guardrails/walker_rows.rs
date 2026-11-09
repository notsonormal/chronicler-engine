//! The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.

use crate::walkers::BannedName::{NameContains, NamePrefix, StaticItem};
use crate::walkers::{BannedNameRow, CallCountRow, CallSiteRow, TypeCheckPosition, TypeCheckRow};

// Row form, parsed by `scripts/generate_guardrails_doc.py`: one struct literal per
// row, one `field: value,` line per field, the reason on the comments above the
// opening line. A value stays on one line, so a long list is a named const.
pub const BANNED_NAME_ROWS: &[BannedNameRow] = &[
    // Rule 1: every read and write names its Game. The active-Game pointer lives in
    // `ActiveGame`, so a `static` item or an `Atomic*` type in the storage adapter
    // would be a second hidden Game id.
    BannedNameRow {
        rule: 1,
        scope: "adapters/driven/storage/",
        banned: &[StaticItem, NamePrefix("Atomic")],
    },
    // Rule 6: the guard never spans a model call. `gate.rs` holds the guard for one
    // check and its writes. A narrator or an Agent named in it would call under it.
    BannedNameRow {
        rule: 6,
        scope: "application/generation/gate.rs",
        banned: &[NamePrefix("Narrator"), NameContains("Agent")],
    },
];

pub const GENERATION_CALLEES: &[&str] = &[
    "Narrator::narrate",
    "AgentRunner::after_narration",
    "AgentRunner::end_of_job",
];

// `narrator_tests.rs` is the `Narrator` seam contract, which calls its own entry point.
pub const GENERATION_CALLERS: &[&str] = &[
    "application/pipeline/job.rs",
    "application/narration/narrator_tests.rs",
];

pub const CALL_SITE_ROWS: &[CallSiteRow] = &[
    // Rule 2: `GameCatalogue` is the one reader and writer of the active-Game
    // pointer. `CurrentGame` reads through it.
    CallSiteRow {
        rule: 2,
        scope: "",
        callees: &["ActiveGame::get", "ActiveGame::lock"],
        allowed: &["application/games/catalogue.rs"],
    },
    // Rule 3: only `GameWriter` writes game records, and `gate.rs` is the one place
    // that opens a unit with a `TxToken`. The storage seam contract gets its token
    // from `test_support`, because its crate cannot call a crate-only constructor.
    CallSiteRow {
        rule: 3,
        scope: "",
        callees: &["TxToken::new"],
        allowed: &["application/generation/gate.rs", "test_support/"],
    },
    // Rule 13: only `job.rs` calls the narrator and the Agent runner hooks, so no
    // second route to a model call grows beside the Job. The row sees files, and
    // `job.rs` holds only Job code. A method call is matched through the receiver's
    // name (`narrator`, `agent_runner`), which `JobDeps` keeps.
    CallSiteRow {
        rule: 13,
        scope: "",
        callees: GENERATION_CALLEES,
        allowed: GENERATION_CALLERS,
    },
];

pub const JOB_RECORD_TYPES: &[&str] = &[
    "Game",
    "Message",
    "SwipeState",
    "Tail",
    "DockRecord",
    "WorldBundle",
];

pub const TYPE_CHECK_ROWS: &[TypeCheckRow] = &[
    // Rule 7: between steps, pass ids. A record that a step loads can go stale in
    // another step's hands, so `job.rs` holds no record field and takes no record
    // parameter.
    TypeCheckRow {
        rule: 7,
        scope: "application/pipeline/job.rs",
        banned_types: JOB_RECORD_TYPES,
        position: TypeCheckPosition::FieldsAndParameters,
    },
];

pub const CALL_COUNT_ROWS: &[CallCountRow] = &[
    // Rule 18: each writer takes the guard once, so a check and its writes are one
    // critical section. A second lock, a weak-ordering atomic in the write path, or a
    // background Agent save that checks "this Swipe is still active" outside `hold`
    // earns a `loom` seam contract.
    CallCountRow {
        rule: 18,
        scope: "application/generation/gate.rs",
        file: None,
        callee: "hold",
        max_calls: 1,
    },
];
