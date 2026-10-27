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

// The control that opened the dialog, the request it sends and the part that
// holds it. A load morphs a part in place, so after a load the node that opened
// the dialog can be a different control: the request, not the node, names the
// opener.
let dialogOpener = null;
let dialogOpenerRequest = null;
let dialogOpenerPart = null;

const OPENER_REQUEST_ATTRIBUTES = ["hx-get", "hx-post"];

function requestOf(element) {
  for (const name of OPENER_REQUEST_ATTRIBUTES) {
    const value = element.getAttribute(name);
    if (value !== null) return { name, value };
  }
  return null;
}

function hasRequest(element, request) {
  return element.getAttribute(request.name) === request.value;
}

function partHolding(element) {
  return element.closest("[data-part]");
}

document.body.addEventListener("htmx:afterSwap", function (evt) {
  if (!evt.detail || evt.detail.target !== editDialogBody) return;
  const request = evt.detail.requestConfig;
  dialogOpener = request ? request.elt : null;
  dialogOpenerRequest = dialogOpener ? requestOf(dialogOpener) : null;
  dialogOpenerPart = dialogOpener ? partHolding(dialogOpener) : null;
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

// The control that opened the dialog now: the opener itself while it still sends
// its request, else the control of its part that sends that request, else the
// part.
function openerNow(opener, request, part) {
  if (opener && opener.isConnected && (!request || hasRequest(opener, request))) {
    return opener;
  }
  if (part && part.isConnected && request) {
    for (const candidate of part.querySelectorAll(`[${request.name}]`)) {
      if (hasRequest(candidate, request)) return candidate;
    }
  }
  return part && part.isConnected ? part : null;
}

// `showModal()` returns focus to the node that had it when the dialog opened. A
// load can have replaced that node or rewritten it into another control, so focus
// goes to the control that sends the opener's request.
function returnFocusAfterClose() {
  const focused = document.activeElement;
  const opener = dialogOpener;
  const request = dialogOpenerRequest;
  const part = dialogOpenerPart;
  dialogOpener = null;
  dialogOpenerRequest = null;
  dialogOpenerPart = null;
  const focusIsFree =
    !focused || focused === document.body || editDialog.contains(focused) || focused === opener;
  if (!focusIsFree) return;
  const target = openerNow(opener, request, part);
  if (!target) return;
  if (target === part && !part.hasAttribute("tabindex")) {
    part.setAttribute("tabindex", "-1");
  }
  target.focus();
}

// Each open of the dialog is one session. A save that waits for the screen closes
// only the session that sent it, so it never closes a dialog that opened after it.
let dialogSession = 0;
let screenErrorSummary = "";

// The status line says that a save waits for the screen, and why the screen does
// not show yet: the slot of the screen error is behind the modal.
function renderDialogStatus() {
  const status = document.getElementById("edit-dialog-status");
  if (!status) return;
  const lines = [];
  if (editDialog.hasAttribute("data-saving")) lines.push("Saving…");
  if (editDialog.open && screenErrorSummary) lines.push(screenErrorSummary);
  const text = lines.join(" ");
  if (status.textContent !== text) status.textContent = text;
}

document.addEventListener("screen:error", function (evt) {
  screenErrorSummary = evt.detail.summary;
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
// failed save keeps the dialog open with the typed values, and `settleSave` and
// the error handlers report the failure or the busy refusal in the form's own
// slot. The Test control of a form is a button, so it never closes the dialog.
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
  if (!evt.detail.successful) return;
  const session = dialogSession;
  editDialog.setAttribute("data-saving", "");
  renderDialogStatus();
  afterScreenShows(evt.detail.xhr.getResponseHeader("X-Revision")).then(() => {
    if (editDialog.open && session === dialogSession) editDialog.close();
  });
});
