//! htmx settle harness primitive: an `htmx:afterSettle` counter installed at page load, and a target-scoped wait for an interaction's own swap.

// htmx attaches an element's `hx-trigger` listeners at the end of the settle
// task for the swap that registered it. A test that interacts as soon as the
// element is *visible* can dispatch `change` before the listener exists, and
// the interaction is silently lost: zero POSTs, no error. Waiting for the
// registering swap to settle closes that race; scoping the wait to the
// interaction's own target stops the dashboard's self-polling regions from
// satisfying it early.
//
// playwright-rs 0.9.0 has no `wait_for_function`, so readiness is polled over
// `evaluate_value` in Rust.

use std::time::Duration;

use playwright_rs::Page;

use super::wait::wait_for_condition_async;

/// Default wait for a settle after an interaction; the settle task runs 20 ms
/// after the swap response, well inside this budget.
pub const SETTLE_TIMEOUT: Duration = Duration::from_secs(5);

/// Counter install script. Each entry records the settled element's id, tag,
/// and classes so the Rust side can name the swap; entries are capped, and only
/// the tail is read.
pub const HTMX_SETTLE_INIT_SCRIPT: &str = r#"(() => {
  const gate = { count: 0, targets: [] };
  window.__chroniclerSettle = gate;
  document.addEventListener('htmx:afterSettle', (evt) => {
    gate.count += 1;
    const elt = evt && evt.detail ? evt.detail.elt : null;
    gate.targets.push(elt ? {
      id: elt.id || '',
      tag: elt.tagName || '',
      cls: typeof elt.className === 'string' ? elt.className : '',
    } : { id: '', tag: '', cls: '' });
    if (gate.targets.length > 64) gate.targets.shift();
  });
})();"#;

/// One settled element as htmx reported it in `htmx:afterSettle`'s
/// `detail.elt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettleTarget {
    pub id: String,
    pub tag: String,
    pub classes: String,
}

impl SettleTarget {
    /// Render the target for logs: `#id` when it has one, else
    /// `tag.class1.class2`.
    pub fn describe(&self) -> String {
        if !self.id.is_empty() {
            return format!("#{}", self.id);
        }
        if self.classes.is_empty() {
            return self.tag.clone();
        }
        let cls: Vec<&str> = self.classes.split_whitespace().collect();
        format!("{}.{}", self.tag, cls.join("."))
    }

    /// True when this settle belongs to the element the caller named by
    /// simple selector: `#id`, `.class`, or a bare tag name.
    pub fn matches_selector(&self, selector: &str) -> bool {
        if let Some(id) = selector.strip_prefix('#') {
            return self.id == id;
        }
        if let Some(class) = selector.strip_prefix('.') {
            return self.classes.split_whitespace().any(|c| c == class);
        }
        self.tag.eq_ignore_ascii_case(selector)
    }
}

/// A settle-gate reading: how many swaps have settled, and which elements htmx
/// reported as settled.
#[derive(Debug, Clone)]
pub struct SettleSnapshot {
    /// Number of `htmx:afterSettle` events observed since page load.
    pub count: u64,
    /// Settle targets in event order, newest last (capped at 64).
    pub targets: Vec<SettleTarget>,
}

/// Install the htmx settle counter on `page`. Must run before navigation: the
/// `htmx:afterSettle` listener has to exist before the shell's scripts run.
pub async fn install_htmx_settle(page: &Page) {
    page.add_init_script(HTMX_SETTLE_INIT_SCRIPT)
        .await
        .expect("add_init_script for the htmx settle counter");
}

/// Read the current counter state; `None` when the counter was never installed.
async fn read_htmx_settle(page: &Page) -> Option<SettleSnapshot> {
    let raw = page
        .evaluate_value(
            "(() => { const g = window.__chroniclerSettle; \
             return g ? JSON.stringify({ count: g.count, targets: g.targets }) : ''; })()",
        )
        .await
        .ok()?;
    if raw.is_empty() {
        return None;
    }
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let targets = parsed
        .get("targets")?
        .as_array()?
        .iter()
        .map(|v| SettleTarget {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            tag: v
                .get("tag")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            classes: v
                .get("cls")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();
    Some(SettleSnapshot {
        count: parsed.get("count")?.as_u64()?,
        targets,
    })
}

/// Read the counter immediately before an interaction and return its baseline.
async fn arm_htmx_settle(page: &Page) -> u64 {
    read_htmx_settle(page)
        .await
        .expect(
            "the htmx settle counter must be installed before arming it \
             (page loaded without install_htmx_settle)",
        )
        .count
}

/// Settle targets recorded since `baseline`, oldest first.
async fn settles_since(page: &Page, baseline: u64) -> Vec<SettleTarget> {
    let snapshot = match read_htmx_settle(page).await {
        Some(s) => s,
        None => return Vec::new(),
    };
    let seen = snapshot.count.saturating_sub(baseline) as usize;
    let targets = snapshot.targets;
    let start = targets.len().saturating_sub(seen);
    targets[start..].to_vec()
}

/// Wait for a settle matching `selector` past `baseline`; `false` on timeout.
async fn await_settle_target(
    page: &Page,
    baseline: u64,
    selector: &str,
    timeout: Duration,
) -> bool {
    wait_for_condition_async(timeout, Duration::from_millis(25), || async {
        settles_since(page, baseline)
            .await
            .iter()
            .any(|t| t.matches_selector(selector))
    })
    .await
}

/// Result of a gated interaction: whether the named swap settled, every settle
/// observed after the baseline, and the subset matching the named element.
#[derive(Debug, Clone)]
pub struct SettleOutcome {
    /// The element the caller waited for, as a simple selector.
    pub expected: String,
    /// True when a settle matching `expected` landed past the baseline.
    pub settled: bool,
    /// Every settle target observed after the baseline, oldest first.
    pub all_targets: Vec<SettleTarget>,
    /// The settle targets matching `expected`, oldest first.
    pub matching_targets: Vec<SettleTarget>,
}

impl SettleOutcome {
    /// Panic when the named element never settled. A missing settle means the
    /// interaction died before htmx attached its trigger listener — not a slow
    /// machine.
    pub fn expect_settled(&self, interaction: &str) {
        let all = self
            .all_targets
            .iter()
            .map(SettleTarget::describe)
            .collect::<Vec<_>>()
            .join(", ");
        assert!(
            self.settled,
            "htmx settle: '{interaction}' never settled '{}' within {SETTLE_TIMEOUT:?} — \
             the interaction was lost before the hx-trigger listener attached; \
             interact through a settle-gated helper (click_and_settle / \
             select_option_and_settle / an open_* helper). \
             Settles observed instead: [{all}]",
            self.expected
        );
    }

    /// One line for the run log: which element was waited for, whether it
    /// settled, and what else settled meanwhile.
    pub fn describe(&self, interaction: &str) -> String {
        let all = self
            .all_targets
            .iter()
            .map(SettleTarget::describe)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "settle-gate: '{interaction}' waited for '{}' -> {}; settles since baseline: [{all}]",
            self.expected,
            if self.settled { "SETTLED" } else { "TIMED OUT" }
        )
    }
}

/// Every settle target recorded in the current document, oldest first.
async fn settle_targets_all(page: &Page) -> Vec<SettleTarget> {
    read_htmx_settle(page)
        .await
        .map(|snapshot| snapshot.targets)
        .unwrap_or_default()
}

/// Wait for a settle matching `selector` recorded anywhere in the current
/// document, not just past a baseline.
///
/// A baseline-relative wait cannot see an `hx-trigger="load"` swap: panels load
/// at page load, before any test arms a baseline. Call this at the start of a
/// test; the recorded list is capped at 64 entries, so it is not a general
/// search back through an arbitrary past.
pub async fn await_panel_ready(page: &Page, selector: &str) -> SettleOutcome {
    let matched = wait_for_condition_async(SETTLE_TIMEOUT, Duration::from_millis(25), || async {
        settle_targets_all(page)
            .await
            .iter()
            .any(|target| target.matches_selector(selector))
    })
    .await;
    // One final read, rather than re-scanning twice: the poll above cannot hand
    // its last snapshot back, so this read is the freshest settled state.
    let all_targets = settle_targets_all(page).await;
    let matching_targets: Vec<SettleTarget> = all_targets
        .iter()
        .filter(|target| target.matches_selector(selector))
        .cloned()
        .collect();
    SettleOutcome {
        expected: selector.to_string(),
        settled: matched && !matching_targets.is_empty(),
        all_targets,
        matching_targets,
    }
}

/// Click `selector`, then wait for the swap targeting `swap_target` to settle.
/// Baseline read immediately before the click.
pub async fn click_and_settle(page: &Page, selector: &str, swap_target: &str) -> SettleOutcome {
    let baseline = arm_htmx_settle(page).await;
    page.locator(selector)
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click('{selector}') failed: {e}"));
    let outcome = finish_settle(page, baseline, swap_target).await;
    eprintln!("{}", outcome.describe(&format!("click('{selector}')")));
    outcome.expect_settled(&format!("click('{selector}')"));
    outcome
}

/// Select `value` in `selector`, then wait for the swap targeting
/// `swap_target` to settle.
///
/// Waiting for the interaction's *own* target proves the round-trip completed;
/// waiting for any settle would be satisfied by a poller.
pub async fn select_option_and_settle(
    page: &Page,
    selector: &str,
    value: &str,
    swap_target: &str,
) -> SettleOutcome {
    let baseline = arm_htmx_settle(page).await;
    page.locator(selector)
        .await
        .select_option(value, None)
        .await
        .unwrap_or_else(|e| panic!("select_option('{selector}' -> '{value}') failed: {e}"));
    let outcome = finish_settle(page, baseline, swap_target).await;
    eprintln!(
        "{}",
        outcome.describe(&format!("select_option('{selector}' -> '{value}')"))
    );
    outcome.expect_settled(&format!("select_option('{selector}' -> '{value}')"));
    outcome
}

/// Read the settles past `baseline`, wait for the named target, and report the
/// outcome.
async fn finish_settle(page: &Page, baseline: u64, swap_target: &str) -> SettleOutcome {
    let polled = await_settle_target(page, baseline, swap_target, SETTLE_TIMEOUT).await;
    let all_targets = settles_since(page, baseline).await;
    let matching_targets: Vec<SettleTarget> = all_targets
        .iter()
        .filter(|t| t.matches_selector(swap_target))
        .cloned()
        .collect();
    SettleOutcome {
        expected: swap_target.to_string(),
        settled: polled || !matching_targets.is_empty(),
        all_targets,
        matching_targets,
    }
}
