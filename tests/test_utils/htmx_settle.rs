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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettleTarget {
    pub id: String,
    pub tag: String,
    pub classes: String,
}

impl SettleTarget {
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

#[derive(Debug, Clone)]
pub struct SettleSnapshot {
    pub count: u64,
    /// Newest last, capped at 64.
    pub targets: Vec<SettleTarget>,
}

/// Install the htmx settle counter on `page`. Must run before navigation: the
/// `htmx:afterSettle` listener has to exist before the shell's scripts run.
pub async fn install_htmx_settle(page: &Page) {
    page.add_init_script(HTMX_SETTLE_INIT_SCRIPT)
        .await
        .expect("add_init_script for the htmx settle counter");
}

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

async fn arm_htmx_settle(page: &Page) -> u64 {
    read_htmx_settle(page)
        .await
        .expect(
            "the htmx settle counter must be installed before arming it \
             (page loaded without install_htmx_settle)",
        )
        .count
}

async fn settle_targets(page: &Page, scope: &SettleScope) -> Option<Vec<SettleTarget>> {
    let snapshot = read_htmx_settle(page).await?;
    let targets = match scope {
        SettleScope::WholeDocument => snapshot.targets,
        SettleScope::Since(baseline) => {
            let seen = snapshot.count.saturating_sub(*baseline) as usize;
            let start = snapshot.targets.len().saturating_sub(seen);
            snapshot.targets[start..].to_vec()
        }
    };
    Some(targets)
}

/// Result of one scoped poll: the last read of `scope`, and whether that read
/// (or any earlier one) matched the waited-for selector.
struct PollResult {
    matched: bool,
    targets: Vec<SettleTarget>,
}

/// Poll `scope` every 25 ms for a settle matching `selector`, keeping the
/// last scoped read so no extra final read is needed on match or timeout.
async fn poll_settles(
    page: &Page,
    scope: &SettleScope,
    selector: &str,
    timeout: Duration,
) -> PollResult {
    let deadline = std::time::Instant::now() + timeout;
    let mut targets: Vec<SettleTarget> = Vec::new();
    let mut matched = false;
    loop {
        if let Some(read) = settle_targets(page, scope).await {
            targets = read;
            matched = matched || targets.iter().any(|t| t.matches_selector(selector));
        }
        if matched || std::time::Instant::now() >= deadline {
            return PollResult { matched, targets };
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[derive(Debug, Clone)]
pub struct SettleOutcome {
    pub expected: String,
    pub settled: bool,
    pub all_targets: Vec<SettleTarget>,
    pub matching_targets: Vec<SettleTarget>,
}

impl SettleOutcome {
    /// Assemble the outcome from one scoped poll. `matched` is kept even when
    /// the final read's matching subset is empty: a matching target the poll
    /// saw can fall out of the 64-entry cap window before this re-read, so
    /// dropping `matched` would report a real settle as a timeout.
    fn from_poll(expected: &str, result: PollResult) -> SettleOutcome {
        let matching_targets: Vec<SettleTarget> = result
            .targets
            .iter()
            .filter(|t| t.matches_selector(expected))
            .cloned()
            .collect();
        SettleOutcome {
            expected: expected.to_string(),
            settled: result.matched || !matching_targets.is_empty(),
            all_targets: result.targets,
            matching_targets,
        }
    }

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

    pub fn log_line(&self, interaction: &str) -> String {
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

/// Which recorded settles a wait considers: every settle in the current
/// document, or only those past a baseline read.
enum SettleScope {
    /// Every settle target recorded in the current document. A baseline-relative
    /// wait cannot see an `hx-trigger="load"` swap: panels load at page load,
    /// before any test arms a baseline. A `WholeDocument` wait runs at the
    /// start of a test; the recorded list is capped at 64 entries, so it is not
    /// a general search back through an arbitrary past.
    WholeDocument,
    /// Only settles recorded after this baseline count.
    Since(u64),
}

pub async fn await_panel_ready(page: &Page, selector: &str) -> SettleOutcome {
    SettleOutcome::from_poll(
        selector,
        poll_settles(page, &SettleScope::WholeDocument, selector, SETTLE_TIMEOUT).await,
    )
}

pub async fn click_and_settle(page: &Page, selector: &str, swap_target: &str) -> SettleOutcome {
    let baseline = arm_htmx_settle(page).await;
    page.locator(selector)
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click('{selector}') failed: {e}"));
    let outcome = finish_settle(page, baseline, swap_target).await;
    eprintln!("{}", outcome.log_line(&format!("click('{selector}')")));
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
        outcome.log_line(&format!("select_option('{selector}' -> '{value}')"))
    );
    outcome.expect_settled(&format!("select_option('{selector}' -> '{value}')"));
    outcome
}

async fn finish_settle(page: &Page, baseline: u64, swap_target: &str) -> SettleOutcome {
    SettleOutcome::from_poll(
        swap_target,
        poll_settles(
            page,
            &SettleScope::Since(baseline),
            swap_target,
            SETTLE_TIMEOUT,
        )
        .await,
    )
}
