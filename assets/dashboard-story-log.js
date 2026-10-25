// The story log: edit mode, its entry locks, the swipe/retry/delete requests, the follow-new-entries scroll and its own error slot.

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

function lockOtherEditButtons() {
  document.querySelectorAll("#story-log .edit-btn").forEach((btn) => {
    btn.disabled = true;
  });
}

function unlockEditButtons() {
  document.querySelectorAll("#story-log .edit-btn").forEach((btn) => {
    btn.disabled = false;
  });
}

function showEditForm(id) {
  if (editState !== null) return;
  editState = { id: id, text: null, actions: null, swipeControls: null };
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
    const swipeControls = entry.querySelector(".swipe-controls");
    if (swipeControls) {
      editState.swipeControls = swipeControls.innerHTML;
      swipeControls.querySelectorAll(".action-btn").forEach((btn) => {
        btn.disabled = true;
      });
    }

    // The open editor holds the story log: a load waits until the edit ends.
    entry.setAttribute("data-work", "");
    lockOtherEditButtons();
    textarea.focus();
  }
}

function clearEditState() {
  document
    .querySelectorAll("#story-log [data-work]")
    .forEach((root) => root.removeAttribute("data-work"));
  editState = null;
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
    const swipeControls = entry.querySelector(".swipe-controls");
    if (swipeControls && editState.swipeControls !== null) {
      swipeControls.innerHTML = editState.swipeControls;
    }
  }
  clearEditState();
  unlockEditButtons();
}

function cancelEdit() {
  if (editState === null) return;
  const entryId = editState.id;
  revertEdit();
  liveWorkEnded();
  const entry = document.querySelector(`[data-id="${entryId}"]`);
  const editBtn = entry ? entry.querySelector(".edit-btn") : null;
  if (editBtn) editBtn.focus();
}

// A 200 can still refuse a story write: a Job or an Autoplay run holds the Game.
// True when the body is the busy answer; the story-log error then says what the
// refused action was.
function reportedBusy(body, message) {
  if (!isBusyAnswer(body)) return false;
  showStoryLogError(message, BUSY_REFUSAL_DETAIL);
  return true;
}

function submitEdit() {
  if (editState === null) return;
  clearInlineError(storyLogErrorSlot());
  const newText = document.getElementById("edit-textarea").value;
  const formData = new URLSearchParams();
  formData.append("text", newText);
  let editKeptOpen = false;
  // Kept set until the outcome is known, so a failed save can restore the entry.
  actionFetch(`/history/${editState.id}`, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: formData,
  })
    .then((response) => {
      if (!response.ok) return throwServerDetail(response);
      return response.text();
    })
    .then((body) => {
      if (reportedBusy(body, "The Game is busy, so the edit was not saved.")) {
        editKeptOpen = true;
        return;
      }
      // The saved text is the rendered text now, so the textarea is no longer
      // a changed field. The lock stays until the load replaces the entry's
      // controls, so a second editor cannot open on a stale textarea.
      const saved = document.getElementById("edit-textarea");
      if (saved) saved.defaultValue = saved.value;
      clearEditState();
    })
    .catch((err) => {
      console.error("Failed to save message:", err);
      revertEdit();
      showStoryLogError("Failed to save the message.", err.message);
    })
    .finally(() => {
      if (!editKeptOpen) liveWorkEnded();
    });
}

// Retry and retrigger answer nothing when the engine takes them, and the Job
// they start shows in the status display. A refusal or a failure reports in the
// story log's error slot and ends the action area's own state.
function submitGenerationRequest(url, failureMessage, busyMessage) {
  clearInlineError(storyLogErrorSlot());
  updateToThinking();
  actionFetch(url, { method: "POST" })
    .then((response) => {
      if (!response.ok) return throwServerDetail(response);
      return response.text();
    })
    .then((body) => {
      if (reportedBusy(body, busyMessage)) endPending();
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
    "The Game is busy, so the retry was not started.",
  );
}

function submitRetrigger() {
  submitGenerationRequest(
    "/retrigger",
    "Failed to retrigger the event",
    "The Game is busy, so the retrigger was not started.",
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
      if (!response.ok) return throwServerDetail(response);
      return response.text();
    })
    .then((body) => {
      if (reportedBusy(body, "The Game is busy, so the swipe was not switched.")) {
        return;
      }
      // The story log loads the restored Swipe, so the restore is announced
      // once that load has settled.
      return chroniclerLive.settle().then(announceSwipeRestore);
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
      if (!response.ok) return throwServerDetail(response);
      return response.text().then((body) => {
        if (reportedBusy(body, "The Game is busy, so the message was not deleted.")) {
          return;
        }
        return chroniclerLive.settle().then(focusAfterLogDelete);
      });
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
// only once the regions beside it have loaded (the options dock alone takes
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

document.body.addEventListener("htmx:beforeSwap", function (evt) {
  const target = evt.detail && evt.detail.target;
  if (!target || target.id !== "story-log" || !evt.detail.shouldSwap) return;
  storyLogSwapStart = {
    atBottom: storyLogIsAtBottom(target),
    entries: storyLogEntryCount(target),
  };
});

document.body.addEventListener("htmx:afterSwap", function (evt) {
  const start = storyLogSwapStart;
  storyLogSwapStart = null;
  const target = evt.detail && (evt.detail.target || evt.detail.elt);
  if (!start || !target || target.id !== "story-log") return;
  // An open edit owns the log's position, and the log does not load while it is open.
  if (editState !== null) return;
  if (!start.atBottom) return;
  if (storyLogEntryCount(target) <= start.entries) return;
  followStoryLogBottom(target);
});
