// Panel chrome: the tab bar, the Settings sub-tabs, the LLM Messages expansion state, the swap focus restore and the world posture status clear.

// htmx restores focus only to an id that survives the swap, so a control
// replaced by a different one (Edit to form, Cancel to list) needs this fallback.
const FOCUS_SWAP_TARGETS = ".worlds-panel, .prompt-presets-panel, .preset-card";
const FOCUSABLE = "input:not([type=hidden]), select, textarea, button, a[href]";
let focusedBeforeSwap = null;

document.body.addEventListener("htmx:beforeSwap", function () {
  focusedBeforeSwap = document.activeElement;
});

document.body.addEventListener("htmx:afterSettle", function (evt) {
  const saved = focusedBeforeSwap;
  focusedBeforeSwap = null;
  if (!saved || saved === document.body || document.contains(saved)) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  const target = evt.detail.elt;
  if (!target || !target.matches || !target.matches(FOCUS_SWAP_TARGETS)) return;
  for (const next of target.querySelectorAll(FOCUSABLE)) {
    if (!next.disabled) {
      next.focus();
      return;
    }
  }
});

// A failed Update World must not sit under green feedback from an earlier auto-save.
function clearPostureStatus() {
  const status = document.getElementById("world-posture-status");
  if (status) status.innerHTML = "";
}

document.addEventListener("DOMContentLoaded", function () {

  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".tab").forEach((b) => {
        b.classList.remove("active");
        b.setAttribute("aria-selected", b === btn ? "true" : "false");
      });
      document
        .querySelectorAll(".tab-content")
        .forEach((c) => c.classList.remove("active"));
      btn.classList.add("active");
      document
        .getElementById(btn.dataset.tab + "-tab")
        .classList.add("active");
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
