// Panel chrome: the tab bar, the Settings sub-tabs, the LLM Messages expansion toggle, the swap focus restore and the world posture status clear.

// htmx restores focus only to an id that survives the swap, so a control
// replaced by a different one (Edit to form, Cancel to list) needs this
// fallback. Every panel swapped as a whole is listed, so a panel rewrite cannot
// silently drop out of the restore.
const FOCUS_SWAP_TARGETS =
  ".worlds-panel, .prompt-presets-panel, .preset-card, .settings-panel";
const FOCUSABLE = "input:not([type=hidden]), select, textarea, button, a[href]";
// A form field is the better landing spot: an Edit that opens a form should put
// the caret in its first field, not on the back link above it.
const FOCUSABLE_FIELD = "input:not([type=hidden]), select, textarea";
// A Delete removes the row its control sits in; the story log plans its own
// landing spot in dashboard-story-log.js.
const FOCUS_ROW = ".preset-card, .world-item";
const FOCUS_ROW_PANEL = ".prompt-presets-panel, .worlds-panel";
// A refused action can answer 200 with an error fragment standing in for the row,
// card or panel it replaced: that message is the landing spot, not a neighbour.
const FOCUS_ERROR = ".error-message";
// The element a swap took focus from, with the swap target whose settle may claim
// focus for it: htmx reports both events for one request, and `detail.target` is
// the resolved swap target in `htmx:beforeSwap` and stays that element in
// `htmx:afterSettle` (whose `detail.elt` is the swapped-in replacement).
let focusBeforeSwap = null;

// `focus()` is a no-op on a hidden or detached control (inside a closed
// disclosure, in a removed row), so a candidate counts only once it holds focus.
function focusFirstIn(root, selector) {
  if (!root || !root.querySelectorAll) return false;
  for (const next of root.querySelectorAll(selector)) {
    if (next.disabled) continue;
    next.focus();
    if (document.activeElement === next) return true;
  }
  return false;
}

function focusInto(root) {
  return focusFirstIn(root, FOCUSABLE_FIELD) || focusFirstIn(root, FOCUSABLE);
}

// A server error fragment carries no focus of its own (it is the same markup
// every failure surface uses), so it becomes focusable here, where it is the
// landing spot. `tabindex="-1"` takes no stop in the tab order.
function focusErrorFragment(target) {
  const error = target.matches(FOCUS_ERROR)
    ? target
    : target.querySelector(FOCUS_ERROR);
  if (!error) return false;
  if (!error.hasAttribute("tabindex")) error.setAttribute("tabindex", "-1");
  error.focus();
  return document.activeElement === error;
}

// A preset section's add form sits between its cards, so the landing spot is the
// next card in the container.
function focusAfterRemovedRow(row) {
  const cards = Array.from(row.parentElement.children).filter((child) =>
    child.matches(FOCUS_ROW),
  );
  const position = cards.indexOf(row);
  return (
    focusInto(cards[position + 1]) ||
    focusInto(cards[position - 1]) ||
    focusInto(row.closest(FOCUS_ROW_PANEL))
  );
}

// A swap that inserts a panel, a card, or the error fragment that stands in for
// one takes focus from `htmx:afterSettle`, so a row with no such replacement is
// the one whose landing spot has to be claimed while the row is still there.
function insertsFocusTarget(response) {
  const fragment = document.createElement("template");
  fragment.innerHTML = response || "";
  return !!fragment.content.querySelector(`${FOCUS_SWAP_TARGETS}, ${FOCUS_ERROR}`);
}

// A swap whose target is the row itself takes the row out; a rename or a reset
// form inside a row leaves it in place. That removal needs focus to move before
// the swap: a `swap:0.3s` delete removes the row after htmx:afterRequest has
// already been handled, and a swap that inserts nothing fires no afterSettle.
//
// Only a swap that can take the focused control's row or panel away records a
// landing spot: a poll's swap target is neither, so its 2–5 s cycle cannot
// overwrite the element a delete saved while the delete's settle delay runs.
document.body.addEventListener("htmx:beforeSwap", function (evt) {
  const target = evt.detail.elt;
  if (!target || !target.matches || !evt.detail.shouldSwap) return;
  if (!target.matches(FOCUS_SWAP_TARGETS) && !target.closest(FOCUS_ROW)) return;
  const element = document.activeElement;
  focusBeforeSwap = { element, target };
  const row = element && element.closest && element.closest(FOCUS_ROW);
  if (!row || target !== row) return;
  if (insertsFocusTarget(evt.detail.serverResponse)) return;
  focusAfterRemovedRow(row);
});

document.body.addEventListener("htmx:afterSettle", function (evt) {
  const saved = focusBeforeSwap;
  // A poll can settle inside a `swap:0.3s` delete's delay, and that settle
  // carries its own target: only the swap that saved the element may claim or
  // clear it, or the delete's landing spot is lost and focus falls to `body`.
  // Scenario 29.5 dispatches that poll settle on purpose.
  if (!saved || !evt.detail || evt.detail.target !== saved.target) return;
  focusBeforeSwap = null;
  const element = saved.element;
  if (!element || element === document.body || document.contains(element)) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  const target = evt.detail.elt;
  if (!target || !target.matches) return;
  if (target.matches(FOCUS_SWAP_TARGETS)) {
    focusInto(target);
    return;
  }
  // The row the control sat in came back as an error fragment: focus the message
  // there, so the refusal is read where the control was, not on a neighbour. Only
  // a row or an error fragment may take focus this way, so an unrelated poll's
  // fragment cannot.
  if (target.matches(FOCUS_ROW) || target.matches(FOCUS_ERROR)) {
    focusErrorFragment(target);
  }
});

// A failed Update World must not sit under green feedback from an earlier auto-save.
function clearPostureStatus() {
  const status = document.getElementById("world-posture-status");
  if (status) status.innerHTML = "";
}

// Tab state is client-only, so the active tab is remembered for a reload of the
// page.
const ACTIVE_TAB_KEY = "chronicle.activeTab";

function activateTab(btn) {
  document.querySelectorAll(".tab").forEach((b) => {
    b.classList.remove("active");
    b.setAttribute("aria-selected", b === btn ? "true" : "false");
  });
  document
    .querySelectorAll(".tab-content")
    .forEach((c) => c.classList.remove("active"));
  btn.classList.add("active");
  const panel = document.getElementById(btn.dataset.tab + "-tab");
  if (panel) {
    panel.classList.add("active");
    liveSectionOpened(panel);
  }
}

function rememberActiveTab(tab) {
  try {
    sessionStorage.setItem(ACTIVE_TAB_KEY, tab);
  } catch (e) {
    // Without storage the active tab lasts for the page only.
  }
}

function restoreActiveTab() {
  let saved = null;
  try {
    saved = sessionStorage.getItem(ACTIVE_TAB_KEY);
  } catch (e) {
    return;
  }
  if (!saved) return;
  const btn = document.querySelector('.tab[data-tab="' + saved + '"]');
  if (btn) activateTab(btn);
}

document.addEventListener("DOMContentLoaded", function () {

  restoreActiveTab();

  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      activateTab(btn);
      rememberActiveTab(btn.dataset.tab);
    });
  });

  // The Settings panel is swapped as a whole, so the sub-tab buttons cannot be bound per button.
  document.addEventListener("click", (event) => {
    const subtab = event.target.closest(".settings-subtab");
    if (!subtab) return;
    const panel = subtab.closest(".settings-panel");
    if (!panel) return;
    panel.querySelectorAll(".settings-subtab").forEach((b) => {
      const selected = b === subtab;
      b.classList.toggle("active", selected);
      b.setAttribute("aria-selected", selected ? "true" : "false");
    });
    panel
      .querySelectorAll(".settings-subtab-panel")
      .forEach((p) => p.classList.remove("active"));
    const target = panel.querySelector(
      "#" + subtab.getAttribute("aria-controls"),
    );
    if (target) target.classList.add("active");
  });

  // The header holds the open state in aria-expanded and the body in hidden. A
  // load keeps both (data-keep), so a new LLM message never closes a row.
  window.toggleLlmMessage = function (header) {
    const body = document.getElementById(header.getAttribute("aria-controls"));
    if (!body) return;
    const expanded = header.getAttribute("aria-expanded") !== "true";
    header.setAttribute("aria-expanded", String(expanded));
    body.hidden = !expanded;
  };
});
