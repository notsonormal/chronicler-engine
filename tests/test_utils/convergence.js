// The body of the convergence check, run inside the page by `convergence.rs`: it waits until the page is quiet, then compares each visible live region with a fresh GET of its own route.
async ({ timeoutMs, heldEditors, tolerableDiffs }) => {
  const HTMX_CLASSES = ["htmx-request", "htmx-settling", "htmx-swapping", "htmx-added"];
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const quote = (text) => JSON.stringify(text.length > 70 ? text.slice(0, 70) + "…" : text);

  const regionName = (region) =>
    region.id ? "#" + region.id : "." + region.classList[0];

  const visibleRegions = () =>
    Array.from(document.querySelectorAll("[data-live]")).filter((region) => {
      const section = region.closest(".tab-content");
      return !section || section.classList.contains("active");
    });

  // The loop keeps its record of each region in `liveRecords`, a script-level
  // binding of dashboard-live.js.
  async function reasonsNotQuiet(regions) {
    let answer;
    try {
      const response = await fetch("/changes", { cache: "no-store" });
      if (!response.ok) return ["GET /changes answered " + response.status];
      answer = await response.json();
    } catch (error) {
      return ["GET /changes failed: " + error.message];
    }
    const reasons = [];
    for (const region of regions) {
      const name = regionName(region);
      const state = region.dataset.liveState;
      if (state !== "current") {
        reasons.push(name + ": data-live-state is " + quote(String(state)));
        continue;
      }
      const record = liveRecords.get(region);
      if (!record) {
        reasons.push(name + ": never loaded");
        continue;
      }
      if (record.loading || region.classList.contains("htmx-request")) {
        reasons.push(name + ": a load is in flight");
        continue;
      }
      const kinds = region.dataset.live.split(/\s+/).filter(Boolean);
      const behind = kinds.filter(
        (kind) => !record.numbers || record.numbers[kind] !== answer[kind],
      );
      if (behind.length > 0) {
        reasons.push(name + ": its record differs from GET /changes in " + behind.join(", "));
      }
    }
    return reasons;
  }

  // A load keeps these two kinds of state (`Idiomorph.defaults.callbacks` in
  // dashboard-live.js): the attributes that the element's `data-keep` names, and
  // an inline error slot that shows an error.
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

  function comparableChildren(liveKids, freshKids) {
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

  // The causes of a hold, by the rule of `regionHoldsEdit` in dashboard-live.js:
  // each `[data-work]` element and each field whose value differs from the one
  // the server rendered (`fieldIsDirty`).
  function holdCauses(region) {
    const fields = Array.from(region.querySelectorAll("input, textarea, select")).filter(fieldIsDirty);
    return [...region.querySelectorAll("[data-work]"), ...fields];
  }

  // A held region is tolerated only when every cause of its hold is inside an
  // editor that the row of the allow list names. Returns the first cause outside
  // one, or null.
  function causeOutsideEditors(region, editors) {
    const allowed = Array.from(region.querySelectorAll(editors));
    return holdCauses(region).find((cause) => !allowed.some((editor) => editor.contains(cause))) || null;
  }

  async function diffRegion(region) {
    const route = region.getAttribute("hx-get");
    const response = await fetch(route, { cache: "no-store", headers: { "HX-Request": "true" } });
    if (!response.ok) return "GET " + route + " answered " + response.status;
    const template = document.createElement("template");
    template.innerHTML = await response.text();
    // The morph extension of htmx hands Idiomorph the element children of the
    // answer, so a text node at the top of the answer is never part of the region.
    return diffChildren(
      Array.from(region.childNodes),
      Array.from(template.content.children),
      regionName(region),
    );
  }

  const deadline = Date.now() + timeoutMs;
  let notQuiet = [];
  let blocking = [];
  let tolerated = [];
  for (;;) {
    // A load does not replace a region that holds an edit (`regionHoldsEdit`), so
    // such a region differs from the server by design, but only while the
    // editor that its row names is the cause.
    const all = visibleRegions();
    const held = [];
    const heldOutside = [];
    for (const region of all) {
      const row = heldEditors.find((entry) => entry.route === region.getAttribute("hx-get"));
      if (!row || !regionHoldsEdit(region)) continue;
      const cause = causeOutsideEditors(region, row.editors);
      if (cause) {
        heldOutside.push({
          region: regionName(region),
          route: region.getAttribute("hx-get"),
          diff: "held by " + quote(cause.cloneNode(false).outerHTML) + " outside the allowed editor " + row.editors,
        });
      } else {
        held.push(region);
      }
    }
    const regions = all.filter((region) => !held.includes(region) && !heldOutside.some((entry) => entry.region === regionName(region)));
    notQuiet = await reasonsNotQuiet(regions);
    blocking = [...heldOutside];
    tolerated = held.map((region) => ({
      region: regionName(region),
      route: region.getAttribute("hx-get"),
      diff: "the region holds an open edit",
    }));
    if (notQuiet.length === 0) {
      for (const region of regions) {
        const diff = await diffRegion(region);
        if (!diff) continue;
        const route = region.getAttribute("hx-get");
        const entry = { region: regionName(region), route, diff };
        const tolerable = tolerableDiffs.some(
          (row) => row.route === route && diff.includes(row.contains),
        );
        (tolerable ? tolerated : blocking).push(entry);
      }
      if (blocking.length === 0) return JSON.stringify({ ok: true, notQuiet, blocking, tolerated });
    }
    if (Date.now() > deadline) {
      return JSON.stringify({ ok: false, notQuiet, blocking, tolerated });
    }
    await sleep(100);
  }
}
