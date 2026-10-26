//! The live regions of the dashboard shell, as the stub-browser tests list them: the selector, the fragment path, the kinds of change that load it, and its section. The guardrails check the table against `assets/index.html`.

use chronicler_engine::domain::model::change::ChangeKind;

/// `section` is the `data-tab` of the region's section; the header is outside every
/// section, so its `section` is empty.
pub struct Region {
    pub selector: &'static str,
    pub path: &'static str,
    pub kinds: &'static [ChangeKind],
    pub section: &'static str,
}

impl Region {
    pub fn starts_visible(&self) -> bool {
        self.section.is_empty() || self.section == "game"
    }
}

pub const REGIONS: [Region; 11] = [
    Region {
        selector: "#header",
        path: "/fragment/header",
        kinds: &[ChangeKind::GAMES],
        section: "",
    },
    Region {
        selector: "#story-log",
        path: "/fragment/story-log",
        kinds: &[
            ChangeKind::STORY,
            ChangeKind::BUSY_STATE,
            ChangeKind::WORLDS,
        ],
        section: "game",
    },
    Region {
        selector: ".visual-sidebar",
        path: "/fragment/visual-sidebar",
        kinds: &[ChangeKind::STORY, ChangeKind::WORLDS],
        section: "game",
    },
    Region {
        selector: "#options-dock",
        path: "/fragment/options-dock",
        kinds: &[ChangeKind::OPTIONS, ChangeKind::BUSY_STATE],
        section: "game",
    },
    Region {
        selector: "#status-display",
        path: "/status/busy",
        kinds: &[ChangeKind::BUSY_STATE],
        section: "game",
    },
    Region {
        selector: ".games-panel",
        path: "/fragment/games",
        kinds: &[
            ChangeKind::GAMES,
            ChangeKind::WORLDS,
            ChangeKind::PROMPT_PRESETS,
        ],
        section: "games",
    },
    Region {
        selector: ".llm-messages-panel",
        path: "/fragment/llm-messages",
        kinds: &[ChangeKind::LLM_MESSAGES],
        section: "llm-messages",
    },
    Region {
        selector: "#failure-banner-degraded",
        path: "/fragment/failure-banner",
        kinds: &[ChangeKind::SETTINGS, ChangeKind::LLM_MESSAGES],
        section: "",
    },
    Region {
        selector: ".settings-panel",
        path: "/fragment/settings",
        kinds: &[ChangeKind::SETTINGS, ChangeKind::LLM_MESSAGES],
        section: "settings",
    },
    Region {
        selector: ".prompt-presets-panel",
        path: "/fragment/prompt-presets",
        kinds: &[
            ChangeKind::PROMPT_PRESETS,
            ChangeKind::SETTINGS,
            ChangeKind::GAMES,
        ],
        section: "prompt-presets",
    },
    Region {
        selector: ".worlds-panel",
        path: "/fragment/worlds",
        kinds: &[ChangeKind::WORLDS, ChangeKind::GAMES],
        section: "worlds",
    },
];
