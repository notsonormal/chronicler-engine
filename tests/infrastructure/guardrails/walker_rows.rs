//! The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.

use crate::walkers::{BannedNameRow, CallCountRow, CallSiteRow, TypeCheckRow};

// Row form, parsed by `scripts/generate_guardrails_doc.py` and documented in
// `docs/diataxis/reference/coding_standards/guardrails.md`: one struct literal
// per row, one `field: value,` line per field, the reason on the comments above
// the opening line. A row lands in the slice that creates the code it guards.
pub const BANNED_NAME_ROWS: &[BannedNameRow] = &[];

pub const CALL_SITE_ROWS: &[CallSiteRow] = &[];

pub const TYPE_CHECK_ROWS: &[TypeCheckRow] = &[];

pub const CALL_COUNT_ROWS: &[CallCountRow] = &[];
