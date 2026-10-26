// The action area: the command form, its Checking/Generating state machine, the reading of the status display and the live-region announcements.

// The DOM carries every action-area state except an in-flight check and a send
// that the status display has not shown yet.
let checkInFlight = false;

// The action area's own state between an accepted send and the first load of the
// status display that follows it. The status display is a live region: the
// action area reads it and never writes it. This state shows in #action-status.
let sendPending = false;
let pendingLoadStarted = false;

function renderActionStatus() {
  const own = document.getElementById("action-status");
  if (!own) return;
  own.innerHTML = sendPending
    ? '<span class="status thinking">Thinking...</span>'
    : "";
}

function beginPending() {
  sendPending = true;
  pendingLoadStarted = false;
  renderActionStatus();
}

function endPending() {
  sendPending = false;
  pendingLoadStarted = false;
  renderActionStatus();
  syncActionState();
}

function statusShows(state) {
  const statusDisplay = document.getElementById("status-display");
  return !!(
    statusDisplay && statusDisplay.querySelector('[data-status="' + state + '"]')
  );
}

function statusIsGenerating() {
  return statusShows("thinking");
}

function statusIsAutoplay() {
  return statusShows("autoplay");
}

function actionIsGenerating() {
  return sendPending || statusIsGenerating();
}

function previewIsOpen() {
  const preview = document.getElementById("action-preview");
  return !!(preview && preview.firstElementChild !== null);
}

function deriveActionState() {
  if (checkInFlight) return "checking";
  if (statusIsAutoplay()) return "autoplay";
  if (actionIsGenerating()) return "generating";
  if (previewIsOpen()) return "preview";
  return "idle";
}

// The CSS swaps the command form for Stop off `data-state`.
function applyAutoplayControls(state) {
  const area = document.getElementById("action-area");
  if (area) area.dataset.state = state;
  const startable = state === "idle";
  ["autoplay-btn", "autoplay-cycles"].forEach((id) => {
    const control = document.getElementById(id);
    if (control) control.disabled = !startable;
  });
}

function applyActionState(state) {
  applyAutoplayControls(state);
  const btn = document.getElementById("submit-btn");
  const input = document.querySelector('#command-form input[name="command"]');
  if (btn) {
    if (state === "generating") {
      btn.innerHTML = '<svg class="icon icon-spin" aria-hidden="true"><use href="#i-loader-circle"></use></svg> Generating…';
      btn.disabled = true;
    } else {
      btn.innerHTML = '<svg class="icon" aria-hidden="true"><use href="#i-send"></use></svg> Send';
      btn.disabled = state === "checking";
    }
  }
  if (input) {
    input.disabled = state === "checking";
  }
}

function focusCorrectedTextarea() {
  const textarea = document.getElementById("corrected-textarea");
  if (textarea) textarea.focus();
}

function focusCommandInput() {
  const input = commandInput();
  if (input) input.focus();
}

let previewWasOpen = false;

function syncActionState() {
  applyActionState(deriveActionState());
  const open = previewIsOpen();
  if (open && !previewWasOpen) {
    focusCorrectedTextarea();
  } else if (!open && previewWasOpen) {
    focusCommandInput();
  }
  previewWasOpen = open;
}

// The container's own class colours whatever it holds, and the failure line is a
// server-rendered disclosure, so the state is read back off the answer's
// `data-status` instead of being remembered. Only the state classes are
// touched, so htmx's own transient classes survive.
const STATUS_STATE_CLASSES = ["ready", "thinking", "error"];

const STATUS_CLASS_OF_STATE = {
  idle: "ready",
  thinking: "thinking",
  autoplay: "thinking",
  failed: "error",
};

function statusDisplayState(statusDisplay) {
  const answer = statusDisplay.querySelector("[data-status]");
  const state = answer ? answer.dataset.status : "idle";
  return STATUS_CLASS_OF_STATE[state] || "ready";
}

function syncStatusClass() {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay) return;
  statusDisplay.classList.remove(...STATUS_STATE_CLASSES);
  statusDisplay.classList.add(statusDisplayState(statusDisplay));
}

function writeLiveRegion(id, text) {
  const region = document.getElementById(id);
  if (!region || region.textContent === text) return;
  region.textContent = text;
}

let announcedStatusLabel = null;

function announceStatusDisplay() {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay || statusDisplay.querySelector(".error-disclosure")) {
    return;
  }
  const label = statusDisplay.textContent.trim();
  if (label === "") return;
  if (announcedStatusLabel === null) {
    announcedStatusLabel = label;
    return;
  }
  if (label === announcedStatusLabel) return;
  announcedStatusLabel = label;
  writeLiveRegion("status-announcer", label);
}

function announceGenerationError(statusDisplay) {
  const message = statusDisplay.querySelector(".error-disclosure-message");
  writeLiveRegion(
    "status-error-announcer",
    message ? message.textContent.trim() : "",
  );
}

function applyStatusDisplay() {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay) return;
  syncStatusClass();
  syncActionState();
  announceGenerationError(statusDisplay);
  announceStatusDisplay();
}

let announcedEntryIds = null;

function announceNewNarration() {
  const log = document.getElementById("story-log");
  if (!log) return;
  const entries = Array.from(log.querySelectorAll(".log-entry"));
  const ids = entries.map((entry) => entry.dataset.id);
  if (announcedEntryIds === null) {
    announcedEntryIds = new Set(ids);
    return;
  }
  const added = entries.filter(
    (entry) => !announcedEntryIds.has(entry.dataset.id),
  );
  announcedEntryIds = new Set(ids);
  const narration = added.filter((entry) =>
    entry.classList.contains("narration"),
  );
  if (narration.length === 0) return;
  writeLiveRegion(
    "narration-announcer",
    narration
      .map((entry) => {
        const text = entry.querySelector(".text");
        return text ? text.textContent.trim() : "";
      })
      .filter((text) => text.length > 0)
      .join("\n\n"),
  );
}

let announcedOptions = null;

function announceOptions() {
  const dock = document.getElementById("options-dock");
  if (!dock) return;
  const options = Array.from(dock.querySelectorAll(".option-btn")).map(
    (btn) => btn.textContent.trim(),
  );
  if (announcedOptions === null) {
    announcedOptions = options;
    return;
  }
  const unchanged =
    options.length === announcedOptions.length &&
    options.every((option, index) => option === announcedOptions[index]);
  if (unchanged) return;
  announcedOptions = options;
  if (options.length === 0) return;
  writeLiveRegion(
    "options-announcer",
    options.length + " options available: " + options.join("; "),
  );
}

function updateToThinking() {
  beginPending();
  syncActionState();
}

// The status display loads after each send the engine took. The first load that
// starts after the send ends the action area's own state, because that load
// reads the state the send caused.
document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const target = evt.detail && evt.detail.elt;
  if (sendPending && target && target.id === "status-display") {
    pendingLoadStarted = true;
  }
});

function readStatusDisplay() {
  if (sendPending && pendingLoadStarted) endPending();
  applyStatusDisplay();
}

document.body.addEventListener("htmx:afterSwap", function (evt) {
  syncActionState();
  const target = evt.detail && (evt.detail.target || evt.detail.elt);
  if (!target || !target.id) return;
  if (target.id === "status-display") {
    syncStatusClass();
  } else if (target.id === "story-log") {
    announceNewNarration();
  } else if (target.id === "options-dock") {
    announceOptions();
  }
});

function onCommandBeforeRequest() {
  checkInFlight = true;
  syncActionState();
}

// htmx marks a transport failure and a non-2xx answer unsuccessful, so a
// command the engine never took keeps the text the player typed.
function requestSucceeded(event) {
  return !!(event && event.detail && event.detail.successful);
}

function clearCommandInput() {
  const form = document.getElementById("command-form");
  if (form) form.reset();
}

function commandFormSlot() {
  return document.querySelector('#command-form [data-error-slot="action"]');
}

function onCommandAfterRequest(event) {
  checkInFlight = false;
  const refusal = answeredRefused(event);
  if (refusal) {
    // The engine did not take the command, so the input keeps it.
    reportRefusal(commandFormSlot(), "the command was not sent", refusal);
  } else if (requestSucceeded(event) && !previewIsOpen()) {
    // The preview keeps the flagged command for editing, and a dispatched
    // command has been consumed, so clear the input.
    clearCommandInput();
    beginPending();
  }
  syncActionState();
}

// Both of the preview's send controls post a command the engine consumes.
function onPreviewSendAfterRequest(event) {
  const refusal = answeredRefused(event);
  if (refusal) {
    reportRefusal(commandFormSlot(), "the command was not sent", refusal);
  } else if (requestSucceeded(event)) {
    clearCommandInput();
    beginPending();
  } else {
    reportConfirmFailure(event);
  }
  closeActionPreview();
}

// The Autoplay start takes the budget and answers nothing. A refusal reports in
// the form's own slot, and the run shows in the status display.
function onAutoplayAfterRequest(event) {
  const refusal = answeredRefused(event);
  if (refusal) {
    reportRefusal(
      document.querySelector('#autoplay-form [data-error-slot="autoplay"]'),
      "the run did not start",
      refusal,
    );
  } else if (requestSucceeded(event)) {
    beginPending();
    syncActionState();
  }
}

// A confirm the engine never took reports on the command form's own slot: the
// preview closes here, and a slot inside it would go with it. htmx fires this
// request's sendError after this event, on a preview that is already gone.
function reportConfirmFailure(event) {
  reportSlotFailure(
    commandFormSlot(),
    event && event.detail ? event.detail.xhr : null,
  );
}

function closeActionPreview() {
  const preview = document.getElementById("action-preview");
  if (preview) preview.innerHTML = "";
  syncActionState();
}

function useOption(btn) {
  const input = commandInput();
  // The input is disabled only during a text check, so a generation in
  // flight needs the explicit actionIsGenerating() guard.
  if (!input || input.disabled || actionIsGenerating()) return;
  input.value = btn.textContent;
  document.getElementById("command-form").requestSubmit();
}

function editOption(btn) {
  const input = commandInput();
  if (!input || input.disabled) return;
  const optionBtn = btn.closest(".option-item").querySelector(".option-btn");
  input.value = optionBtn.textContent;
  input.focus();
}

document.addEventListener("DOMContentLoaded", function () {
  // A page loaded mid-generation renders a thinking status span, so the reconciler must run at load.
  applyStatusDisplay();
});
