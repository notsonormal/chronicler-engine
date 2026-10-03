//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Application-layer utility helpers.

pub(crate) mod name_uniqueness;
pub(crate) use name_uniqueness::name_is_available;
