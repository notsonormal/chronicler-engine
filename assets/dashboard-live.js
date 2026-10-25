// The live regions: the ask loop, the record and state of each region, the hold of an edit and the rule that ends it, the hidden-section rule, the data-keep rule and the focus rule after a removal.

// A live region is an element with `data-live`, a space-separated list of the
// kinds of data it shows. The kinds are the names of the numbers of
// `GET /changes`. The page asks while it is visible. A region loads when a
// number of one of its kinds differs from the number it last loaded. No region
// loads while the player edits in it, and a region in a hidden dashboard
// section waits until the player opens the section.

const LIVE_ASK_INTERVAL_MS = 1000;

// The region's own record: the numbers of its kinds at its last good load (null
// when nothing is known, so the region is due), the numbers of the ask that
// started the load in flight, and what that load did.
const liveRecords = new Map();
let lastAnswer = null;
let askChain = Promise.resolve();
let asksPending = 0;
let liveTimer = null;

function isLiveRegion(node) {
  return !!node && node.nodeType === 1 && node.hasAttribute("data-live");
}

function liveRecordOf(region) {
  let record = liveRecords.get(region);
  if (!record) {
    record = {
      numbers: null,
      pending: null,
      loading: false,
      requested: false,
      swapped: false,
      held: false,
      focusPlan: null,
      waiters: [],
    };
    liveRecords.set(region, record);
  }
  return record;
}

function setLiveState(region, state) {
  region.dataset.liveState = state;
}

function liveKinds(region) {
  return region.dataset.live.split(/\s+/).filter(Boolean);
}

function regionIsDue(region, answer) {
  const numbers = liveRecordOf(region).numbers;
  if (!numbers) return true;
  return liveKinds(region).some((kind) => numbers[kind] !== answer[kind]);
}

// An edit is the root of an open editor (`data-work`) or a field the player
// changed: its value or choice differs from the one the server rendered.
function fieldIsDirty(field) {
  if (field.matches("input[type=checkbox], input[type=radio]")) {
    return field.checked !== field.defaultChecked;
  }
  if (field.matches("input[type=hidden], input[type=button], input[type=submit]")) {
    return false;
  }
  if (field.tagName === "SELECT") {
    const options = Array.from(field.options);
    if (field.multiple) {
      return options.some((option) => option.selected !== option.defaultSelected);
    }
    const rendered = options.findIndex((option) => option.defaultSelected);
    return field.selectedIndex !== (rendered < 0 ? 0 : rendered);
  }
  return field.value !== field.defaultValue;
}

function regionHoldsEdit(region) {
  if (region.querySelector("[data-work]")) return true;
  return Array.from(region.querySelectorAll("input, textarea, select")).some(
    fieldIsDirty,
  );
}

// A region in a section that is not the active one waits: the header is outside
// every section, so it is never hidden.
function sectionIsHidden(region) {
  const section = region.closest(".tab-content");
  return !!section && !section.classList.contains("active");
}

// The fields that an edit covers: the control itself, or the fields of a form
// or of an editor root.
function editFieldsOf(element) {
  if (element.matches("input, textarea, select")) return [element];
  return Array.from(element.querySelectorAll("input, textarea, select"));
}

const FIELD_WITHOUT_VALUE = "input[type=hidden], input[type=button], input[type=submit]";

// The saved value is the rendered value now, so the field is no longer changed.
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

// A control that saves on change goes back to the value the server rendered,
// so a value that was not saved does not stay on screen or hold the region.
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

// The one rule that ends an edit after a save went through. `element` is the
// form or the control that saved. Each field of it takes its current value as
// its rendered value, so it is no longer changed. A form that creates something
// (`data-reset-on-save`) goes back to the empty form that the server rendered.
//
// Only a form ends the edit of its editor: the editor root that holds the region
// (`data-work`) lets go, and the disclosure that holds only this form closes (it
// marks itself with `data-close-on-save`). A control that saves on change, such
// as the World posture, leaves the editor open: the player is still editing.
function endEdit(element) {
  if (element.matches("form[data-reset-on-save]")) {
    element.reset();
  } else {
    editFieldsOf(element).forEach(settleField);
  }
  if (element.tagName !== "FORM") return;
  const root = element.closest("[data-work]");
  if (root) root.removeAttribute("data-work");
  const disclosure = element.closest("details[data-close-on-save]");
  if (disclosure) disclosure.open = false;
}

// The "Saved" line of an auto-save control is view state: the browser shows it
// after a save that went through, through an attribute that a load keeps
// (`data-saved` in the `data-keep` of the status element), and removes it after a
// save that did not.
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

// What happens to the edit of a form or control in a region when its request
// ends. A save that went through ends the edit. A save that failed or was
// refused keeps the edit of a form, and puts a control that saves on change
// back to its rendered value (D2), so the error in its slot is not a lie about
// what the page shows.
function settleSave(evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element || !element.closest || !element.closest("[data-live]")) return;
  if (!isSavingControl(element)) return;
  const refused = answeredBusy(evt);
  const saved = evt.detail.successful && !refused;
  flagSaved(element, saved);
  if (saved) {
    endEdit(element);
    return;
  }
  if (element.tagName === "FORM") {
    // A form that saves on change (`data-autosave`) is a control: its fields go
    // back to the rendered values. Any other form keeps the player's edit.
    if (element.hasAttribute("data-autosave")) editFieldsOf(element).forEach(restoreRenderedValue);
    if (refused) {
      const what = element.getAttribute("data-refusal-what") || "the change was not saved";
      reportBusyRefusal(inlineErrorSlotFor(element, true), what);
    }
    return;
  }
  restoreRenderedValue(element);
  if (refused) {
    reportBusyRefusal(inlineErrorSlotFor(element, true), "the change was not saved");
  }
}

function settleLoad(region, record) {
  record.loading = false;
  record.pending = null;
  const waiters = record.waiters;
  record.waiters = [];
  waiters.forEach((resolve) => resolve());
}

function startLoad(region, answer) {
  const record = liveRecordOf(region);
  record.loading = true;
  record.requested = false;
  record.swapped = false;
  record.held = false;
  record.pending = answer;
  setLiveState(region, "loading");
  const done = new Promise((resolve) => record.waiters.push(resolve));
  htmx.trigger(region, "refresh");
  if (!record.requested) {
    // htmx issued no request, so no afterRequest will end this load.
    setLiveState(region, "due");
    settleLoad(region, record);
  }
  return done;
}

async function runAsk() {
  let answer;
  try {
    const response = await fetch("/changes", { cache: "no-store" });
    if (!response.ok) throw new Error("GET /changes answered " + response.status);
    answer = await response.json();
  } catch (error) {
    markUnreachable();
    return [];
  }
  if (
    lastAnswer &&
    (answer.server !== lastAnswer.server || answer.game !== lastAnswer.game)
  ) {
    liveRecords.forEach((record) => {
      record.numbers = null;
    });
  }
  lastAnswer = answer;
  const loads = [];
  document.querySelectorAll("[data-live]").forEach((region) => {
    if (liveRecordOf(region).loading) return;
    if (!regionIsDue(region, answer)) return;
    if (sectionIsHidden(region)) {
      setLiveState(region, "stale");
      return;
    }
    if (regionHoldsEdit(region)) {
      setLiveState(region, "held");
      return;
    }
    loads.push(startLoad(region, answer));
  });
  // A region whose load keeps failing keeps its banner: the next load of it is
  // what clears the banner.
  const failing = Array.from(liveRecords.keys()).some(
    (region) => region.dataset.liveState === "due",
  );
  if (!failing) clearUnreachable();
  return loads;
}

// One ask at a time: an ask waits for the one before it. An ask that has not
// started yet serves every caller that comes before it starts, because it
// starts after each of their requests has ended.
let queuedAsk = null;

function askNow() {
  if (queuedAsk) return queuedAsk;
  asksPending += 1;
  const run = askChain
    .then(() => {
      queuedAsk = null;
      return runAsk();
    })
    .catch((error) => {
      console.error("The change ask failed:", error);
      return [];
    })
    .finally(() => {
      asksPending -= 1;
    });
  askChain = run;
  queuedAsk = run;
  return run;
}

// The one way a script sends an action: the page asks as soon as the request
// ends, so the result shows without waiting for the next ask of the timer.
function actionFetch(url, init) {
  return fetch(url, init).finally(() => {
    askNow();
  });
}

function liveTick() {
  if (document.visibilityState !== "visible") return Promise.resolve([]);
  if (asksPending > 0) return Promise.resolve([]);
  return askNow();
}

function loadsInFlight() {
  return Array.from(liveRecords.values()).some((record) => record.loading);
}

async function settleLive() {
  for (;;) {
    await askChain;
    if (asksPending === 0 && !loadsInFlight()) return;
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
}

// What the loop gives to a caller that needs one deterministic ask: the tests,
// and the actions that ask at once when their request ends.
window.chroniclerLive = {
  askNow: askNow,
  askOnce: async function () {
    const loads = await askNow();
    await Promise.all(loads);
  },
  settle: settleLive,
  tick: liveTick,
  stopTimer: function () {
    if (liveTimer !== null) clearInterval(liveTimer);
    liveTimer = null;
    return settleLive();
  },
};

// An edit that ends in a held region frees it, so the page asks and the region
// loads the data it waited for.
function liveWorkEnded() {
  return askNow();
}

// A section that opens shows its regions: the page asks, and each region of the
// section that is due loads.
function liveSectionOpened() {
  return askNow();
}

liveTimer = setInterval(liveTick, LIVE_ASK_INTERVAL_MS);

// The first ask of the page finds no record, so it loads every region. It waits
// until htmx has processed the page: a `refresh` sent before htmx initialises a
// node issues no request. htmx registers its own DOMContentLoaded handler in
// the head, so a handler registered here runs after it.
function startFirstAsk() {
  askNow();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", startFirstAsk);
} else {
  startFirstAsk();
}

document.addEventListener("visibilitychange", function () {
  if (document.visibilityState === "visible") askNow();
});

// A region's own load: each one the loop starts with `refresh`, and the one a
// caller starts through `htmx.ajax` with the region as its source.
document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const region = evt.detail && evt.detail.elt;
  if (!isLiveRegion(region)) return;
  const record = liveRecordOf(region);
  record.requested = true;
  if (record.loading) return;
  record.loading = true;
  record.swapped = false;
  record.held = false;
  record.pending = lastAnswer;
  setLiveState(region, "loading");
});

// This listener comes before the focus listener of dashboard-panels.js and the
// scroll listener of the story log, which both skip a swap that does not happen.
document.body.addEventListener("htmx:beforeSwap", function (evt) {
  const region = evt.detail && evt.detail.target;
  if (!isLiveRegion(region) || evt.detail.elt !== region) return;
  if (!evt.detail.shouldSwap) return;
  const record = liveRecordOf(region);
  if (regionHoldsEdit(region)) {
    evt.detail.shouldSwap = false;
    record.held = true;
    setLiveState(region, "held");
    return;
  }
  record.focusPlan = planFocusAfterLoad(region);
});

document.body.addEventListener("htmx:afterSwap", function (evt) {
  const region = evt.detail && (evt.detail.target || evt.detail.elt);
  if (!isLiveRegion(region)) return;
  const record = liveRecordOf(region);
  record.swapped = true;
  applyFocusPlan(region, record.focusPlan);
  record.focusPlan = null;
});

// The record of a region moves only when its load swapped, so a held, a failed
// and a refused load leave the region due.
document.body.addEventListener("htmx:afterRequest", function (evt) {
  const region = evt.detail && evt.detail.elt;
  // Every other request that ends is an action, or a view change that may
  // follow one: the edit of a saved form ends, and the page asks at once.
  if (!isLiveRegion(region)) {
    settleSave(evt);
    askNow();
    return;
  }
  const record = liveRecordOf(region);
  if (record.swapped) {
    record.numbers = record.pending;
    setLiveState(region, "current");
  } else if (record.held) {
    setLiveState(region, "held");
  } else {
    setLiveState(region, "due");
  }
  record.focusPlan = null;
  settleLoad(region, record);
});

// View state lives in attributes that the server never changes. An element
// lists them in `data-keep`, and a load keeps their values on the element it
// morphs.
Idiomorph.defaults.callbacks.beforeAttributeUpdated = function (
  attributeName,
  node,
) {
  const keep = node.getAttribute ? node.getAttribute("data-keep") : null;
  if (keep && keep.split(/\s+/).includes(attributeName)) return false;
};

// An inline error slot that shows an error is view state: a load neither
// rewrites it nor removes it. The next save that goes through clears it.
function showsInlineError(node) {
  return (
    !!node &&
    node.nodeType === 1 &&
    node.hasAttribute("data-error-slot") &&
    !node.hidden
  );
}

Idiomorph.defaults.callbacks.beforeNodeMorphed = function (oldNode) {
  if (showsInlineError(oldNode)) return false;
};

Idiomorph.defaults.callbacks.beforeNodeRemoved = function (node) {
  if (showsInlineError(node)) return false;
};

const LIVE_FOCUSABLE =
  "input:not([type=hidden]), select, textarea, button, a[href], [tabindex]";

// The row is the nearest ancestor of the focused node that has an id or a
// `data-id`, so morph keeps it by identity.
function rowOfFocused(focused, region) {
  let node = focused.parentElement;
  while (node && node !== region) {
    if (node.id || node.hasAttribute("data-id")) return node;
    node = node.parentElement;
  }
  return null;
}

function planFocusAfterLoad(region) {
  const focused = document.activeElement;
  if (!focused || focused === region || !region.contains(focused)) return null;
  const row = rowOfFocused(focused, region);
  const rows = row ? Array.from(row.parentElement.children) : [];
  return { focused: focused, rows: rows, index: rows.indexOf(row) };
}

function focusFirstControlIn(row) {
  for (const control of row.querySelectorAll(LIVE_FOCUSABLE)) {
    if (control.disabled) continue;
    control.focus();
    if (document.activeElement === control) return true;
  }
  return false;
}

// A load that removes the focused node moves focus to its own row when the row
// survives, else to the nearest row that does, the one after it first, or else to
// the region.
function applyFocusPlan(region, plan) {
  if (!plan || plan.focused.isConnected) return;
  for (let distance = 0; distance < plan.rows.length; distance += 1) {
    for (const index of [plan.index + distance, plan.index - distance]) {
      const row = plan.rows[index];
      if (row && row.isConnected && region.contains(row)) {
        if (focusFirstControlIn(row)) return;
      }
    }
  }
  if (!region.hasAttribute("tabindex")) region.setAttribute("tabindex", "-1");
  region.focus();
}

// An editor slot (`.editor-slot`) holds the editor that a view change loaded into
// it. Cancel closes the editor in the browser and sends no request: the slot is
// empty, as every load renders it, and the region that the editor held may load.
let editorOpener = null;

document.addEventListener("click", function (evt) {
  const target = evt.target;
  if (!(target instanceof Element)) return;
  const opener = target.closest("[data-opens-editor]");
  if (opener) editorOpener = opener;
  const cancel = target.closest("[data-close-editor]");
  if (!cancel) return;
  const slot = cancel.closest(".editor-slot");
  if (!slot) return;
  const region = slot.closest("[data-live]");
  slot.replaceChildren();
  if (region && editorOpener && editorOpener.isConnected && region.contains(editorOpener)) {
    editorOpener.focus();
  } else if (region) {
    if (!region.hasAttribute("tabindex")) region.setAttribute("tabindex", "-1");
    region.focus();
  }
  editorOpener = null;
  liveWorkEnded();
});
