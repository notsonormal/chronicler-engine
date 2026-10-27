//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Screen route handlers.

mod parts;
mod screen;

pub use self::parts::{screen_part_names, PartRenderer, ScreenView, SCREEN_PARTS};
pub use self::screen::{screen_handler, ScreenQuery};
