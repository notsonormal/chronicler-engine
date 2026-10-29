//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Budget of an Autoplay run: the number of cycles it plays

use crate::application::generation::gate::NotAccepted;

/// 1 to [`Budget::MAX`] cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget(u32);

impl Budget {
    pub const DEFAULT: Budget = Budget(10);
    pub const MAX: u32 = 100;

    pub fn parse(cycles: &str) -> Result<Self, NotAccepted> {
        let cycles = cycles.trim();
        if cycles.is_empty() {
            return Ok(Self::DEFAULT);
        }
        let cycles: u32 = cycles.parse().map_err(|_| {
            NotAccepted::BadRequest("The budget must be a whole number of cycles".to_string())
        })?;
        if cycles == 0 || cycles > Self::MAX {
            return Err(NotAccepted::BadRequest(format!(
                "An autoplay run needs a budget of 1 to {} cycles",
                Self::MAX
            )));
        }
        Ok(Self(cycles))
    }

    pub fn cycles(self) -> u32 {
        self.0
    }
}
