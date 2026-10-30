// The action area: the command form, its Checking/Generating state machine, the reading of the status display and the live-region announcements.

// The DOM carries every action-area state except an in-flight check and a send
// that the status display has not shown yet.
let checkInFlight = false;

// The action area's own state between an accepted request and the screen that
// shows its effect: the status display is a part, which the action area reads and
// never writes.
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

function beginPendingUntilScreenShows(stamp) {
  beginPending();
  endPendingWhenScreenShows(stamp);
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

// The server renders the disabled state of every control, so no script writes
// `disabled`.
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

// The input clears only if it still holds the text that was sent: the player can
// type the next action before the screen shows the last one.
function clearCommandInputAfterScreenShows(stamp, sentText) {
  afterScreenShows(stamp).then(() => {
    const input = commandInput();
    if (input && input.value === sentText) clearCommandInput();
  });
}

function clearCommandInput() {
  const form = document.getElementById("command-form");
  if (form) form.reset();
  syncCommandDraft();
}

// The draft pins the Game that the screen showed when it began, so a send names
// that Game even after a load shows another one.
function syncCommandDraft() {
  const form = document.getElementById("command-form");
  const input = commandInput();
  if (!form || !input) return;
  if (input.value === "") form.removeAttribute("data-draft-game");
  else if (!form.hasAttribute("data-draft-game")) {
    form.setAttribute("data-draft-game", screenRoot.dataset.game || "");
  }
}

// An Option comes from the screen of the current Game, so it starts a new draft.
function startNewCommandDraft() {
  const form = document.getElementById("command-form");
  if (form) form.removeAttribute("data-draft-game");
  syncCommandDraft();
}

// A `409` of a request of the Action input is a busy Game or a Game that is no
// longer active. The draft then takes the Game that the screen shows once it
// shows the answer.
function retakeDraftGameWhenScreenShows(stamp) {
  afterScreenShows(stamp).then(() => {
    const form = document.getElementById("command-form");
    if (form && form.hasAttribute("data-draft-game")) {
      form.setAttribute("data-draft-game", screenRoot.dataset.game || "");
    }
  });
}

document.addEventListener("input", function (event) {
  if (event.target === commandInput()) syncCommandDraft();
});

// A swap that throws skips the after-request event of the form, so the check is
// over here: the area must not stay in the state "checking".
function onCommandLoadError() {
  checkInFlight = false;
  syncActionState();
}

function onCommandAfterRequest(event) {
  checkInFlight = false;
  const answer = requestAnswer(event);
  if (answer.outcome === "refused") retakeDraftGameWhenScreenShows(answer.stamp);
  if (answer.outcome === "done" && !previewIsOpen()) {
    clearCommandInputAfterScreenShows(answer.stamp, event.detail.requestConfig.parameters.command);
    beginPendingUntilScreenShows(answer.stamp);
  }
  syncActionState();
}

// The player can type on while a preview send is in flight, so the send consumes
// the text that the input held when the request began.
let previewSentText = "";

function onPreviewBeforeRequest() {
  const input = commandInput();
  previewSentText = input ? input.value : "";
}

function onPreviewSendAfterRequest(event) {
  const answer = requestAnswer(event);
  if (answer.outcome === "refused") retakeDraftGameWhenScreenShows(answer.stamp);
  if (answer.outcome !== "done") return;
  clearCommandInputAfterScreenShows(answer.stamp, previewSentText);
  beginPendingUntilScreenShows(answer.stamp);
  closeActionPreview();
}

function onAutoplayAfterRequest(event) {
  const answer = requestAnswer(event);
  if (answer.outcome !== "done") return;
  beginPendingUntilScreenShows(answer.stamp);
  syncActionState();
}

function endPendingAfterRequest(event) {
  const answer = requestAnswer(event);
  if (answer.outcome === "done") endPendingWhenScreenShows(answer.stamp);
  else endPending();
}

function closeActionPreview() {
  const preview = document.getElementById("action-preview");
  if (preview) preview.innerHTML = "";
  syncActionState();
}

function useOption(btn) {
  const input = commandInput();
  if (!input || checkInFlight || actionIsGenerating()) return;
  input.value = btn.textContent;
  startNewCommandDraft();
  document.getElementById("command-form").requestSubmit();
}

function editOption(btn) {
  const input = commandInput();
  if (!input || checkInFlight) return;
  const optionBtn = btn.closest(".option-item").querySelector(".option-btn");
  input.value = optionBtn.textContent;
  startNewCommandDraft();
  input.focus();
}
