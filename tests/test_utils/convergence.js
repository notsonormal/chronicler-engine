// The body of the convergence check, run inside the page by `convergence.rs`: it waits until the page is quiet, then compares each visible part with a fresh GET /screen of that part, checks that the parts of a closed tab are empty, and compares the attributes of each part element with the shell.
async ({ timeoutMs }) => {
  const HTMX_CLASSES = ["htmx-request", "htmx-settling", "htmx-swapping", "htmx-added"];
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const quote = (text) => JSON.stringify(text.length > 70 ? text.slice(0, 70) + "…" : text);

  const partName = (part) => part.dataset.part;

  const visibleParts = () =>
    Array.from(document.querySelectorAll("[data-part]")).filter((part) => {
      const section = part.closest(".tab-content");
      return !section || section.classList.contains("active");
    });

  // The page is quiet when no ask or load is in flight and the stamp of the page
  // equals the `server:revision` of GET /changes.
  async function reasonsNotQuiet() {
    const root = document.documentElement;
    if (root.hasAttribute("data-screen-busy")) return ["an ask or a load is in flight"];
    let answer;
    try {
      const response = await fetch("/changes", { cache: "no-store" });
      if (!response.ok) return ["GET /changes answered " + response.status];
      answer = await response.json();
    } catch (error) {
      return ["GET /changes failed: " + error.message];
    }
    const stamp = answer.server + ":" + answer.revision;
    if (root.dataset.revision !== stamp) {
      return ["the page shows " + quote(String(root.dataset.revision)) + ", the server is at " + stamp];
    }
    return [];
  }

  // A load keeps these three kinds of state (`Idiomorph.defaults.callbacks` in
  // dashboard-screen.js): the attributes that the element's `data-keep` names, an
  // inline error slot that shows an error, and a story-log entry in edit
  // (`data-editing`), which it skips.
  const showsInlineError = (node) =>
    node.nodeType === 1 && node.hasAttribute("data-error-slot") && !node.hidden;

  function comparableAttributes(element) {
    const keep = (element.getAttribute("data-keep") || "").split(/\s+/);
    const entries = [];
    for (const { name, value } of Array.from(element.attributes)) {
      if (keep.includes(name)) continue;
      if (name === "class") {
        const classes = value.split(/\s+/).filter((c) => c && !HTMX_CLASSES.includes(c));
        if (classes.length > 0) entries.push([name, classes.join(" ")]);
        continue;
      }
      entries.push([name, value]);
    }
    return entries.sort((a, b) => (a[0] < b[0] ? -1 : 1));
  }

  function describe(node) {
    if (node.nodeType === 1) {
      return "<" + node.localName + (node.id ? "#" + node.id : "") + ">";
    }
    return node.nodeType === 3 ? "text " + quote(node.nodeValue) : "node type " + node.nodeType;
  }

  const isBeingEdited = (node) => node.nodeType === 1 && node.hasAttribute("data-editing");

  function comparableChildren(liveKids, freshKids) {
    for (const entry of liveKids.filter(isBeingEdited)) {
      freshKids = freshKids.filter((node) => node.nodeType !== 1 || node.id !== entry.id);
    }
    liveKids = liveKids.filter((node) => !isBeingEdited(node));
    for (const slot of liveKids.filter(showsInlineError)) {
      const name = slot.getAttribute("data-error-slot");
      const at = freshKids.findIndex(
        (node) => node.nodeType === 1 && node.getAttribute("data-error-slot") === name,
      );
      if (at >= 0) freshKids = freshKids.filter((_, index) => index !== at);
    }
    return [liveKids.filter((node) => !showsInlineError(node)), freshKids];
  }

  function diffChildren(allLiveKids, allFreshKids, path) {
    const [liveKids, freshKids] = comparableChildren(allLiveKids, allFreshKids);
    const shared = Math.min(liveKids.length, freshKids.length);
    let elementIndex = 0;
    for (let index = 0; index < shared; index += 1) {
      const step =
        liveKids[index].nodeType === 1
          ? path + " > " + describe(liveKids[index]).slice(1, -1) + "[" + elementIndex + "]"
          : path;
      if (liveKids[index].nodeType === 1) elementIndex += 1;
      const diff = diffNodes(liveKids[index], freshKids[index], step);
      if (diff) return diff;
    }
    if (liveKids.length !== freshKids.length) {
      const extra = liveKids.length > freshKids.length ? liveKids[shared] : freshKids[shared];
      const side = liveKids.length > freshKids.length ? "only the page has" : "only the server renders";
      return path + ": " + side + " " + describe(extra) + " (child " + shared + ")";
    }
    return null;
  }

  function diffNodes(live, fresh, path) {
    if (live.nodeType !== fresh.nodeType) {
      return path + ": the page has " + describe(live) + ", the server renders " + describe(fresh);
    }
    if (live.nodeType !== 1) {
      return live.nodeValue === fresh.nodeValue
        ? null
        : path + ": text " + quote(live.nodeValue) + " vs " + quote(fresh.nodeValue);
    }
    if (live.localName !== fresh.localName) {
      return path + ": the page has " + describe(live) + ", the server renders " + describe(fresh);
    }
    const liveAttributes = new Map(comparableAttributes(live));
    const freshAttributes = new Map(comparableAttributes(fresh));
    for (const name of new Set([...liveAttributes.keys(), ...freshAttributes.keys()])) {
      if (liveAttributes.get(name) !== freshAttributes.get(name)) {
        return (
          path + ": attribute " + name + " " + quote(String(liveAttributes.get(name))) +
          " vs " + quote(String(freshAttributes.get(name))) +
          " on " + quote(live.cloneNode(false).outerHTML)
        );
      }
    }
    return diffChildren(
      Array.from(live.childNodes),
      Array.from(fresh.childNodes),
      path,
    );
  }

  async function diffPart(part) {
    const name = partName(part);
    const route = "/screen?parts=" + name;
    const response = await fetch(route, { cache: "no-store" });
    if (!response.ok) return "GET " + route + " answered " + response.status;
    const answer = document.createElement("template");
    answer.innerHTML = await response.text();
    const fresh = answer.content.querySelector("template[data-part]");
    return diffChildren(
      Array.from(part.childNodes),
      Array.from(fresh.content.childNodes),
      "[data-part=" + name + "]",
    );
  }

  // A script sets these on a part element: the focus fallback of a load, and the lock
  // of the Edit controls while a story-log entry is in edit. The lock is expected
  // exactly when the part holds an entry in edit.
  const SCRIPT_ATTRIBUTES = ["tabindex", "data-editing-entry"];

  // The action area derives the state class of the status display from the part's
  // content (`statusDisplayState`), so the class is compared by that rule.
  const STATUS_STATE_CLASSES = ["ready", "thinking", "error"];

  function withoutStateClasses(attributes) {
    if (!attributes.has("class")) return;
    const kept = attributes
      .get("class")
      .split(/\s+/)
      .filter((name) => !STATUS_STATE_CLASSES.includes(name));
    attributes.set("class", kept.join(" "));
  }

  function statusStateProblem(part) {
    const states = part.className.split(/\s+/).filter((name) => STATUS_STATE_CLASSES.includes(name));
    const want = statusDisplayState(part);
    return states.length === 1 && states[0] === want
      ? null
      : "the state class is " + quote(states.join(" ")) + ", the content says " + quote(want);
  }

  function diffPartElement(part, shellPart) {
    const expected = new Map(comparableAttributes(shellPart));
    const live = new Map(comparableAttributes(part));
    if (partName(part) === "status-display") {
      const problem = statusStateProblem(part);
      if (problem) return problem;
      withoutStateClasses(live);
      withoutStateClasses(expected);
    }
    for (const name of SCRIPT_ATTRIBUTES) {
      live.delete(name);
      expected.delete(name);
    }
    const editing = part.querySelector("[data-editing]") !== null;
    const locked = part.hasAttribute("data-editing-entry");
    if (locked !== editing) {
      return editing
        ? "an entry is in edit and the part is not locked"
        : "the part is locked and no entry is in edit";
    }
    for (const name of new Set([...live.keys(), ...expected.keys()])) {
      if (live.get(name) !== expected.get(name)) {
        return (
          "attribute " + name + " " + quote(String(live.get(name))) +
          " on the part element, the shell has " + quote(String(expected.get(name)))
        );
      }
    }
    return null;
  }

  async function shellParts() {
    const response = await fetch("/", { cache: "no-store" });
    const shell = new DOMParser().parseFromString(await response.text(), "text/html");
    return new Map(
      Array.from(shell.querySelectorAll("[data-part]")).map((part) => [partName(part), part]),
    );
  }

  const deadline = Date.now() + timeoutMs;
  let notQuiet = [];
  let blocking = [];
  for (;;) {
    const parts = visibleParts();
    notQuiet = await reasonsNotQuiet();
    blocking = [];
    if (notQuiet.length === 0) {
      const shell = await shellParts();
      for (const part of Array.from(document.querySelectorAll("[data-part]"))) {
        if (parts.includes(part)) continue;
        if (part.childNodes.length > 0) {
          blocking.push({
            part: partName(part),
            route: "(closed tab)",
            diff: "a part of a closed tab holds " + part.childNodes.length + " nodes",
          });
        }
      }
      for (const part of parts) {
        const rootDiff = diffPartElement(part, shell.get(partName(part)));
        if (rootDiff) {
          blocking.push({ part: partName(part), route: "/", diff: rootDiff });
        }
        const diff = await diffPart(part);
        if (!diff) continue;
        blocking.push({
          part: partName(part),
          route: "/screen?parts=" + partName(part),
          diff,
        });
      }
      if (blocking.length === 0) return JSON.stringify({ ok: true, notQuiet, blocking });
    }
    if (Date.now() > deadline) {
      return JSON.stringify({ ok: false, notQuiet, blocking });
    }
    // The page asks every 100 ms in the stub, so a fixed pause could meet each ask in flight.
    await sleep(60 + Math.random() * 80);
  }
}
