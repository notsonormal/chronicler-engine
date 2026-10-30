// The screen: the ask loop, the screen load of the visible tab, the keep rules of a load, the stamp that says what the page shows, and the Game header of a request.

const DEFAULT_POLL_MS = 1000;
const DEFAULT_ASK_TIMEOUT_MS = 10000;
const GAMES_PAGE_KEY = "chronicle.gamesPage";
const GAME_HEADER = "X-Chronicler-Game";
const screenRoot = document.documentElement;

function pollIntervalMs() {
  const value = Number(screenRoot.dataset.pollMs);
  return value > 0 ? value : DEFAULT_POLL_MS;
}

// Without a limit, one ask that never ends keeps the next ask from starting: a
// request that gets no answer within this time is "No answer", the Unreachable
// banner.
function askTimeoutSignal() {
  const value = Number(screenRoot.dataset.askTimeoutMs);
  return AbortSignal.timeout(value > 0 ? value : DEFAULT_ASK_TIMEOUT_MS);
}

// The page of the saved Games is view state that survives a tab change and a
// reload, as the active tab and the Settings sub-tab do.
function rememberGamesPage(page) {
  try {
    sessionStorage.setItem(GAMES_PAGE_KEY, page);
  } catch (e) {
    // Without storage the page lasts until the page reloads.
  }
}

function restoreGamesPage() {
  try {
    const saved = sessionStorage.getItem(GAMES_PAGE_KEY);
    if (saved) screenRoot.dataset.gamesPage = saved;
  } catch (e) {
    // Without storage the Games tab opens on page 1.
  }
}

restoreGamesPage();

function fireScreenEvent(name, detail) {
  document.dispatchEvent(new CustomEvent(name, { detail: detail }));
}

function visibleSection() {
  return document.querySelector(".tab-content.active");
}

function visibleParts() {
  const section = visibleSection();
  return Array.from(document.querySelectorAll("[data-part]")).filter((part) => {
    const owner = part.closest(".tab-content");
    return !owner || owner === section;
  });
}

// View state lives in attributes that the server never renders, so an element
// lists them in `data-keep` and a morph keeps their values.
Idiomorph.defaults.callbacks.beforeAttributeUpdated = function (
  attributeName,
  node,
) {
  const keep = node.getAttribute ? node.getAttribute("data-keep") : null;
  if (keep && keep.split(/\s+/).includes(attributeName)) return false;
};

// The htmx class `htmx-request` is not enough, because the settle of a swap that
// lands during a request writes the old `class` back onto the element after the
// request ended: the page counts the requests itself.
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

// The end of a request leaves the screen stale in a way that the revision may not
// show: a load that skipped the control of the request recorded the stamp, and a
// request that wrote nothing moves no revision. So the end of every request that is
// not a read forgets the stamp.
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

// A load skips the entry in edit, and the skip counts as current: the engine's
// render of the entry is not wanted until the edit ends.
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

// The server id differs after a restart, and the revision of the old server means
// nothing to the new one, so such a stamp is shown only by a load that started
// after the wait began: an earlier load may still carry the old server.
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

// The engine answered with a failure status. A body that never arrives is
// "No answer", as in the load.
async function showScreenFailure(request, response) {
  let body;
  try {
    body = await response.text();
  } catch (error) {
    markUnreachable();
    return;
  }
  reportScreenError(
    request + " answered " + response.status + ". The screen was not updated.",
    plainResponseText(body),
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

// An answer that arrives after a tab change or a click on Previous or Next is
// dropped, and that change has already asked again.
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
    response = await fetch("/screen?" + query, {
      cache: "no-store",
      signal: askTimeoutSignal(),
    });
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
    response = await fetch("/changes", {
      cache: "no-store",
      signal: askTimeoutSignal(),
    });
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
    if (error.name === "TimeoutError") markUnreachable();
    else showScreenException("GET /changes", "could not be read", error);
    return;
  }
  if (answer.server + ":" + answer.revision === screenRoot.dataset.revision) {
    clearScreenError();
    return;
  }
  await loadScreen();
}

// An ask that has not started yet serves every caller that comes before it starts,
// because it starts after each of their requests has ended.
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

// A control that cannot move is `aria-disabled`, not `disabled`: it keeps keyboard
// focus through the morph.
document.addEventListener("click", function (event) {
  const control = event.target.closest('[data-part="games"] [data-page]');
  if (!control || control.getAttribute("aria-disabled") === "true") return;
  screenRoot.dataset.gamesPage = control.dataset.page;
  rememberGamesPage(control.dataset.page);
  forgetScreenStamp();
  askNow();
});

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

// The engine refuses a request from a page that shows a Game that is no longer
// active, so every request that is not a read names its Game, and a draft names the
// Game of the screen when the draft began.
// The preview forms sit outside #command-form, but they send the same draft.
function draftOfRequest(element) {
  if (element.closest("#action-preview")) return document.getElementById("command-form");
  return element.closest("[data-draft-game]");
}

document.body.addEventListener("htmx:configRequest", function (evt) {
  if (evt.detail.verb === "get") return;
  const draft = draftOfRequest(evt.detail.elt);
  const game =
    draft && draft.hasAttribute("data-draft-game")
      ? draft.dataset.draftGame
      : screenRoot.dataset.game;
  if (game) evt.detail.headers[GAME_HEADER] = game;
});
