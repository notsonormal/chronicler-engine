// The edit dialog: the one modal that holds a form that waits for a Save, or the answer of an action. A swap into its body opens it, Cancel, Escape and an accepted save close it, and closing discards the form.

const editDialog = document.getElementById("edit-dialog");
const editDialogBody = document.getElementById("edit-dialog-body");

// The control that opened the dialog, and the region that holds it: focus goes
// back to the region when a load has replaced the control.
let dialogOpener = null;
let dialogOpenerRegion = null;

document.body.addEventListener("htmx:afterSwap", function (evt) {
  if (!evt.detail || evt.detail.target !== editDialogBody) return;
  const request = evt.detail.requestConfig;
  dialogOpener = request ? request.elt : null;
  dialogOpenerRegion = dialogOpener ? dialogOpener.closest("[data-live]") : null;
  if (!editDialog.open) editDialog.showModal();
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

// `showModal()` returns focus to the control that was focused when it opened. A
// control that a load replaced is gone, so focus goes to its region instead.
function returnFocusAfterClose() {
  const focused = document.activeElement;
  const opener = dialogOpener;
  const region = dialogOpenerRegion;
  dialogOpener = null;
  dialogOpenerRegion = null;
  if (focused && focused !== document.body && !editDialog.contains(focused)) return;
  if (opener && opener.isConnected) {
    opener.focus();
  } else if (region && region.isConnected) {
    if (!region.hasAttribute("tabindex")) region.setAttribute("tabindex", "-1");
    region.focus();
  }
}

editDialog.addEventListener("close", function () {
  returnFocusAfterClose();
  editDialogBody.replaceChildren();
});

document.addEventListener("click", function (evt) {
  const target = evt.target;
  if (!(target instanceof Element)) return;
  const cancel = target.closest("[data-close-dialog]");
  if (cancel && editDialog.contains(cancel)) editDialog.close();
});

// A save closes the dialog when the engine accepts it. A refused or failed save
// keeps the dialog open with the typed values, and `settleSave` and the error
// handlers report the failure or the busy refusal in the form's own slot. The
// Test control of a form is a button, so it never closes the dialog.
document.body.addEventListener("htmx:afterRequest", function (evt) {
  const form = evt.detail && evt.detail.elt;
  if (!form || form.tagName !== "FORM" || !editDialog.contains(form)) return;
  if (!evt.detail.successful || answeredBusy(evt)) return;
  editDialog.close();
});
