//! The convergence check: once the page is quiet, each visible live region equals a fresh GET of its own route, apart from the story-log entry in edit. The stub and the full-stack browser tests run it at the end of each check.

use std::time::Duration;

use serde::Deserialize;

const CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(10);

const CONVERGENCE_SCRIPT: &str = include_str!("convergence.js");

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
}

/// Waits until the page is quiet, then fails with the region, the route and a
/// short diff for each visible region that differs from a fresh server render.
/// A region that differs is given until the timeout to load again, so a write
/// that lands during the check does not fail it.
pub async fn assert_screen_converged(page: &playwright_rs::Page) {
    let arguments = serde_json::json!({
        "timeoutMs": CONVERGENCE_TIMEOUT.as_millis() as u64,
    });
    let answer = page
        .evaluate::<serde_json::Value, String>(CONVERGENCE_SCRIPT, Some(&arguments))
        .await
        .expect("the convergence check runs in the page");
    let report: ConvergenceReport =
        serde_json::from_str(&answer).expect("the convergence check answers its report");

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
