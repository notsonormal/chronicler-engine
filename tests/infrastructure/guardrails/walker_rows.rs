//! The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.

use crate::walkers::BannedName::{NameContains, NamePrefix, StaticItem};
use crate::walkers::{BannedNameRow, CallCountRow, CallSiteRow, TypeCheckPosition, TypeCheckRow};

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
    // check and its writes, so the narrator or an Agent (an after-narration Agent, an
    // end-of-Job Agent, their runner) named in that file would let a call run under it.
    BannedNameRow {
        rule: 6,
        scope: "application/generation/gate.rs",
        banned: &[NamePrefix("Narrator"), NameContains("Agent")],
    },
];

// The calls rule 13 guards: the three paths fit no single row field line, so the
// list is one named const the row points at.
pub const GENERATION_CALLEES: &[&str] = &[
    "Narrator::narrate",
    "AgentRunner::after_narration",
    "AgentRunner::end_of_job",
];

// What may make those calls: the Job, and the `Narrator` seam contract, which calls
// its own entry point. Two paths fit no single row field line.
pub const GENERATION_CALLERS: &[&str] = &[
    "application/pipeline/job.rs",
    "application/narration/narrator_tests.rs",
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
    // Rule 13: one generation path. Only `job.rs` calls the narrator and the
    // hooks of the Agent runner, and its step functions are the one place that
    // does, so no second route to a model call grows beside the Job. The row sees
    // files, not functions, and `job.rs` holds only Job code. A method call is
    // matched through the receiver's name (`narrator`, `agent_runner`), which
    // `JobDeps` keeps.
    CallSiteRow {
        rule: 13,
        scope: "",
        callees: GENERATION_CALLEES,
        allowed: GENERATION_CALLERS,
    },
];

// The records rule 7 bans from a Job field: the six names fit no single row field
// line, so the list is one named const the row points at.
pub const JOB_RECORD_TYPES: &[&str] = &[
    "Game",
    "Message",
    "SwipeState",
    "Tail",
    "DockRecord",
    "WorldBundle",
];

pub const TYPE_CHECK_ROWS: &[TypeCheckRow] = &[
    // Rule 7: between steps, pass ids. A Job holds no record field, so a record a
    // step loads cannot go stale in another step's hands.
    TypeCheckRow {
        rule: 7,
        scope: "application/pipeline/job.rs",
        banned_types: JOB_RECORD_TYPES,
        position: TypeCheckPosition::Fields,
    },
];

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
