// The action area: the command form, its Checking/Generating state machine, the status display and the live-region announcements.

// The DOM carries every action-area state except an in-flight check.
let checkInFlight = false;

function statusIsGenerating() {
  const statusDisplay = document.getElementById("status-display");
  return !!(statusDisplay && statusDisplay.querySelector(".status.thinking"));
}

function previewIsOpen() {
  const preview = document.getElementById("action-preview");
  return !!(preview && preview.firstElementChild !== null);
}

function deriveActionState() {
  if (checkInFlight) return "checking";
  if (statusIsGenerating()) return "generating";
  if (previewIsOpen()) return "preview";
  return "idle";
}

function applyActionState(state) {
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

function setStatus(label, statusClass) {
  const statusDisplay = document.getElementById("status-display");
  if (statusDisplay) {
    statusDisplay.innerHTML =
      '<span class="status ' + statusClass + '">' + label + "</span>";
  }
}

// The container's own class colours whatever it holds, and a server-rendered
// error disclosure replaces the inner span without touching it, so the state is
// read back off the content instead of being remembered. Only the state classes
// are touched, so htmx's own transient classes survive.
const STATUS_STATE_CLASSES = ["ready", "thinking", "wait", "error"];

function statusDisplayState(statusDisplay) {
  if (statusDisplay.querySelector(".error-disclosure, .status.error")) {
    return "error";
  }
  if (statusIsGenerating()) return "thinking";
  if (statusDisplay.querySelector(".status.wait")) return "wait";
  return "ready";
}

function syncStatusClass() {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay) return;
  statusDisplay.classList.remove(...STATUS_STATE_CLASSES);
  statusDisplay.classList.add(statusDisplayState(statusDisplay));
}

// A 200 answer can still refuse the command: the engine reports a generation
// already in flight as the wait state, and that command was never taken.
function commandWasRefused() {
  const statusDisplay = document.getElementById("status-display");
  return !!statusDisplay && statusDisplayState(statusDisplay) === "wait";
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
  setStatus("Thinking...", "thinking");
  applyStatusDisplay();
}

// /status/generating returns a bare phase name or "idle" as plain text,
// or an error fragment, which stays in place for applyStatusDisplay.
function onStatusPoll(el) {
  const text = el.textContent.trim();
  const phaseMap = {
    narrating: "Generating narration...",
    quantifying: "Quantifying scene...",
    "generating-event": "Generating event...",
    options: "Generating options...",
  };
  if (text === "idle") {
    setStatus("Ready", "ready");
  } else if (phaseMap[text]) {
    setStatus(phaseMap[text], "thinking");
  }
  applyStatusDisplay();
}

// The status poll replaces this disclosure on its next cycle.
function showStatusError(message, rawText) {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay) return;
  statusDisplay.innerHTML = errorDisclosureHtml(
    "status-error-popover",
    message,
    rawText
  );
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

// Raw-fetch paths get no htmx swap, so a failure would otherwise leave
// "Thinking..." with Send stuck as a generating indicator.
function resetStatusToReady() {
  setStatus("Ready", "ready");
  applyStatusDisplay();
}

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

function onCommandAfterRequest(event) {
  checkInFlight = false;
  // The preview keeps the flagged command for editing, and a dispatched
  // command has been consumed, so clear the input. A refused one was not.
  if (requestSucceeded(event) && !previewIsOpen() && !commandWasRefused()) {
    clearCommandInput();
  }
  syncActionState();
}

// Both of the preview's send controls post a command the engine consumes.
function onPreviewSendAfterRequest(event) {
  if (requestSucceeded(event)) {
    clearCommandInput();
  } else {
    reportConfirmFailure(event);
  }
  closeActionPreview();
}

// A confirm the engine never took reports on the command form's own slot: the
// preview closes here, and a slot inside it would go with it. htmx fires this
// request's sendError after this event, on a preview that is already gone.
function reportConfirmFailure(event) {
  reportSlotFailure(
    document.querySelector('#command-form [data-error-slot="action"]'),
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
  // flight needs the explicit statusIsGenerating() guard.
  if (!input || input.disabled || statusIsGenerating()) return;
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
