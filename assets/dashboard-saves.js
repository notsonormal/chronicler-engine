// The controls that save on change: the rendered value of a control after its save, the "Saved" line, and the report of a refused save.

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

// A form that reports its own refusal in its own after-request handler, such as
// the new-options form of the dock, is not a saving form.
function isSavingControl(element) {
  if (element.hasAttribute("hx-on::after-request")) return false;
  return element.tagName === "FORM" || element.matches("input, textarea, select");
}

// A save that failed or was refused keeps the edit of a form, but puts a control
// that saves on change back to its rendered value, so the error in its slot does
// not contradict what the page shows. A save that went through shows "Saved" once
// the screen shows it.
function settleSave(evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element || !element.closest) return;
  if (!element.closest("[data-part], #edit-dialog")) return;
  if (!isSavingControl(element)) return;
  const refusal = answeredRefused(evt);
  const saved = evt.detail.successful && !refusal;
  if (saved) {
    settleSavedFields(element);
    afterScreenShows(evt.detail.xhr.getResponseHeader("X-Revision")).then(() =>
      flagSaved(element, true),
    );
    return;
  }
  flagSaved(element, false);
  if (element.tagName !== "FORM") restoreRenderedValue(element);
  else if (element.hasAttribute("data-autosave")) editFieldsOf(element).forEach(restoreRenderedValue);
  if (refusal) {
    const what = element.getAttribute("data-refusal-what") || "the change was not saved";
    reportRefusal(inlineErrorSlotFor(element, true), what, refusal);
  }
}

document.body.addEventListener("htmx:afterRequest", settleSave);
