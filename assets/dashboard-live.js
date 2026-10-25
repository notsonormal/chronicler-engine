// The live regions: the ask loop, the record and state of each region, the hold of an edit, the data-keep rule and the focus rule after a removal.

// A live region is an element with `data-live`, a space-separated list of the
// kinds of data it shows. The kinds are the names of the numbers of
// `GET /changes`. The page asks while it is visible. A region loads when a
// number of one of its kinds differs from the number it last loaded, and no
// region loads while the player edits in it.

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

// One ask at a time: an ask waits for the one before it.
function askNow() {
  asksPending += 1;
  const run = askChain
    .then(runAsk)
    .catch((error) => {
      console.error("The change ask failed:", error);
      return [];
    })
    .finally(() => {
      asksPending -= 1;
    });
  askChain = run;
  return run;
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

liveTimer = setInterval(liveTick, LIVE_ASK_INTERVAL_MS);

document.addEventListener("visibilitychange", function () {
  if (document.visibilityState === "visible") askNow();
});

// A region's own load: the one htmx runs at page load and each one the loop
// starts with `refresh`.
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
  if (!isLiveRegion(region)) return;
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

// A load that removes the focused node moves focus to the nearest row that
// survives, the one after it first, or else to the region.
function applyFocusPlan(region, plan) {
  if (!plan || plan.focused.isConnected) return;
  for (let distance = 1; distance < plan.rows.length; distance += 1) {
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
