// The edit dialog: the one modal that holds a form that waits for a Save, or the answer of an action. A swap into its body opens it, Cancel, Escape and an accepted save close it, and closing discards the form.

const editDialog = document.getElementById("edit-dialog");
const editDialogBody = document.getElementById("edit-dialog-body");

const FOCUSABLE = "input:not([type=hidden]), select, textarea, button, a[href]";
// A form field is the better landing spot: an Edit that opens a form should put
// the caret in its first field, not on a control above it.
const FOCUSABLE_FIELD = "input:not([type=hidden]), select, textarea";

// `focus()` is a no-op on a hidden or detached control, so a candidate counts
// only once it holds focus.
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

// A load can replace the opener node, but the server renders the same id on the
// control that replaces it.
let dialogOpenerId = "";

document.body.addEventListener("htmx:afterSwap", function (evt) {
  if (!evt.detail || evt.detail.target !== editDialogBody) return;
  const request = evt.detail.requestConfig;
  dialogOpenerId = request && request.elt ? request.elt.id : "";
  if (!editDialog.open) editDialog.showModal();
  renderDialogStatus();
  focusInto(editDialogBody);
});

// The dialog is modal, so an answer that comes from a control outside it was sent
// before the dialog opened. It must not replace the form that the player is
// working in.
document.body.addEventListener("htmx:beforeSwap", function (evt) {
  if (!evt.detail || evt.detail.target !== editDialogBody || !editDialog.open) return;
  const request = evt.detail.requestConfig;
  if (request && request.elt && editDialog.contains(request.elt)) return;
  evt.detail.shouldSwap = false;
});

// `showModal()` returns focus to the node that had it when the dialog opened, and a
// load can have replaced that node, so focus is given to the control that is there now.
function returnFocusAfterClose() {
  const opener = document.getElementById(dialogOpenerId);
  dialogOpenerId = "";
  if (opener) opener.focus();
}

// Each open of the dialog is one session. A save that waits for the screen closes
// only the session that sent it, so it never closes a dialog that opened after it.
let dialogSession = 0;
let noticeSummary = "";
let unreachableText = "";

// The modal hides the banner, so the status line repeats the summary of the notice
// and the Unreachable text of the banner.
function renderDialogStatus() {
  const status = document.getElementById("edit-dialog-status");
  if (!status) return;
  const lines = [];
  if (editDialog.hasAttribute("data-saving")) lines.push("Saving…");
  if (editDialog.open && noticeSummary) lines.push(noticeSummary);
  if (editDialog.open && unreachableText) lines.push(unreachableText);
  const text = lines.join(" ");
  if (status.textContent !== text) status.textContent = text;
}

document.addEventListener("notice:changed", function (evt) {
  noticeSummary = evt.detail.summary;
  renderDialogStatus();
});

document.addEventListener("banner:changed", function (evt) {
  unreachableText = evt.detail.text;
  renderDialogStatus();
});

editDialog.addEventListener("close", function () {
  returnFocusAfterClose();
  editDialogBody.replaceChildren();
  editDialog.removeAttribute("data-saving");
  dialogSession += 1;
  renderDialogStatus();
});

document.addEventListener("click", function (evt) {
  const target = evt.target;
  if (!(target instanceof Element)) return;
  const cancel = target.closest("[data-close-dialog]");
  if (cancel && editDialog.contains(cancel)) editDialog.close();
});

// A save closes the dialog when the engine accepts it and the screen shows the
// save. The dialog says "Saving…" until then, and takes no second submit: the
// form is still on screen, and a second save would repeat the write. A refused or
// failed save keeps the dialog open with the typed values. The Test control of a form
// is a button, so it never closes the dialog.
document.body.addEventListener("htmx:confirm", function (evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element || element.tagName !== "FORM") return;
  if (editDialog.hasAttribute("data-saving") && editDialog.contains(element)) {
    evt.preventDefault();
  }
});

document.body.addEventListener("htmx:afterRequest", function (evt) {
  const form = evt.detail && evt.detail.elt;
  if (!form || form.tagName !== "FORM" || !editDialog.contains(form)) return;
  const answer = actionAnswer(evt);
  if (answer.outcome !== "done") return;
  const session = dialogSession;
  editDialog.setAttribute("data-saving", "");
  renderDialogStatus();
  afterScreenShows(answer.stamp).then(() => {
    if (editDialog.open && session === dialogSession) editDialog.close();
  });
});
