// Failure surfaces: the one reader and reporter of a request answer, the form slots, the notice place, the failure banner and the details popovers.

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

// Mirrors the server's disclosure shape: the short line carries its own text
// element, which the status column's line clamp is applied to. A report with no
// text beyond its short line has no Details, and its sentence wraps instead of
// ending in an ellipsis, because nothing else holds the end of it.
function errorDisclosureHtml(popoverId, message, rawText) {
  const details = rawText
    ? '<button type="button" class="error-details-toggle" aria-expanded="false" aria-controls="' +
      escapeHtml(popoverId) + '" data-keep="aria-expanded">Details</button>' +
      '<div class="error-detail-popover" id="' + escapeHtml(popoverId) + '" hidden data-keep="hidden style">' +
      '<pre class="error-detail-raw">' + escapeHtml(rawText) + "</pre></div>"
    : "";
  return (
    '<div class="error-disclosure' + (rawText ? "" : " error-disclosure-plain") + '">' +
    '<span class="error-disclosure-message">' +
    '<span class="error-disclosure-text">' + escapeHtml(message) + "</span></span>" +
    details + "</div>"
  );
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

function formSlotOf(elt) {
  const container = elt && elt.closest ? elt.closest("form, .edit-actions") : null;
  return container ? container.querySelector(":scope > [data-error-slot]") : null;
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
  fireScreenEvent("banner:changed", {
    text: unreachableUp ? unreachable.textContent.trim() : "",
  });
}

function markUnreachable() {
  const unreachable = document.getElementById("failure-banner-unreachable");
  if (!unreachable) return;
  if (unreachable.innerHTML.trim() === "") {
    unreachable.innerHTML =
      '<div class="error-disclosure">' +
      '<span class="error-disclosure-message"><span class="error-disclosure-text">' +
      "The engine is unreachable. Nothing you do is being saved." +
      "</span></span>" +
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

// The body is escaped HTML, so only a parse gives the real characters and keeps
// the line break between the two lines.
function plainResponseText(body) {
  if (!body) return "";
  return new DOMParser().parseFromString(body, "text/html").body.textContent.trim();
}

const REFUSED_STATUS = 409;

function outcomeOfStatus(status) {
  if (status === 0) return "none";
  if (status < 400) return "done";
  if (status === REFUSED_STATUS) return "refused";
  return status < 500 ? "invalid" : "failed";
}

function requestAnswer(event) {
  const xhr = event && event.detail ? event.detail.xhr : null;
  const status = xhr ? xhr.status : 0;
  return {
    outcome: outcomeOfStatus(status),
    status: status,
    stamp: xhr ? xhr.getResponseHeader("X-Revision") : null,
  };
}

// The place holds one report, and its `data-source` decides which clear applies:
// a screen load never clears the report of a request.
function noticeElement() {
  return document.getElementById("notice");
}

function noticeShows(place, summary, detail, source) {
  if (place.hidden || place.dataset.source !== source) return false;
  const shownSummary = place.querySelector(".error-disclosure-text");
  const shownDetail = place.querySelector(".error-detail-raw");
  return (
    !!shownSummary &&
    shownSummary.textContent === summary &&
    (shownDetail ? shownDetail.textContent : "") === detail
  );
}

// Rewriting the report would close an open Details and fire the alert again, so a
// notice that already shows it is left alone. The dialog is modal, so it hears the
// summary through `notice:changed`.
function showNotice(summary, detail, source) {
  const place = noticeElement();
  const report = place ? place.querySelector("[data-report]") : null;
  if (!place || !report || noticeShows(place, summary, detail, source)) return;
  renderInlineError(report, summary, detail);
  place.dataset.source = source;
  place.hidden = false;
  fireScreenEvent("notice:changed", { summary: summary });
}

function clearNotice() {
  const place = noticeElement();
  if (!place || place.hidden) return;
  clearInlineError(place.querySelector("[data-report]"));
  delete place.dataset.source;
  place.hidden = true;
  fireScreenEvent("notice:changed", { summary: "" });
}

// htmx dispatches `htmx:afterRequest` to the body before it runs the handler of the
// control, so the sender is still in the page when a network error arrives.
function reportAnswer(event) {
  const answer = requestAnswer(event);
  if (answer.outcome === "none") {
    markUnreachable();
    return;
  }
  clearUnreachable();
  if (answer.outcome === "done") return;
  const text = plainResponseText(event.detail.xhr.responseText);
  const summary = text.split("\n")[0].trim() || "The server answered " + answer.status + ".";
  const detail = text === summary ? "" : text;
  const slot = answer.outcome === "invalid" ? formSlotOf(event.detail.elt) : null;
  if (slot) renderInlineError(slot, summary, detail);
  else showNotice(summary, detail, "request");
}

document.body.addEventListener("htmx:afterRequest", reportAnswer);

document.body.addEventListener("htmx:beforeRequest", function (evt) {
  clearNotice();
  clearInlineError(formSlotOf(evt.detail.elt));
});

document.addEventListener("click", function (evt) {
  const target = evt.target;
  if (!(target instanceof Element) || !target.closest("[data-close-notice]")) return;
  clearNotice();
  const main = document.getElementById("main-content");
  if (main) main.focus();
});

document.addEventListener("screen:loaded", refreshFailureBanner);

// The open state lives in `hidden` and the position `style` of the popover, and
// in `aria-expanded` of the toggle. Each lists its attributes in `data-keep`, so a
// load of the part keeps them.
function closeErrorPopover(popover) {
  if (!popover) return;
  popover.hidden = true;
  const toggle = document.querySelector(
    '.error-details-toggle[aria-controls="' + popover.id + '"]',
  );
  if (toggle) toggle.setAttribute("aria-expanded", "false");
}

function closeAllErrorPopovers() {
  document
    .querySelectorAll(".error-detail-popover:not([hidden])")
    .forEach(closeErrorPopover);
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
      const open = popover.hidden;
      popover.hidden = !open;
      toggle.setAttribute("aria-expanded", String(open));
      if (open) {
        positionErrorPopover(popover, toggle);
      } else {
        closeErrorPopover(popover);
      }
    }
    return;
  }
  if (!target.closest(".error-detail-popover")) closeAllErrorPopovers();
});

// Escape closes an open error detail first. It must not also reach the modal
// dialog, which would close and discard the draft that the error is about.
document.addEventListener("keydown", function (evt) {
  if (evt.key !== "Escape") return;
  const anyOpen = document.querySelector(".error-detail-popover:not([hidden])");
  closeAllErrorPopovers();
  if (anyOpen) evt.preventDefault();
});

window.addEventListener("resize", closeAllErrorPopovers);
window.addEventListener("scroll", closeAllErrorPopovers, true);
