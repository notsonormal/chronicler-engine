// The screen: the ask loop, the screen load of the visible tab, the keep rules of a load, the stamp that says what the page shows, and the Game header of an action.

const DEFAULT_POLL_MS = 1000;
const GAME_HEADER = "X-Chronicler-Game";
const screenRoot = document.documentElement;

function pollIntervalMs() {
  const value = Number(screenRoot.dataset.pollMs);
  return value > 0 ? value : DEFAULT_POLL_MS;
}

function fireScreenEvent(name, detail) {
  document.dispatchEvent(new CustomEvent(name, { detail: detail }));
}

function visibleSection() {
  return document.querySelector(".tab-content.active");
}

// The header and the failure banner are outside every section, so they always
// load. The other parts load while their section is the visible one.
function visibleParts() {
  const section = visibleSection();
  return Array.from(document.querySelectorAll("[data-part]")).filter((part) => {
    const owner = part.closest(".tab-content");
    return !owner || owner === section;
  });
}

// A part is not written while its element holds a state that the server does not
// know: a request in flight, a story-log entry in edit, or an inline error that
// shows. View state lives in attributes that the server never changes; an element
// lists them in `data-keep`, and a load keeps their values on the element it
// morphs.
Idiomorph.defaults.callbacks.beforeAttributeUpdated = function (
  attributeName,
  node,
) {
  const keep = node.getAttribute ? node.getAttribute("data-keep") : null;
  if (keep && keep.split(/\s+/).includes(attributeName)) return false;
};

function showsInlineError(node) {
  return (
    !!node &&
    node.nodeType === 1 &&
    node.hasAttribute("data-error-slot") &&
    !node.hidden
  );
}

// A control whose request is in flight holds a value that the engine has not
// seen, so a load must not overwrite it. The page counts the requests itself: the
// htmx class `htmx-request` is not enough, because the settle of a swap that lands
// during a request writes the old `class` back onto the element after the request
// ended. A request also holds the controls that its `hx-disabled-elt` names: htmx
// clears their `disabled` when the request ends, whatever the server rendered
// there meanwhile. A load leaves them alone, and the end of the request asks for
// the load that shows them.
const requestsInFlight = new Map();
const disabledByRequest = new WeakMap();

function holdInFlight(node) {
  requestsInFlight.set(node, (requestsInFlight.get(node) || 0) + 1);
}

function releaseInFlight(node) {
  const count = requestsInFlight.get(node) || 0;
  if (count > 1) requestsInFlight.set(node, count - 1);
  else requestsInFlight.delete(node);
}

// The controls that htmx disables for a request: the elements that the
// `hx-disabled-elt` of the request's element, or of its closest ancestor that has
// one, selects. This reads plain CSS selectors only. htmx also takes `this`,
// `closest …` and `find …`, which `querySelectorAll` does not read, so the shell
// names its controls with plain selectors.
function controlsDisabledBy(element) {
  const holder = element.closest("[hx-disabled-elt]");
  if (!holder) return [];
  return Array.from(document.querySelectorAll(holder.getAttribute("hx-disabled-elt")));
}

document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const element = evt.detail && evt.detail.elt;
  if (!element) return;
  holdInFlight(element);
  const controls = controlsDisabledBy(element);
  controls.forEach(holdInFlight);
  disabledByRequest.set(evt.detail.xhr, {
    controls: controls,
    renderedDisabled: controls.some((control) => control.hasAttribute("disabled")),
  });
});

function endRequest(evt) {
  const element = evt.detail && evt.detail.elt;
  if (element && requestsInFlight.has(element)) releaseInFlight(element);
  const held = disabledByRequest.get(evt.detail.xhr);
  if (!held) return;
  disabledByRequest.delete(evt.detail.xhr);
  held.controls.forEach(releaseInFlight);
  // A control that the server rendered as disabled shows enabled once htmx
  // clears it, so the screen must load again to render it as the server does.
  if (held.renderedDisabled) forgetScreenStamp();
}

document.body.addEventListener("htmx:afterRequest", endRequest);

// A swap that throws skips `htmx:afterRequest` and leaves `disabled` on the
// controls of the request. The request is over all the same, so the loads stop
// waiting for it, and the next load renders the controls of the parts again.
document.body.addEventListener("htmx:onLoadError", function (evt) {
  endRequest(evt);
  forgetScreenStamp();
});

function requestIsInFlight(node) {
  return !!node && node.nodeType === 1 && requestsInFlight.has(node);
}

// An element that left the page can end its request without an event that
// reaches the body, so only a request of an element on the page counts.
function aRequestOfThePageIsInFlight() {
  for (const element of requestsInFlight.keys()) {
    if (element.isConnected) return true;
  }
  return false;
}

// A story-log entry in edit holds the text that the player is typing. A load skips
// it, and the skip counts as current: the engine's render of the entry is not
// wanted until the edit ends, and the end of the edit forgets the stamp.
function isBeingEdited(node) {
  return !!node && node.nodeType === 1 && node.hasAttribute("data-editing");
}

// A load that skipped a control in flight shows what the control showed, not what
// the engine rendered, so it must not stand for the revision it saw.
let skippedInFlightInLoad = false;

// While the request that a load skipped runs, another load would skip it again and
// write nothing new, so the asks wait. The end of the request asks.
let loadWaitsForRequests = false;

function loadKeeps(node) {
  if (isBeingEdited(node)) return true;
  if (requestIsInFlight(node)) {
    skippedInFlightInLoad = true;
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

const SCREEN_FOCUSABLE =
  "input:not([type=hidden]), select, textarea, button, a[href], [tabindex]";

// Morph keeps a row with an id or a `data-id` by identity.
function rowOfFocused(focused, part) {
  let node = focused.parentElement;
  while (node && node !== part) {
    if (node.id || node.hasAttribute("data-id")) return node;
    node = node.parentElement;
  }
  return null;
}

function planFocusAfterLoad(part) {
  const focused = document.activeElement;
  if (!focused || focused === part || !part.contains(focused)) return null;
  const row = rowOfFocused(focused, part);
  const rows = row ? Array.from(row.parentElement.children) : [];
  return { focused: focused, rows: rows, index: rows.indexOf(row) };
}

function focusFirstControlIn(row) {
  for (const control of row.querySelectorAll(SCREEN_FOCUSABLE)) {
    if (control.disabled) continue;
    control.focus();
    if (document.activeElement === control) return true;
  }
  return false;
}

// A load that removes the focused node moves focus to its own row when the row
// survives, else to the nearest row that does, the one after it first, or else to
// the part.
function applyFocusPlan(part, plan) {
  if (!plan || plan.focused.isConnected) return;
  for (let distance = 0; distance < plan.rows.length; distance += 1) {
    for (const index of [plan.index + distance, plan.index - distance]) {
      const row = plan.rows[index];
      if (row && row.isConnected && part.contains(row)) {
        if (focusFirstControlIn(row)) return;
      }
    }
  }
  if (!part.hasAttribute("tabindex")) part.setAttribute("tabindex", "-1");
  part.focus();
}

// The stamp `<server>:<revision>` of the page is `<html data-revision>`. A page
// that shows nothing yet, or whose last load must be repeated, has none.
function forgetScreenStamp() {
  delete screenRoot.dataset.revision;
  loadWaitsForRequests = false;
}

function splitStamp(stamp) {
  const at = stamp.lastIndexOf(":");
  return { server: stamp.slice(0, at), revision: Number(stamp.slice(at + 1)) };
}

// The servers that the page has met, oldest first. A restart makes a newer server,
// so a stamp of a server that is newer than the server of a wait ends that wait.
const serversMet = [];

function meetServer(stamp) {
  const server = splitStamp(stamp).server;
  if (!serversMet.includes(server)) serversMet.push(server);
}

function screenShows(stamp) {
  const shown = screenRoot.dataset.revision;
  if (!shown) return false;
  const have = splitStamp(shown);
  const want = splitStamp(stamp);
  return have.server === want.server && have.revision >= want.revision;
}

// An answer that carries the revision after its write is shown when the screen
// holds that revision. The server id differs after a restart, and the revision of
// the old server means nothing to the new one, so such a stamp is shown by the
// next good load.
const screenWaiters = [];

function afterScreenShows(stamp) {
  if (!stamp) return Promise.resolve();
  if (screenShows(stamp)) return Promise.resolve();
  meetServer(stamp);
  return new Promise((resolve) => screenWaiters.push({ stamp: stamp, resolve: resolve }));
}

function resolveScreenWaiters() {
  const shown = screenRoot.dataset.revision;
  const waiting = screenWaiters.splice(0);
  for (const waiter of waiting) {
    const restarted =
      !!shown &&
      serversMet.indexOf(splitStamp(shown).server) >
        serversMet.indexOf(splitStamp(waiter.stamp).server);
    if (screenShows(waiter.stamp) || restarted) waiter.resolve();
    else screenWaiters.push(waiter);
  }
}

function screenErrorSlot() {
  return document.getElementById("screen-error");
}

// An answer that the page cannot use shows in one shell slot, and the parts keep
// their content. The dialog is modal, so it also hears the summary: the slot is
// behind it. An empty summary says that the slot is clear.
function reportScreenError(summary, detail) {
  renderInlineError(screenErrorSlot(), summary, detail);
  fireScreenEvent("screen:error", { summary: summary });
}

// The engine answered, so it is reachable.
async function showScreenFailure(request, response) {
  const body = await response.text();
  reportScreenError(
    request + " answered " + response.status + ". The screen was not updated.",
    plainResponseText(body) || "No detail.",
  );
}

function showScreenException(request, outcome, error) {
  console.error(request + " " + outcome + ":", error);
  reportScreenError(
    request + " " + outcome + ". The screen was not updated.",
    String((error && error.message) || error),
  );
}

function clearScreenError() {
  const slot = screenErrorSlot();
  if (!slot || slot.hidden) return;
  clearInlineError(slot);
  fireScreenEvent("screen:error", { summary: "" });
}

// The count of Game changes that this page sent. A load that started before one
// of them must not name its Game.
let gameChanges = 0;

// A load is for the tab that was visible when it started. An answer that arrives
// after a tab change is dropped, and the tab change has already asked for the
// visible tab.
async function loadScreen() {
  const section = visibleSection();
  const parts = visibleParts();
  const names = parts.map((part) => part.dataset.part);
  skippedInFlightInLoad = false;
  const gameChangesAtStart = gameChanges;
  fireScreenEvent("screen:beforeload", { parts: names });
  let response;
  let body;
  try {
    response = await fetch("/screen?parts=" + names.join(","), { cache: "no-store" });
    clearUnreachable();
    if (!response.ok) {
      await showScreenFailure("GET /screen", response);
      return;
    }
    body = await response.text();
  } catch (error) {
    markUnreachable();
    return;
  }
  if (visibleSection() !== section) return;
  try {
    morphParts(parts, body, response, gameChangesAtStart);
  } catch (error) {
    showScreenException("GET /screen", "could not be shown", error);
  }
}

function morphParts(parts, body, response, gameChangesAtStart) {
  const answer = document.createElement("template");
  answer.innerHTML = body;
  const templates = new Map(
    Array.from(answer.content.children)
      .filter((child) => child.matches("template[data-part]"))
      .map((child) => [child.dataset.part, child]),
  );
  const written = parts.filter((part) => templates.has(part.dataset.part));
  const names = written.map((part) => part.dataset.part);
  fireScreenEvent("screen:beforemorph", { parts: names });
  const plans = written.map((part) => planFocusAfterLoad(part));
  written.forEach((part) => {
    Idiomorph.morph(part, templates.get(part.dataset.part).innerHTML, {
      morphStyle: "innerHTML",
    });
    htmx.process(part);
  });
  written.forEach((part, index) => applyFocusPlan(part, plans[index]));
  // A Game change that this page sent while the load ran makes the Game of the
  // answer the old one.
  if (gameChanges === gameChangesAtStart) {
    screenRoot.dataset.game = response.headers.get("X-Game") || "";
  }
  const stamp = response.headers.get("X-Revision");
  loadWaitsForRequests = skippedInFlightInLoad;
  if (stamp && !skippedInFlightInLoad) {
    screenRoot.dataset.revision = stamp;
    meetServer(stamp);
  }
  clearScreenError();
  fireScreenEvent("screen:loaded", { parts: names });
  resolveScreenWaiters();
}

async function runAsk() {
  let response;
  try {
    response = await fetch("/changes", { cache: "no-store" });
  } catch (error) {
    markUnreachable();
    return;
  }
  clearUnreachable();
  if (!response.ok) {
    await showScreenFailure("GET /changes", response);
    return;
  }
  let answer;
  try {
    answer = await response.json();
  } catch (error) {
    showScreenException("GET /changes", "could not be read", error);
    return;
  }
  if (answer.server + ":" + answer.revision === screenRoot.dataset.revision) {
    clearScreenError();
    return;
  }
  if (loadWaitsForRequests && aRequestOfThePageIsInFlight()) return;
  await loadScreen();
}

// One ask at a time: an ask waits for the one before it. An ask that has not
// started yet serves every caller that comes before it starts, because it starts
// after each of their requests has ended.
let askChain = Promise.resolve();
let asksPending = 0;
let queuedAsk = null;

function markScreenBusy() {
  if (asksPending > 0) screenRoot.setAttribute("data-screen-busy", "");
  else screenRoot.removeAttribute("data-screen-busy");
}

function askNow() {
  if (queuedAsk) return queuedAsk;
  asksPending += 1;
  markScreenBusy();
  const run = askChain
    .then(() => {
      queuedAsk = null;
      return runAsk();
    })
    .catch((error) => {
      console.error("The screen ask failed:", error);
    })
    .finally(() => {
      asksPending -= 1;
      markScreenBusy();
    });
  askChain = run;
  queuedAsk = run;
  return run;
}

function screenTick() {
  if (document.visibilityState === "visible" && asksPending === 0) askNow();
  setTimeout(screenTick, pollIntervalMs());
}

setTimeout(screenTick, pollIntervalMs());

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => askNow());
} else {
  askNow();
}

document.addEventListener("visibilitychange", function () {
  if (document.visibilityState === "visible") askNow();
});

// A tab change empties each part of the tab that closes, and the tab that opens
// loads at once.
function leaveSection(section) {
  if (!section) return;
  const cleared = [];
  section.querySelectorAll("[data-part]").forEach((part) => {
    part.replaceChildren();
    cleared.push(part.dataset.part);
  });
  forgetScreenStamp();
  fireScreenEvent("screen:cleared", { parts: cleared });
}

// The Game of the screen goes with every action that is not a read, so the engine
// can refuse an action from a page that shows a Game that is no longer active. A
// page that shows no Game names none, and its action acts on the active Game.
document.body.addEventListener("htmx:configRequest", function (evt) {
  const game = screenRoot.dataset.game;
  if (game && evt.detail.verb !== "get") evt.detail.headers[GAME_HEADER] = game;
});

// A create, a switch or a Reset that this tab sent makes the shown Game stale at
// once, so the page names none until the next load has shown the new Game: the
// next send acts on the active Game and meets no false refusal.
const GAME_CHANGING_ROUTES = [/^\/games$/, /^\/games\/[^/]+\/switch$/, /^\/reset$/];

function forgetShownGameAfterGameChange(evt) {
  const path = evt.detail.pathInfo && evt.detail.pathInfo.requestPath;
  if (!evt.detail.successful || !path) return;
  const route = path.split("?")[0];
  if (GAME_CHANGING_ROUTES.some((pattern) => pattern.test(route))) {
    gameChanges += 1;
    delete screenRoot.dataset.game;
  }
}

// The one way a script sends an action: the page asks as soon as the request
// ends, so the result shows without waiting for the next ask of the timer.
function actionFetch(url, init) {
  const headers = new Headers((init && init.headers) || {});
  const game = screenRoot.dataset.game;
  if (game) headers.set(GAME_HEADER, game);
  return fetch(url, { ...init, headers }).finally(() => {
    askNow();
  });
}

// Every request that ends is an action, or a view change that may follow one, and
// the page asks at once.
document.body.addEventListener("htmx:afterRequest", function (evt) {
  forgetShownGameAfterGameChange(evt);
  askNow();
});
