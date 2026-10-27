//! The parts of the dashboard screen, as the stub-browser tests list them: the selector, the name, and its section. The guardrails check the table against `assets/index.html` and `SCREEN_PARTS`.

/// `section` is the `data-tab` of the part's section; the header and the failure
/// banner are outside every section, so their `section` is empty.
pub struct Part {
    pub selector: &'static str,
    pub name: &'static str,
    pub section: &'static str,
}

impl Part {
    /// The Game tab is the visible one when the page starts.
    pub fn starts_visible(&self) -> bool {
        self.section.is_empty() || self.section == "game"
    }
}

pub const PARTS: [Part; 13] = [
    Part {
        selector: "#header",
        name: "header",
        section: "",
    },
    Part {
        selector: "#failure-banner-degraded",
        name: "failure-banner",
        section: "",
    },
    Part {
        selector: "#story-log",
        name: "story-log",
        section: "game",
    },
    Part {
        selector: ".visual-sidebar",
        name: "visual-sidebar",
        section: "game",
    },
    Part {
        selector: "#options-dock",
        name: "options-dock",
        section: "game",
    },
    Part {
        selector: "#send-control",
        name: "send-control",
        section: "game",
    },
    Part {
        selector: "#autoplay-controls",
        name: "autoplay-controls",
        section: "game",
    },
    Part {
        selector: "#status-display",
        name: "status-display",
        section: "game",
    },
    Part {
        selector: ".settings-panel",
        name: "settings",
        section: "settings",
    },
    Part {
        selector: ".prompt-presets-panel",
        name: "prompt-presets",
        section: "prompt-presets",
    },
    Part {
        selector: ".worlds-panel",
        name: "worlds",
        section: "worlds",
    },
    Part {
        selector: ".games-panel",
        name: "games",
        section: "games",
    },
    Part {
        selector: ".llm-messages-panel",
        name: "llm-messages",
        section: "llm-messages",
    },
];
