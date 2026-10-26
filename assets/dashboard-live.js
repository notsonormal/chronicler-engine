// The live regions: the ask loop, the record and state of each region, the hidden-section rule, the load rules (data-keep, an inline error, a control in flight, a story-log entry in edit) and the focus rule after a removal.

// A live region is an element with `data-live`, a space-separated list of the
// kinds of data it shows. The kinds are the names of the numbers of
// `GET /changes`. The page asks while it is visible. A region loads when a
// number of one of its kinds differs from the number it last loaded. A region in
// a hidden dashboard section waits until the player opens the section. No region
// holds a draft: a form that waits for a Save is in the edit dialog, and a load
// skips the one element that holds unsaved input.

const LIVE_ASK_INTERVAL_MS = 1000;

const FIELD_WITHOUT_VALUE = "input[type=hidden], input[type=button], input[type=submit]";

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
      skippedInFlight: false,
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

// The header is outside every section, so it is never hidden.
function sectionIsHidden(region) {
  const section = region.closest(".tab-content");
  return !!section && !section.classList.contains("active");
}

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

// A value that was not saved must not stay on screen.
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

// A save that failed or was refused keeps the edit of a form, but puts a control
// that saves on change back to its rendered value, so the error in its slot does
// not contradict what the page shows.
function settleSave(evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element || !element.closest) return;
  if (!element.closest("[data-live], #edit-dialog")) return;
  if (!isSavingControl(element)) return;
  const refusal = answeredRefused(evt);
  const saved = evt.detail.successful && !refusal;
  flagSaved(element, saved);
  if (saved) {
    settleSavedFields(element);
    return;
  }
  // A form that saves on change (`data-autosave`) is a control: its fields go
  // back to the rendered values. Any other form keeps the player's edit.
  if (element.tagName !== "FORM") restoreRenderedValue(element);
  else if (element.hasAttribute("data-autosave")) editFieldsOf(element).forEach(restoreRenderedValue);
  if (refusal) {
    const what = element.getAttribute("data-refusal-what") || "the change was not saved";
    reportRefusal(inlineErrorSlotFor(element, true), what, refusal);
  }
}

function beginLoad(region, record, answer) {
  record.loading = true;
  record.swapped = false;
  record.skippedInFlight = false;
  record.pending = answer;
  setLiveState(region, "loading");
}

function settleLoad(record) {
  record.loading = false;
  record.pending = null;
  const waiters = record.waiters;
  record.waiters = [];
  waiters.forEach((resolve) => resolve());
  recordShownGame();
}

function startLoad(region, answer) {
  const record = liveRecordOf(region);
  record.requested = false;
  beginLoad(region, record, answer);
  const done = new Promise((resolve) => record.waiters.push(resolve));
  htmx.trigger(region, "refresh");
  if (!record.requested) {
    // htmx issued no request, so no afterRequest will end this load.
    setLiveState(region, "due");
    settleLoad(record);
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
    loads.push(startLoad(region, answer));
  });
  // A region whose load keeps failing keeps its banner: the next load of it is
  // what clears the banner.
  const failing = Array.from(liveRecords.keys()).some(
    (region) => region.dataset.liveState === "due",
  );
  if (!failing) clearUnreachable();
  Promise.all(loads).then(recordShownGame);
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

// The Game the page shows, for the Game header of an action. The page records the
// Game of the last answer only when every visible region shows that answer: none is
// loading, due or behind it. A hidden region does not block, because the player
// does not see it. A page that has not recorded a Game names none, and its action
// acts on the active Game.
const GAME_HEADER = "X-Chronicler-Game";

function visibleRegionsShow(answer) {
  return Array.from(document.querySelectorAll("[data-live]")).every((region) => {
    if (sectionIsHidden(region)) return true;
    return (
      !liveRecordOf(region).loading &&
      region.dataset.liveState !== "due" &&
      !regionIsDue(region, answer)
    );
  });
}

function recordShownGame() {
  if (!lastAnswer || !visibleRegionsShow(lastAnswer)) return;
  document.documentElement.dataset.game =
    lastAnswer.game === null ? "" : String(lastAnswer.game);
}

// A create, a switch or a Reset that this tab sent makes the shown Game stale at
// once, so the page names none until the next ask has loaded the new Game: the
// next send acts on the active Game and meets no false refusal.
const GAME_CHANGING_ROUTES = [/^\/games$/, /^\/games\/[^/]+\/switch$/, /^\/reset$/];

function forgetShownGameAfterGameChange(evt) {
  const path = evt.detail.pathInfo && evt.detail.pathInfo.requestPath;
  if (!evt.detail.successful || !path) return;
  const route = path.split("?")[0];
  if (GAME_CHANGING_ROUTES.some((pattern) => pattern.test(route))) {
    delete document.documentElement.dataset.game;
  }
}

document.body.addEventListener("htmx:configRequest", function (evt) {
  const game = document.documentElement.dataset.game;
  if (game && evt.detail.verb !== "get") evt.detail.headers[GAME_HEADER] = game;
});

// The one way a script sends an action: the page asks as soon as the request
// ends, so the result shows without waiting for the next ask of the timer.
function actionFetch(url, init) {
  const headers = new Headers((init && init.headers) || {});
  const game = document.documentElement.dataset.game;
  if (game) headers.set(GAME_HEADER, game);
  return fetch(url, { ...init, headers }).finally(() => {
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

// The hooks of the loop. `settle` is for a script that needs the result of an
// ask on screen before it moves focus. The rest is the test API: the stub
// browser tests have no other way to make one ask, stop the timer, run one tick
// of it, or wait for the loop to go quiet, without sleeping. They ship in
// production because the tests run against the same file.
window.chroniclerLive = {
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

liveTimer = setInterval(liveTick, LIVE_ASK_INTERVAL_MS);

// The first ask of the page finds no record, so it loads every region. It waits
// until htmx has processed the page: a `refresh` sent before htmx initialises a
// node issues no request. htmx registers its own DOMContentLoaded handler in
// the head, so a handler registered here runs after it.
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => askNow());
} else {
  askNow();
}

document.addEventListener("visibilitychange", function () {
  if (document.visibilityState === "visible") askNow();
});

// A load that the loop did not start (an `htmx.ajax` call with the region as its
// source) begins its record here.
document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const region = evt.detail && evt.detail.elt;
  if (!isLiveRegion(region)) return;
  const record = liveRecordOf(region);
  record.requested = true;
  if (!record.loading) beginLoad(region, record, lastAnswer);
});

// This listener comes before the focus listener of dashboard-panels.js and the
// scroll listener of the story log, which both skip a swap that does not happen.
document.body.addEventListener("htmx:beforeSwap", function (evt) {
  const region = evt.detail && evt.detail.target;
  if (!isLiveRegion(region) || evt.detail.elt !== region) return;
  if (!evt.detail.shouldSwap) return;
  liveRecordOf(region).focusPlan = planFocusAfterLoad(region);
});

document.body.addEventListener("htmx:afterSwap", function (evt) {
  const region = evt.detail && (evt.detail.target || evt.detail.elt);
  if (!isLiveRegion(region)) return;
  const record = liveRecordOf(region);
  record.swapped = true;
  applyFocusPlan(region, record.focusPlan);
  record.focusPlan = null;
});

// The record of a region moves only when its load swapped, so a failed and a
// refused load leave the region due.
document.body.addEventListener("htmx:afterRequest", function (evt) {
  const region = evt.detail && evt.detail.elt;
  // Every other request that ends is an action, or a view change that may
  // follow one: the edit of a saved form ends, and the page asks at once.
  if (!isLiveRegion(region)) {
    settleSave(evt);
    forgetShownGameAfterGameChange(evt);
    askNow();
    return;
  }
  const record = liveRecordOf(region);
  // A load that skipped a control in flight shows what the control showed, not
  // what the engine rendered, so it must not stand for the numbers it saw.
  // A load that skipped the entry in edit counts as current: the entry shows
  // the draft until the edit ends, and `loadRegionAgain` then makes the region
  // due.
  const staleBySkip = record.swapped && record.skippedInFlight;
  if (record.swapped && !staleBySkip) {
    record.numbers = record.pending;
    setLiveState(region, "current");
  } else {
    setLiveState(region, "due");
  }
  record.focusPlan = null;
  settleLoad(record);
  // While the request is in flight, its own end asks again. A request that ended
  // before this load did has asked already, and found the region loading.
  if (staleBySkip && !regionHasRequestInFlight(region)) askNow();
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

// A control whose request is in flight holds a value that the engine has not
// seen, so a load must not overwrite it. The next load, after the request ends,
// sets the rendered value. The page counts the requests itself: the htmx class
// `htmx-request` is not enough, because the settle of a swap that lands during a
// request writes the old `class` back onto the element after the request ended.
const requestsInFlight = new Map();

document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const element = evt.detail && evt.detail.elt;
  if (element) requestsInFlight.set(element, (requestsInFlight.get(element) || 0) + 1);
});

document.body.addEventListener("htmx:afterRequest", function (evt) {
  const element = evt.detail && evt.detail.elt;
  const count = element ? requestsInFlight.get(element) || 0 : 0;
  if (count > 1) requestsInFlight.set(element, count - 1);
  else requestsInFlight.delete(element);
});

function requestIsInFlight(node) {
  return !!node && node.nodeType === 1 && requestsInFlight.has(node);
}

// The entry that a load skipped shows the draft, not the engine's render. When
// the edit ends, the region forgets its numbers, so the next ask loads it.
function loadRegionAgain(region) {
  if (region) liveRecordOf(region).numbers = null;
}

function regionHasRequestInFlight(region) {
  return Array.from(requestsInFlight.keys()).some(
    (element) => element !== region && region.contains(element),
  );
}

function noteSkippedInFlight(node) {
  const region = node.closest("[data-live]");
  if (region && region !== node) liveRecordOf(region).skippedInFlight = true;
}

// A story-log entry in edit holds the text that the player is typing. A load
// skips it, and the skip counts as current: the engine's render of the entry
// is not wanted until the edit ends.
function isBeingEdited(node) {
  return !!node && node.nodeType === 1 && node.hasAttribute("data-editing");
}

function loadKeeps(node) {
  if (isBeingEdited(node)) return true;
  if (requestIsInFlight(node)) {
    noteSkippedInFlight(node);
    return true;
  }
  return showsInlineError(node);
}

Idiomorph.defaults.callbacks.beforeNodeMorphed = function (oldNode) {
  if (loadKeeps(oldNode)) return false;
};

Idiomorph.defaults.callbacks.beforeNodeRemoved = function (node) {
  if (loadKeeps(node)) return false;
};

const LIVE_FOCUSABLE =
  "input:not([type=hidden]), select, textarea, button, a[href], [tabindex]";

// Morph keeps a row with an id or a `data-id` by identity.
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
