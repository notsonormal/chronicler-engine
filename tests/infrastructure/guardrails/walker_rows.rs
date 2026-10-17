//! The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.

use crate::walkers::{BannedName, BannedNameRow, CallCountRow, CallSiteRow, TypeCheckRow};

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
        banned: &[BannedName::StaticItem, BannedName::NamePrefix("Atomic")],
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
];

pub const TYPE_CHECK_ROWS: &[TypeCheckRow] = &[];

pub const CALL_COUNT_ROWS: &[CallCountRow] = &[];
