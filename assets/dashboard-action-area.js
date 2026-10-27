// The action area: the command form, its Checking/Generating state machine, the reading of the status display and the live-region announcements.

// The DOM carries every action-area state except an in-flight check and a send
// that the status display has not shown yet.
let checkInFlight = false;

// The action area's own state between an accepted request and the screen that
// shows its effect. The status display is a part of the screen: the action area
// reads it and never writes it. This state shows in #action-status. Each request
// that begins it ends it with its own answer: when the screen shows the revision
// of the answer, or at once when the engine did not take the request.
let sendPending = false;

function renderActionStatus() {
  const own = document.getElementById("action-status");
  if (!own) return;
  own.innerHTML = sendPending
    ? '<span class="status thinking">Thinking...</span>'
    : "";
}

function beginPending() {
  sendPending = true;
  renderActionStatus();
}

function endPending() {
  sendPending = false;
  renderActionStatus();
  syncActionState();
}

function endPendingWhenScreenShows(stamp) {
  afterScreenShows(stamp).then(endPending);
}

// A request that the engine took begins the state, and the screen that shows the
// revision of its answer ends it.
function beginPendingUntilScreenShows(event) {
  beginPending();
  endPendingWhenScreenShows(event.detail.xhr.getResponseHeader("X-Revision"));
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

// `data-state` on the action area is the browser's view state: a pending send, a
// check in flight and an open preview are known to the page only. The server
// renders the disabled state of every control, and a request disables only the
// shell field of its form through `hx-disabled-elt`, so no script writes `disabled`.
function applyActionState(state) {
  const area = document.getElementById("action-area");
  if (area) area.dataset.state = state;
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

function writeLiveRegion(id, text) {
  const region = document.getElementById(id);
  if (!region || region.textContent === text) return;
  region.textContent = text;
}

// The page starts at the label of an idle display, so a page that opens during a
// Job announces the step at its first load, and an idle page announces nothing.
let announcedStatusLabel = "Ready";

function announceStatusDisplay() {
  const statusDisplay = document.getElementById("status-display");
  if (!statusDisplay || statusDisplay.querySelector(".error-disclosure")) {
    return;
  }
  const label = statusDisplay.textContent.trim();
  if (label === "") return;
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

document.addEventListener("screen:loaded", function (evt) {
  const parts = evt.detail.parts;
  if (parts.includes("status-display")) applyStatusDisplay();
  else syncActionState();
  if (parts.includes("story-log")) announceNewNarration();
  if (parts.includes("options-dock")) announceOptions();
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

// The input clears once the screen shows the command that it sent, and only if it
// still holds the text that was sent: the player can type the next action before
// the screen shows the last one.
function clearCommandInputAfterScreenShows(event, sentText) {
  const stamp = event.detail.xhr.getResponseHeader("X-Revision");
  afterScreenShows(stamp).then(() => {
    const input = commandInput();
    if (input && input.value === sentText) clearCommandInput();
  });
}

function clearCommandInput() {
  const form = document.getElementById("command-form");
  if (form) form.reset();
}

function commandFormSlot() {
  return document.querySelector('#command-form [data-error-slot="action"]');
}

// A swap that throws skips the after-request event of the form, so the check is
// over here: the area must not stay in the state "checking".
document.body.addEventListener("htmx:onLoadError", function (evt) {
  const path = evt.detail.pathInfo && evt.detail.pathInfo.requestPath;
  if (path !== "/action/check") return;
  checkInFlight = false;
  syncActionState();
});

function onCommandAfterRequest(event) {
  checkInFlight = false;
  const refusal = answeredRefused(event);
  if (refusal) {
    // The engine did not take the command, so the input keeps it.
    reportRefusal(commandFormSlot(), "the command was not sent", refusal);
  } else if (requestSucceeded(event) && !previewIsOpen()) {
    // The preview keeps the flagged command for editing, and a dispatched
    // command has been consumed, so clear the input.
    clearCommandInputAfterScreenShows(event, event.detail.requestConfig.parameters.command);
    beginPendingUntilScreenShows(event);
  }
  syncActionState();
}

// The text that a preview send consumes is the text that the input held when the
// request began: the player can type on while the request is in flight.
let previewSentText = "";

document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const path = evt.detail.pathInfo && evt.detail.pathInfo.requestPath;
  if (path !== "/action/confirm") return;
  const input = commandInput();
  previewSentText = input ? input.value : "";
});

function onPreviewSendAfterRequest(event) {
  const refusal = answeredRefused(event);
  if (refusal) {
    reportRefusal(commandFormSlot(), "the command was not sent", refusal);
  } else if (requestSucceeded(event)) {
    clearCommandInputAfterScreenShows(event, previewSentText);
    beginPendingUntilScreenShows(event);
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
    beginPendingUntilScreenShows(event);
    syncActionState();
  }
}

// The new-options request begins the pending state before it is sent, from the
// form's before-request handler. Its answer ends the state, and a refusal reports
// in a slot of the form.
function onNewOptionsAfterRequest(event) {
  const refusal = answeredRefused(event);
  if (refusal) {
    reportRefusal(
      inlineErrorSlotFor(event.detail.elt, true),
      "the new options were not requested",
      refusal,
    );
    endPending();
  } else if (requestSucceeded(event)) {
    endPendingWhenScreenShows(event.detail.xhr.getResponseHeader("X-Revision"));
  } else {
    endPending();
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
  // The input is disabled only while a check request runs, so a generation in
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
