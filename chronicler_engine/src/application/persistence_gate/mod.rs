//! [DOC: docs/system/game_flow.md]
//! PersistenceGate — game-storage seam + persistence helpers
//! (T2 ticket 02 — façade-first carve-out from DefaultApplicationService).

pub mod dto;
pub mod gate;

pub use dto::WorldSnapshot;
pub use gate::PersistenceGate;
