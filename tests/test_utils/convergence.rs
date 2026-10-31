//! The convergence check: once the page is quiet, each visible part equals a fresh `GET /screen` of that part, apart from the story-log entry in edit. The parts of a closed tab are empty. Each part element has the attributes of the shell. The stub and the full-stack browser tests run it at the end of each check.

use std::time::Duration;

use serde::Deserialize;

const CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(10);

const CONVERGENCE_SCRIPT: &str = include_str!("convergence.js");

#[derive(Deserialize)]
struct PartMismatch {
    part: String,
    route: String,
    diff: String,
}

#[derive(Deserialize)]
struct ConvergenceReport {
    ok: bool,
    #[serde(rename = "notQuiet")]
    not_quiet: Vec<String>,
    blocking: Vec<PartMismatch>,
}

/// Waits until the page is quiet, then fails with the part, the route and a
/// short diff for each visible part that differs from a fresh server render.
/// A part that differs is given until the timeout to load again, so a write
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
            mismatch.part, mismatch.route, mismatch.diff
        ));
    }
    panic!("{failure}");
}
