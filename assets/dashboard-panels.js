// Panel chrome: the tab bar, the Settings sub-tabs, the LLM Messages expansion toggle, the focus after a preset card swap, the editor focus and the world posture status clear.

// htmx restores focus only to an id that survives the swap, so a control
// replaced by a different one (Edit to form, Cancel to card) needs this
// fallback. A panel is a live region: its load keeps what it does not change, and
// the focus rule of dashboard-live.js moves focus when a load removes the focused
// node. A preset card is the one element that a view change replaces: its
// Edit, View and Cancel swap the card that holds the control.
const PRESET_CARD = ".preset-card";
const FOCUSABLE = "input:not([type=hidden]), select, textarea, button, a[href]";
// A form field is the better landing spot: an Edit that opens a form should put
// the caret in its first field, not on the back link above it.
const FOCUSABLE_FIELD = "input:not([type=hidden]), select, textarea";
// A refused Edit or View answers 200 with an error fragment standing in for the
// card it replaced: that message is the landing spot.
const FOCUS_ERROR = ".error-message";
// The element a card swap took focus from, with the card that was the swap
// target: htmx reports both events for one request, and `detail.target` is the
// resolved swap target in `htmx:beforeSwap` and stays that element in
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

document.body.addEventListener("htmx:beforeSwap", function (evt) {
  const target = evt.detail.elt;
  if (!target || !target.matches || !evt.detail.shouldSwap) return;
  if (!target.matches(PRESET_CARD)) return;
  focusBeforeSwap = { element: document.activeElement, target };
});

document.body.addEventListener("htmx:afterSettle", function (evt) {
  const saved = focusBeforeSwap;
  // A region load can settle between the card swap and its own settle, and that
  // settle carries its own target: only the swap that saved the element may
  // claim or clear it.
  if (!saved || !evt.detail || evt.detail.target !== saved.target) return;
  focusBeforeSwap = null;
  const element = saved.element;
  if (!element || element === document.body || document.contains(element)) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  const target = evt.detail.elt;
  if (!target || !target.matches) return;
  if (target.matches(PRESET_CARD)) {
    focusInto(target);
  } else if (target.matches(FOCUS_ERROR)) {
    focusErrorFragment(target);
  }
});

// An editor opens into its slot, where the control that opened it is hidden, so
// focus goes to the first field of the editor.
document.body.addEventListener("htmx:afterSettle", function (evt) {
  const slot = evt.detail && evt.detail.target;
  if (!slot || !slot.classList || !slot.classList.contains("editor-slot")) return;
  if (slot.firstElementChild) focusInto(slot);
});

// A failed Update World must not sit under green feedback from an earlier auto-save.
function clearPostureStatus() {
  const status = document.getElementById("world-posture-status");
  if (status) status.removeAttribute("data-saved");
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
    // A section that opens shows its regions: each one that is due loads.
    askNow();
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

  // The selected sub-tab is view state in `aria-selected` (buttons) and `hidden`
  // (panels). Both are in `data-keep`, so a load of the Settings panel keeps them.
  document.addEventListener("click", (event) => {
    const subtab = event.target.closest(".settings-subtab");
    if (!subtab) return;
    const panel = subtab.closest(".settings-panel");
    if (!panel) return;
    panel.querySelectorAll(".settings-subtab").forEach((b) => {
      b.setAttribute("aria-selected", b === subtab ? "true" : "false");
    });
    const target = subtab.getAttribute("aria-controls");
    panel.querySelectorAll(".settings-subtab-panel").forEach((p) => {
      p.hidden = p.id !== target;
    });
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
