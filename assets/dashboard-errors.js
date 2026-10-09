// Failure surfaces: inline error slots, the failure banner, the details popovers and the htmx error hooks.

console.log("[HTMX] Initializing...");

function escapeHtml(value) {
  const entities = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
  };
  return String(value).replace(/[&<>"']/g, (ch) => entities[ch]);
}

// Mirrors the server's disclosure shape for a request with no response body.
function errorDisclosureHtml(popoverId, message, rawText) {
  return (
    '<div class="error-disclosure">' +
    '<span class="error-disclosure-message">' + escapeHtml(message) + "</span>" +
    '<button type="button" class="error-details-toggle" aria-expanded="false" aria-controls="' +
    escapeHtml(popoverId) + '">Details</button>' +
    '<div class="error-detail-popover" id="' + escapeHtml(popoverId) + '" hidden>' +
    '<pre class="error-detail-raw">' + escapeHtml(rawText) + "</pre></div></div>"
  );
}

const INLINE_ERROR_CONTAINERS =
  "form, .world-item, .preset-card, .game-item, .llm-message-card, .posture-override, .posture-group, .worlds-panel, .games-panel";

// The counter keeps two generated slots from sharing one popover id.
let generatedErrorSlots = 0;

// A failed action needs a region to report in, so a container without a slot
// gets one here.
function inlineErrorSlotFor(elt, create) {
  if (!elt || !elt.closest) return null;
  const container = elt.closest(INLINE_ERROR_CONTAINERS);
  if (!container) return null;
  const existing = container.querySelector(":scope > [data-error-slot]");
  if (existing || !create) return existing;
  const slot = document.createElement("div");
  slot.className = "inline-error-slot";
  slot.dataset.errorSlot = "generated-" + ++generatedErrorSlots;
  slot.hidden = true;
  container.appendChild(slot);
  return slot;
}

function renderInlineError(slot, message, rawText) {
  if (!slot) return;
  const id = slot.dataset.errorSlot || "inline-error";
  slot.innerHTML = errorDisclosureHtml(id + "-popover", message, rawText);
  slot.hidden = false;
}

function clearInlineError(slot) {
  if (!slot) return;
  slot.innerHTML = "";
  slot.hidden = true;
}

function refreshFailureBanner() {
  const banner = document.getElementById("failure-banner");
  const degraded = document.getElementById("failure-banner-degraded");
  const unreachable = document.getElementById("failure-banner-unreachable");
  if (!banner) return;
  const degradedUp = !!degraded && degraded.innerHTML.trim() !== "";
  const unreachableUp = !!unreachable && unreachable.innerHTML.trim() !== "";
  banner.hidden = !(degradedUp || unreachableUp);
  banner.classList.toggle("unreachable", unreachableUp);
  banner.setAttribute("role", unreachableUp ? "alert" : "status");
}

function markUnreachable() {
  const unreachable = document.getElementById("failure-banner-unreachable");
  if (!unreachable) return;
  if (unreachable.innerHTML.trim() === "") {
    unreachable.innerHTML =
      '<div class="error-disclosure">' +
      '<span class="error-disclosure-message">The engine is unreachable. Nothing you do is being saved.</span>' +
      "</div>";
  }
  refreshFailureBanner();
}

function clearUnreachable() {
  const unreachable = document.getElementById("failure-banner-unreachable");
  if (!unreachable || unreachable.innerHTML.trim() === "") return;
  unreachable.innerHTML = "";
  refreshFailureBanner();
}

// A server error body is an HTML fragment, so the disclosure shows its text.
function plainResponseText(body) {
  return (body || "").replace(/<[^>]*>/g, "").trim();
}

// A failed fetch reports the server's own words behind the Details disclosure.
function throwServerDetail(response) {
  return response.text().then((body) => {
    throw new Error(
      plainResponseText(body) || `The server answered ${response.status}.`
    );
  });
}

// A server that answered is reachable, so only a failed poll marks the engine unreachable.
function isPollRequest(elt) {
  if (!elt || !elt.getAttribute) return false;
  const trigger = elt.getAttribute("hx-trigger") || "";
  return trigger.includes("every");
}

document.body.addEventListener("htmx:responseError", function (evt) {
  const slot = inlineErrorSlotFor(evt.detail.elt, true);
  if (slot) {
    renderInlineError(slot, "That action failed.", plainResponseText(evt.detail.xhr.responseText));
  } else if (isPollRequest(evt.detail.elt)) {
    markUnreachable();
  }
});

document.body.addEventListener("htmx:sendError", function (evt) {
  markUnreachable();
  const slot = inlineErrorSlotFor(evt.detail.elt, true);
  if (slot) {
    renderInlineError(slot, "The engine is unreachable.", "No response from the server.");
  }
});

document.body.addEventListener("htmx:afterRequest", function (evt) {
  if (!evt.detail.successful) return;
  clearUnreachable();
  clearInlineError(inlineErrorSlotFor(evt.detail.elt));
});

document.body.addEventListener("htmx:afterSettle", refreshFailureBanner);

// A swap replaces the open disclosure node, which collapses it and drops focus.
let openErrorDetails = null;

function closeErrorPopover(popover) {
  if (!popover) return;
  popover.classList.remove("open");
  popover.hidden = true;
  if (openErrorDetails && openErrorDetails.popoverId === popover.id) {
    openErrorDetails = null;
  }
  const toggle = document.querySelector(
    '.error-details-toggle[aria-controls="' + popover.id + '"]',
  );
  if (toggle) toggle.setAttribute("aria-expanded", "false");
}

function rememberOpenErrorDetails(popover, toggle) {
  openErrorDetails = {
    popoverId: popover.id,
    hadFocus: document.activeElement === toggle,
  };
}

function restoreOpenErrorDetails() {
  if (!openErrorDetails) return;
  const popover = document.getElementById(openErrorDetails.popoverId);
  const toggle = document.querySelector(
    '.error-details-toggle[aria-controls="' + openErrorDetails.popoverId + '"]',
  );
  if (!popover || !toggle) {
    openErrorDetails = null;
    return;
  }
  popover.classList.add("open");
  popover.hidden = false;
  toggle.setAttribute("aria-expanded", "true");
  const focusFell =
    !document.activeElement || document.activeElement === document.body;
  if (openErrorDetails.hadFocus && focusFell) toggle.focus();
  positionErrorPopover(popover, toggle);
}

function closeAllErrorPopovers() {
  document.querySelectorAll(".error-detail-popover.open").forEach(closeErrorPopover);
}

function positionErrorPopover(popover, toggle) {
  const rect = toggle.getBoundingClientRect();
  popover.style.left = Math.max(8, rect.left) + "px";
  const spaceBelow = window.innerHeight - rect.bottom;
  if (spaceBelow < 220) {
    popover.style.bottom = window.innerHeight - rect.top + 6 + "px";
    popover.style.top = "";
  } else {
    popover.style.top = rect.bottom + 6 + "px";
    popover.style.bottom = "";
  }
}

document.addEventListener("click", function (evt) {
  const target = evt.target;
  if (!(target instanceof HTMLElement)) return;
  const toggle = target.closest(".error-details-toggle");
  if (toggle) {
    const popover = document.getElementById(toggle.getAttribute("aria-controls"));
    if (popover) {
      const open = popover.classList.toggle("open");
      popover.hidden = !open;
      toggle.setAttribute("aria-expanded", String(open));
      if (open) {
        positionErrorPopover(popover, toggle);
        rememberOpenErrorDetails(popover, toggle);
      } else {
        closeErrorPopover(popover);
      }
    }
    return;
  }
  if (!target.closest(".error-detail-popover")) closeAllErrorPopovers();
});

document.addEventListener("keydown", function (evt) {
  if (evt.key === "Escape") closeAllErrorPopovers();
});

window.addEventListener("resize", closeAllErrorPopovers);
window.addEventListener("scroll", closeAllErrorPopovers, true);

// `afterSettle` fires on the final DOM after out-of-band processing, so the restore runs on both events.
document.body.addEventListener("htmx:afterSwap", restoreOpenErrorDetails);
document.body.addEventListener("htmx:afterSettle", restoreOpenErrorDetails);
