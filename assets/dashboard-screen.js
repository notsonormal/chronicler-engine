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
// know: a request in flight or a story-log entry in edit. View state lives in
// attributes that the server never changes; an element lists them in
// `data-keep`, and a load keeps their values on the element it morphs.
Idiomorph.defaults.callbacks.beforeAttributeUpdated = function (
  attributeName,
  node,
) {
  const keep = node.getAttribute ? node.getAttribute("data-keep") : null;
  if (keep && keep.split(/\s+/).includes(attributeName)) return false;
};

// A control whose request is in flight holds a value that the engine has not
// seen, so a load must not overwrite it. The page counts the requests itself: the
// htmx class `htmx-request` is not enough, because the settle of a swap that lands
// during a request writes the old `class` back onto the element after the request
// ended.
const requestsInFlight = new Map();

function holdInFlight(node) {
  requestsInFlight.set(node, (requestsInFlight.get(node) || 0) + 1);
}

function releaseInFlight(node) {
  const count = requestsInFlight.get(node) || 0;
  if (count > 1) requestsInFlight.set(node, count - 1);
  else requestsInFlight.delete(node);
}

document.body.addEventListener("htmx:beforeRequest", function (evt) {
  const element = evt.detail && evt.detail.elt;
  if (element) holdInFlight(element);
});

// The end of an action leaves the screen stale in a way that the revision may not
// show: a load that skipped the control of the request recorded the stamp, and an
// action that wrote nothing moves no revision. So the end of every request that is
// not a read forgets the stamp, and the page asks at once. A swap that throws
// skips `htmx:afterRequest`, so `htmx:onLoadError` is the end of that request.
function endRequest(evt) {
  const element = evt.detail && evt.detail.elt;
  if (element && requestsInFlight.has(element)) releaseInFlight(element);
  const config = evt.detail && evt.detail.requestConfig;
  if (!config || config.verb !== "get") forgetScreenStamp();
  askNow();
}

document.body.addEventListener("htmx:afterRequest", endRequest);
document.body.addEventListener("htmx:onLoadError", endRequest);

function requestIsInFlight(node) {
  return !!node && node.nodeType === 1 && requestsInFlight.has(node);
}

// A story-log entry in edit holds the text that the player is typing. A load skips
// it, and the skip counts as current: the engine's render of the entry is not
// wanted until the edit ends, and the end of the edit forgets the stamp.
function isBeingEdited(node) {
  return !!node && node.nodeType === 1 && node.hasAttribute("data-editing");
}

function loadKeeps(node) {
  return isBeingEdited(node) || requestIsInFlight(node);
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
}

function splitStamp(stamp) {
  const at = stamp.lastIndexOf(":");
  return { server: stamp.slice(0, at), revision: Number(stamp.slice(at + 1)) };
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
// first good load that started after the wait began: a load that started earlier
// may still carry the old server. Each load takes the next number when it starts,
// and each wait records the number of the last load that had started.
let loadsStarted = 0;
const screenWaiters = [];

function afterScreenShows(stamp) {
  if (!stamp) return Promise.resolve();
  if (screenShows(stamp)) return Promise.resolve();
  return new Promise((resolve) =>
    screenWaiters.push({ stamp: stamp, loadsStartedBefore: loadsStarted, resolve: resolve }),
  );
}

function resolveScreenWaiters(loadNumber) {
  const shown = screenRoot.dataset.revision;
  const waiting = screenWaiters.splice(0);
  for (const waiter of waiting) {
    const restarted =
      !!shown &&
      loadNumber > waiter.loadsStartedBefore &&
      splitStamp(shown).server !== splitStamp(waiter.stamp).server;
    if (screenShows(waiter.stamp) || restarted) waiter.resolve();
    else screenWaiters.push(waiter);
  }
}

function reportScreenError(summary, detail) {
  showNotice(summary, detail, "screen");
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
  const place = noticeElement();
  if (place && place.dataset.source === "screen") clearNotice();
}

// A load is for the tab that was visible and the page of the saved Games that was
// asked for when it started. An answer that arrives after a tab change or a click
// on Previous or Next is dropped, and that change has already asked again.
async function loadScreen() {
  const section = visibleSection();
  const gamesPage = screenRoot.dataset.gamesPage;
  const parts = visibleParts();
  const names = parts.map((part) => part.dataset.part);
  const number = (loadsStarted += 1);
  let response;
  let body;
  try {
    const query =
      "parts=" + names.join(",") + (gamesPage ? "&games_page=" + gamesPage : "");
    response = await fetch("/screen?" + query, { cache: "no-store" });
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
  if (visibleSection() !== section || screenRoot.dataset.gamesPage !== gamesPage) return;
  try {
    morphParts(parts, body, response, number);
  } catch (error) {
    showScreenException("GET /screen", "could not be shown", error);
  }
}

function morphParts(parts, body, response, loadNumber) {
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
  screenRoot.dataset.game = response.headers.get("X-Game") || "";
  const stamp = response.headers.get("X-Revision");
  if (stamp) screenRoot.dataset.revision = stamp;
  clearScreenError();
  fireScreenEvent("screen:loaded", { parts: names });
  resolveScreenWaiters(loadNumber);
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

// The page of the saved Games is view state of the Games tab: a Previous or Next
// control carries the page that it goes to, and the loads ask for it until the
// player leaves the tab. A control that cannot move is `aria-disabled`, not
// `disabled`: it keeps keyboard focus through the morph.
document.addEventListener("click", function (event) {
  const control = event.target.closest('[data-part="games"] [data-page]');
  if (!control || control.getAttribute("aria-disabled") === "true") return;
  screenRoot.dataset.gamesPage = control.dataset.page;
  forgetScreenStamp();
  askNow();
});

// A tab change empties each part of the tab that closes, and the tab that opens
// loads at once.
function leaveSection(section) {
  if (!section) return;
  delete screenRoot.dataset.gamesPage;
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
