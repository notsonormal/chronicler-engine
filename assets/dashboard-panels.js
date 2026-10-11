// Panel chrome: the tab bar, the Settings sub-tabs, the LLM Messages expansion state, the swap focus restore and the world posture status clear.

// htmx restores focus only to an id that survives the swap, so a control
// replaced by a different one (Edit to form, Cancel to list) needs this
// fallback. Every panel swapped as a whole is listed, so a panel rewrite cannot
// silently drop out of the restore.
const FOCUS_SWAP_TARGETS =
  ".worlds-panel, .prompt-presets-panel, .preset-card, .settings-panel, .games-panel";
const FOCUSABLE = "input:not([type=hidden]), select, textarea, button, a[href]";
// A form field is the better landing spot: an Edit that opens a form should put
// the caret in its first field, not on the back link above it.
const FOCUSABLE_FIELD = "input:not([type=hidden]), select, textarea";
// The saved-game Delete removes the row its control sits in, so there is
// nothing left inside the swap target to land on. (The story log's own delete
// plans its landing spot in dashboard-story-log.js.)
const FOCUS_ROW = ".game-item";
let focusedBeforeSwap = null;
let removedRow = null;

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

document.body.addEventListener("htmx:beforeSwap", function (evt) {
  focusedBeforeSwap = document.activeElement;
  removedRow = null;
  const target = evt.detail.elt;
  const row =
    focusedBeforeSwap &&
    focusedBeforeSwap.closest &&
    focusedBeforeSwap.closest(FOCUS_ROW);
  if (!row || !target || !target.contains || !target.contains(focusedBeforeSwap)) {
    return;
  }
  removedRow = {
    focused: focusedBeforeSwap,
    next: row.nextElementSibling,
    previous: row.previousElementSibling,
  };
});

document.body.addEventListener("htmx:afterSettle", function (evt) {
  const saved = focusedBeforeSwap;
  focusedBeforeSwap = null;
  if (!saved || saved === document.body || document.contains(saved)) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  const target = evt.detail.elt;
  if (!target || !target.matches || !target.matches(FOCUS_SWAP_TARGETS)) return;
  focusInto(target);
});

// A removed row fires no afterSettle at all when it was the last child, so its
// landing spot is applied when the request settles instead: the row that takes
// its place, else the row before it, else the panel. htmx re-fires afterRequest
// on the nearest surviving ancestor of a request element the swap removed,
// which is what reaches this listener.
document.body.addEventListener("htmx:afterRequest", function () {
  const removed = removedRow;
  removedRow = null;
  if (!removed || document.contains(removed.focused)) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  focusInto(removed.next) ||
    focusInto(removed.previous) ||
    focusInto(document.querySelector(".games-panel"));
});

// A failed Update World must not sit under green feedback from an earlier auto-save.
function clearPostureStatus() {
  const status = document.getElementById("world-posture-status");
  if (status) status.innerHTML = "";
}

// Tab state is client-only, and several actions reload the page (a rename, a
// game switch, a new game), so the active tab is remembered for the reload.
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
  if (panel) panel.classList.add("active");
}

function rememberActiveTab(tab) {
  try {
    sessionStorage.setItem(ACTIVE_TAB_KEY, tab);
  } catch (e) {
    // Without storage the tab only lasts for the page, which is the old behaviour.
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

  // htmx polls re-render the panel, so expansion state lives in JS.
  const expandedLlmMessages = new Set();
  let llmOriginalTrigger = null;

  function pauseLlmPolling() {
    const panel = document.querySelector(".llm-messages-panel");
    if (panel) {
      llmOriginalTrigger = panel.getAttribute("hx-trigger");
      panel.setAttribute("hx-trigger", "none");
      htmx.process(panel);
    }
  }

  function resumeLlmPolling() {
    const panel = document.querySelector(".llm-messages-panel");
    if (panel && llmOriginalTrigger) {
      panel.setAttribute("hx-trigger", llmOriginalTrigger);
      htmx.process(panel);
      llmOriginalTrigger = null;
    }
  }

  window.toggleLlmMessage = function (header) {
    const card = header.closest(".llm-message-card");
    if (card) {
      const expanded = card.classList.toggle("expanded");
      header.setAttribute("aria-expanded", String(expanded));
      const msgId = card.id;
      if (msgId) {
        if (expanded) {
          expandedLlmMessages.add(msgId);
          pauseLlmPolling();
        } else {
          expandedLlmMessages.delete(msgId);
          if (expandedLlmMessages.size === 0) {
            resumeLlmPolling();
          }
        }
      }
    }
  };

  window.restoreLlmExpandedState = function () {
    expandedLlmMessages.forEach(function (msgId) {
      const card = document.getElementById(msgId);
      if (card) {
        card.classList.add("expanded");
        const header = card.querySelector(".llm-message-header");
        if (header) header.setAttribute("aria-expanded", "true");
      } else {
        expandedLlmMessages.delete(msgId);
      }
    });
  };


});
