// The story log: edit mode, its entry lock, the swipe/retry/delete requests, the follow-new-entries scroll and its own error slot.

const noticeClearTimers = new WeakMap();

function scheduleNoticeClear(element, onClear, delayMs) {
  const pending = noticeClearTimers.get(element);
  if (pending !== undefined) clearTimeout(pending);
  noticeClearTimers.set(
    element,
    setTimeout(() => {
      noticeClearTimers.delete(element);
      onClear();
    }, delayMs)
  );
}

function storyLogErrorSlot() {
  return document.getElementById("story-log-error");
}

function showStoryLogError(message, rawText) {
  renderInlineError(storyLogErrorSlot(), message, rawText);
}

let editState = null;

function autoResizeTextarea(textarea) {
  textarea.style.height = "auto";
  // `box-sizing: border-box` excludes the border from `height`, so add it
  // back or the textarea overflows and grows its own scrollbar.
  const border = textarea.offsetHeight - textarea.clientHeight;
  textarea.style.height = textarea.scrollHeight + border + "px";
}

// While an entry is in edit, the root of the story log carries
// `data-editing-entry`. CSS shows the Edit buttons and the swipe controls as
// disabled, and the click guard below stops a keyboard activation. A load morphs
// only the children of the root, so the lock survives every load. It ends after
// the load that follows a saved edit, so the saved entry's controls stay locked
// until the entry shows the saved text.
function storyLogRoot() {
  return document.getElementById("story-log");
}

function lockEditing() {
  const root = storyLogRoot();
  if (root) root.setAttribute("data-editing-entry", "");
}

function unlockEditing() {
  const root = storyLogRoot();
  if (root) root.removeAttribute("data-editing-entry");
}

document.addEventListener(
  "click",
  function (evt) {
    const target = evt.target;
    if (!(target instanceof Element)) return;
    const locked = target.closest(
      "#story-log[data-editing-entry] :is(.edit-btn, .swipe-controls .action-btn)",
    );
    if (!locked) return;
    evt.preventDefault();
    evt.stopPropagation();
  },
  true,
);

function showEditForm(id) {
  if (editState !== null) return;
  editState = { id: id, text: null, actions: null };
  const entry = document.querySelector(`[data-id="${id}"]`);
  if (entry) {
    const textSpan = entry.querySelector(".text");

    // Edit the markdown source, not the rendered HTML, so a save round-trips the original text.
    const rawText = entry.getAttribute("data-raw-text") || "";
    editState.text = textSpan.innerHTML;

    textSpan.innerHTML = "";
    const textarea = document.createElement("textarea");
    textarea.id = "edit-textarea";
    textarea.className = "edit-textarea";
    textarea.textContent = rawText;
    textSpan.appendChild(textarea);
    autoResizeTextarea(textarea);

    textarea.addEventListener("input", function () {
      autoResizeTextarea(this);
    });
    textarea.addEventListener("keydown", function (evt) {
      if (evt.key === "Escape") {
        evt.preventDefault();
        cancelEdit();
      } else if (evt.key === "Enter" && (evt.ctrlKey || evt.metaKey)) {
        evt.preventDefault();
        submitEdit();
      }
    });

    const actionsContainer = entry.querySelector(".message-actions");
    if (actionsContainer) {
      editState.actions = actionsContainer.innerHTML;
      actionsContainer.innerHTML = `<button class="save-btn" onclick="submitEdit()" title="Save" aria-label="Save"><svg class="icon" aria-hidden="true"><use href="#i-check"></use></svg></button><button class="cancel-btn" onclick="cancelEdit()" title="Cancel" aria-label="Cancel"><svg class="icon" aria-hidden="true"><use href="#i-x"></use></svg></button>`;
    }
    // A load skips the entry in edit, so new narration shows around the draft.
    entry.setAttribute("data-editing", "");
    lockEditing();
    textarea.focus();
  }
}

function clearEditState() {
  document
    .querySelectorAll("#story-log [data-editing]")
    .forEach((entry) => entry.removeAttribute("data-editing"));
  editState = null;
  forgetScreenStamp();
  askNow();
}

function revertEdit() {
  if (editState === null) return;
  const entry = document.querySelector(`[data-id="${editState.id}"]`);
  if (entry) {
    const textSpan = entry.querySelector(".text");
    textSpan.innerHTML = editState.text;
    const actionsContainer = entry.querySelector(".message-actions");
    if (actionsContainer && editState.actions !== null) {
      actionsContainer.innerHTML = editState.actions;
    }
  }
  clearEditState();
  unlockEditing();
}

function cancelEdit() {
  if (editState === null) return;
  const entryId = editState.id;
  revertEdit();
  const entry = document.querySelector(`[data-id="${entryId}"]`);
  const editBtn = entry ? entry.querySelector(".edit-btn") : null;
  if (editBtn) editBtn.focus();
}

// The engine refuses a story write with 409: a Job or an Autoplay run holds the
// Game, or the page shows a Game that is no longer active. True when the response
// is a refusal; the story-log error then says what the refused action was
// (`refusedWhat`, for example "the edit was not saved").
function reportedRefusal(response, refusedWhat) {
  const refusal = refusalOfResponse(response);
  if (!refusal) return false;
  reportRefusal(storyLogErrorSlot(), refusedWhat, refusal);
  return true;
}

function submitEdit() {
  if (editState === null) return;
  clearInlineError(storyLogErrorSlot());
  const newText = document.getElementById("edit-textarea").value;
  const formData = new URLSearchParams();
  formData.append("text", newText);
  // Kept set until the outcome is known, so a failed save can restore the entry.
  actionFetch(`/history/${editState.id}`, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: formData,
  })
    .then((response) => {
      if (reportedRefusal(response, "the edit was not saved")) return;
      if (!response.ok) return throwServerDetail(response);
      // The lock stays until the screen shows the saved entry, so a second
      // editor cannot open on a stale textarea.
      clearEditState();
      afterScreenShows(response.headers.get("X-Revision")).then(unlockEditing);
    })
    .catch((err) => {
      console.error("Failed to save message:", err);
      revertEdit();
      showStoryLogError("Failed to save the message.", err.message);
    });
}

// Retry and retrigger answer nothing when the engine takes them, and the Job
// they start shows in the status display. A refusal or a failure reports in the
// story log's error slot and ends the action area's own state.
function submitGenerationRequest(url, failureMessage, refusedWhat) {
  clearInlineError(storyLogErrorSlot());
  updateToThinking();
  actionFetch(url, { method: "POST" })
    .then((response) => {
      if (reportedRefusal(response, refusedWhat)) {
        endPending();
        return;
      }
      if (!response.ok) return throwServerDetail(response);
    })
    .catch((err) => {
      console.error(`${failureMessage}:`, err);
      endPending();
      showStoryLogError(failureMessage, err.message);
    });
}

function submitNewSwipe() {
  submitGenerationRequest(
    "/swipe/new",
    "Failed to generate a new swipe",
    "the retry was not started",
  );
}

function submitRetrigger() {
  submitGenerationRequest(
    "/retrigger",
    "Failed to retrigger the event",
    "the retrigger was not started",
  );
}

// A restored swipe leaves no state signal, so the restore is announced explicitly.
function announceSwipeRestore() {
  const notice = document.getElementById("restore-notice");
  if (!notice) return;
  const counter = document.querySelector("#story-log .swipe-counter");
  const label = counter ? counter.textContent.trim() : "";
  notice.textContent = label ? `Restored swipe ${label}` : "Swipe restored";
  scheduleNoticeClear(notice, () => {
    notice.textContent = "";
  }, 4000);
}

function switchSwipe(messageId, index) {
  clearInlineError(storyLogErrorSlot());
  actionFetch(`/message/${messageId}/swipe/${index}`, { method: "POST" })
    .then((response) => {
      if (reportedRefusal(response, "the swipe was not switched")) return;
      if (!response.ok) return throwServerDetail(response);
      // The story log shows the restored Swipe once the screen holds the
      // revision of the switch, and the restore is announced then.
      return afterScreenShows(response.headers.get("X-Revision")).then(announceSwipeRestore);
    })
    .catch((err) => {
      console.error("Swipe switch failed:", err);
      showStoryLogError("Failed to switch the swipe.", err.message);
    });
}

// The Delete control lives on the last entry, so the swap replaces the node that
// had focus. The entry that becomes the last one carries the equivalent
// controls, and the command input is the landing spot for a log left empty.
function focusAfterLogDelete() {
  const entries = document.querySelectorAll("#story-log .log-entry");
  const lastEntry = entries.length ? entries[entries.length - 1] : null;
  const landing =
    (lastEntry && lastEntry.querySelector(".edit-btn")) ||
    document.getElementById("command-input");
  if (landing) landing.focus();
}

function deleteMessage() {
  if (!confirm("Delete this message?")) return;
  clearInlineError(storyLogErrorSlot());
  actionFetch("/history/delete", { method: "POST" })
    .then((response) => {
      if (reportedRefusal(response, "the message was not deleted")) return;
      if (!response.ok) return throwServerDetail(response);
      return afterScreenShows(response.headers.get("X-Revision")).then(focusAfterLogDelete);
    })
    .catch((err) => {
      console.error("Delete failed:", err);
      showStoryLogError("Failed to delete the message.", err.message);
    });
}

// A load morphs the log in place, so its scroll position survives the swap and
// a new entry would land below the fold. A swap that adds an entry scrolls the
// log back to its bottom, but only when the log was already there: a player
// reading further up keeps their place.
const STORY_LOG_BOTTOM_SLACK_PX = 8;
let storyLogSwapStart = null;

function storyLogIsAtBottom(log) {
  return (
    log.scrollHeight - log.scrollTop - log.clientHeight <=
    STORY_LOG_BOTTOM_SLACK_PX
  );
}

function storyLogEntryCount(log) {
  return log.querySelectorAll(".log-entry").length;
}

function storyLogCanScroll(log) {
  return log.scrollHeight > log.clientHeight;
}

// The log's height is what the flex column has left over, so its box settles
// only once the parts beside it have loaded (the options dock alone takes
// ~150px of it). An assignment made in the swap, or in the next frame, can
// therefore still measure a log that does not overflow yet and clamp to the
// top. While the log cannot scroll there is no position of the player's to
// respect, so the follow retries each frame until the box can hold the bottom.
const STORY_LOG_FOLLOW_TIMEOUT_MS = 1000;

function followStoryLogBottom(log) {
  log.scrollTop = log.scrollHeight;
  if (storyLogCanScroll(log)) return;
  const deadline = performance.now() + STORY_LOG_FOLLOW_TIMEOUT_MS;
  const retry = () => {
    // An edit opened meanwhile owns the log's position.
    if (editState !== null) return;
    if (storyLogCanScroll(log)) {
      log.scrollTop = log.scrollHeight;
      return;
    }
    if (performance.now() < deadline) requestAnimationFrame(retry);
  };
  requestAnimationFrame(retry);
}

document.addEventListener("screen:beforemorph", function (evt) {
  const log = storyLogRoot();
  if (!log || !evt.detail.parts.includes("story-log")) return;
  storyLogSwapStart = {
    atBottom: storyLogIsAtBottom(log),
    entries: storyLogEntryCount(log),
  };
});

document.addEventListener("screen:loaded", function (evt) {
  const start = storyLogSwapStart;
  storyLogSwapStart = null;
  const log = storyLogRoot();
  if (!start || !log || !evt.detail.parts.includes("story-log")) return;
  // An open edit owns the log's position.
  if (editState !== null) return;
  if (!start.atBottom) return;
  if (storyLogEntryCount(log) <= start.entries) return;
  followStoryLogBottom(log);
});

// A tab change empties the story log, so an edit that was open is discarded and
// the lock of the Edit buttons goes with it.
document.addEventListener("screen:cleared", function () {
  editState = null;
  unlockEditing();
});
