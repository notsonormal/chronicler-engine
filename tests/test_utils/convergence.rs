//! The convergence check: once the page is quiet, each visible live region equals a fresh GET of its own route. The stub and the full-stack browser tests run it at the end of each check.

use std::time::Duration;

use serde::Deserialize;

const CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(10);

const CONVERGENCE_SCRIPT: &str = include_str!("convergence.js");

/// What a row of the allow list lets a region do.
pub enum Tolerates {
    /// The region holds an open edit (the hold rule of `assets/dashboard-live.js`):
    /// a load does not replace it, so it does not converge. The selector names the
    /// editors. The region is tolerated only when every `[data-work]` element and
    /// every dirty field of the region is inside an element that matches it.
    HeldByOpenEdit(&'static str),
    /// The region differs from the server render, and the diff contains this text.
    DiffContaining(&'static str),
}

/// A region the check lets differ from the server, because a later stage of
/// `docs/plans/dashboard-visible-tab-refresh.md` removes the cause. The list only
/// shrinks. A tolerated region is printed, not failed.
pub struct AllowedMismatch {
    pub route: &'static str,
    pub tolerates: Tolerates,
    pub reason: &'static str,
    pub removed_in_stage: u8,
}

pub const ALLOWED_MISMATCHES: &[AllowedMismatch] = &[
    AllowedMismatch {
        route: "/fragment/story-log",
        tolerates: Tolerates::HeldByOpenEdit(".log-entry[data-work]"),
        reason: "an entry in edit (`data-work`) holds the whole story log, so it keeps the edit form that the server does not render",
        removed_in_stage: 3,
    },
    AllowedMismatch {
        route: "/fragment/prompt-presets",
        tolerates: Tolerates::HeldByOpenEdit(".preset-card[data-work], .preset-add"),
        reason: "an open preset editor or an Add form with typed text holds the Prompt Presets panel, so it keeps the form that the server does not render",
        removed_in_stage: 3,
    },
    AllowedMismatch {
        route: "/fragment/games",
        tolerates: Tolerates::HeldByOpenEdit(".game-rename, .new-game-add"),
        reason: "an open Rename or New Game form holds the Games panel, so it keeps the form text that the server does not render",
        removed_in_stage: 3,
    },
    AllowedMismatch {
        route: "/fragment/worlds",
        tolerates: Tolerates::HeldByOpenEdit("#world-editor"),
        reason: "an open World editor (`#world-editor`) holds the Worlds panel, so it keeps the form that the server does not render",
        removed_in_stage: 3,
    },
    AllowedMismatch {
        route: "/fragment/prompt-presets",
        tolerates: Tolerates::DiffContaining(
            "attribute class \"error-message\" vs \"preset-card\"",
        ),
        reason: "a refused card View or Edit swaps its error in place of the card (`hx-target=\"closest .preset-card\"`), and no load follows because no number moved",
        removed_in_stage: 3,
    },
];

#[derive(Deserialize)]
struct RegionMismatch {
    region: String,
    route: String,
    diff: String,
}

#[derive(Deserialize)]
struct ConvergenceReport {
    ok: bool,
    #[serde(rename = "notQuiet")]
    not_quiet: Vec<String>,
    blocking: Vec<RegionMismatch>,
    tolerated: Vec<RegionMismatch>,
}

/// Waits until the page is quiet, then fails with the region, the route and a
/// short diff for each visible region that differs from a fresh server render.
/// A region that differs is given until the timeout to load again, so a write
/// that lands during the check does not fail it.
pub async fn assert_screen_converged(page: &playwright_rs::Page) {
    let held_editors: Vec<serde_json::Value> = ALLOWED_MISMATCHES
        .iter()
        .filter_map(|row| match row.tolerates {
            Tolerates::HeldByOpenEdit(editors) => {
                Some(serde_json::json!({ "route": row.route, "editors": editors }))
            }
            Tolerates::DiffContaining(_) => None,
        })
        .collect();
    let tolerable_diffs: Vec<serde_json::Value> = ALLOWED_MISMATCHES
        .iter()
        .filter_map(|row| match row.tolerates {
            Tolerates::DiffContaining(contains) => {
                Some(serde_json::json!({ "route": row.route, "contains": contains }))
            }
            Tolerates::HeldByOpenEdit(_) => None,
        })
        .collect();
    let arguments = serde_json::json!({
        "timeoutMs": CONVERGENCE_TIMEOUT.as_millis() as u64,
        "heldEditors": held_editors,
        "tolerableDiffs": tolerable_diffs,
    });
    let answer = page
        .evaluate::<serde_json::Value, String>(CONVERGENCE_SCRIPT, Some(&arguments))
        .await
        .expect("the convergence check runs in the page");
    let report: ConvergenceReport =
        serde_json::from_str(&answer).expect("the convergence check answers its report");

    for mismatch in &report.tolerated {
        eprintln!(
            "convergence: tolerated mismatch in {} ({}): {}",
            mismatch.region, mismatch.route, mismatch.diff
        );
    }
    if report.ok {
        return;
    }
    let mut failure = format!("the screen did not converge within {CONVERGENCE_TIMEOUT:?}:");
    for reason in &report.not_quiet {
        failure.push_str(&format!("\n  not quiet: {reason}"));
    }
    for mismatch in &report.blocking {
        failure.push_str(&format!(
            "\n  {} differs from a fresh GET {}: {}",
            mismatch.region, mismatch.route, mismatch.diff
        ));
    }
    panic!("{failure}");
}
