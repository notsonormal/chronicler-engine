//! [DOC: docs/system/game_flow.md]
//! Debug DTOs for the HTTP `/debug/state` endpoint (T2 ticket 04 — extracted from DefaultApplicationService).

pub mod dto;

pub use dto::DebugStateView;
