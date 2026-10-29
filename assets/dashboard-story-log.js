// The story log: edit mode, its entry lock, what follows the swipe, retry, retrigger and delete requests, and the follow-new-entries scroll.

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
  editState = { id: id, text: null };
  const entry = document.querySelector(`[data-id="${id}"]`);
  if (entry) {
    const textSpan = entry.querySelector(".text");

    // Edit the markdown source, not the rendered HTML, so a save round-trips the original text.
    const rawText = entry.getAttribute("data-raw-text") || "";
    editState.text = textSpan.innerHTML;

    textSpan.innerHTML = "";
    const textarea = document.createElement("textarea");
    textarea.id = "edit-textarea";
    textarea.name = "text";
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
        if (editState === null) return;
        entry.querySelector(".save-btn").click();
      }
    });

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

// The lock of a done Save is released only once the screen shows the saved entry, so a
// second editor cannot open on a stale textarea.
function onEditSaved(event) {
  if (editState === null) return;
  const answer = actionAnswer(event);
  if (answer.outcome === "refused" || answer.outcome === "invalid") return;
  if (answer.outcome !== "done") {
    revertEdit();
    return;
  }
  clearEditState();
  afterScreenShows(answer.stamp).then(unlockEditing);
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

function onSwipeSwitched(event) {
  const answer = actionAnswer(event);
  if (answer.outcome === "done") afterScreenShows(answer.stamp).then(announceSwipeRestore);
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

function onMessageDeleted(event) {
  const answer = actionAnswer(event);
  if (answer.outcome === "done") afterScreenShows(answer.stamp).then(focusAfterLogDelete);
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
