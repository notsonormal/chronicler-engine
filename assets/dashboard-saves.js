// The controls that save on change: the rendered value of a control after its save, and the "Saved" line.

const FIELD_WITHOUT_VALUE = "input[type=hidden], input[type=button], input[type=submit]";

function editFieldsOf(element) {
  if (element.matches("input, textarea, select")) return [element];
  return Array.from(element.querySelectorAll("input, textarea, select"));
}

function settleField(field) {
  if (field.matches("input[type=checkbox], input[type=radio]")) {
    field.defaultChecked = field.checked;
  } else if (field.tagName === "SELECT") {
    Array.from(field.options).forEach((option) => {
      option.defaultSelected = option.selected;
    });
  } else if (!field.matches(FIELD_WITHOUT_VALUE)) {
    field.defaultValue = field.value;
  }
}

function restoreRenderedValue(field) {
  if (field.matches("input[type=checkbox], input[type=radio]")) {
    field.checked = field.defaultChecked;
  } else if (field.tagName === "SELECT") {
    const rendered = Array.from(field.options).findIndex(
      (option) => option.defaultSelected,
    );
    field.selectedIndex = rendered < 0 ? 0 : rendered;
  } else if (!field.matches(FIELD_WITHOUT_VALUE)) {
    field.value = field.defaultValue;
  }
}

// A save that went through makes the saved values the rendered ones, so a later
// failed save puts the control back to what the engine accepted.
function settleSavedFields(element) {
  editFieldsOf(element).forEach(settleField);
}

// The "Saved" line is view state: `data-saved` is in the `data-keep` of the
// status element, so a load keeps it.
function flagSaved(element, saved) {
  const id = element.getAttribute("data-saved-flag");
  const flag = id ? document.getElementById(id) : null;
  if (!flag) return;
  if (saved) flag.setAttribute("data-saved", "");
  else flag.removeAttribute("data-saved");
}

function isSavingControl(element) {
  return element.tagName === "FORM" || element.matches("input, textarea, select");
}

function settleSave(evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element || !element.closest) return;
  if (!element.closest("[data-part], #edit-dialog")) return;
  if (!isSavingControl(element)) return;
  const answer = requestAnswer(evt);
  if (answer.outcome === "done") {
    settleSavedFields(element);
    afterScreenShows(answer.stamp).then(() => flagSaved(element, true));
    return;
  }
  flagSaved(element, false);
  if (element.tagName !== "FORM") restoreRenderedValue(element);
  else if (element.hasAttribute("data-autosave")) editFieldsOf(element).forEach(restoreRenderedValue);
}

document.body.addEventListener("htmx:afterRequest", settleSave);

// A failed Update World must not sit under green feedback from an earlier
// auto-save of the posture selects in the same dialog.
function clearPostureStatus() {
  const status = document.getElementById("world-posture-status");
  if (status) status.removeAttribute("data-saved");
}
