// Panel chrome: the tab bar, the Settings sub-tabs, and the LLM Messages expansion toggle.

// Tab state is client-only, so the active tab is remembered for a reload of the
// page.
const ACTIVE_TAB_KEY = "chronicle.activeTab";

function activateTab(btn) {
  document.querySelectorAll(".tab").forEach((b) => {
    b.classList.remove("active");
    b.setAttribute("aria-selected", b === btn ? "true" : "false");
  });
  document
    .querySelectorAll(".tab-content")
    .forEach((c) => c.classList.remove("active"));
  btn.classList.add("active");
  const panel = document.getElementById(btn.dataset.tab + "-tab");
  if (panel) {
    panel.classList.add("active");
    // A section that opens shows its regions: each one that is due loads.
    askNow();
  }
}

function rememberActiveTab(tab) {
  try {
    sessionStorage.setItem(ACTIVE_TAB_KEY, tab);
  } catch (e) {
    // Without storage the active tab lasts for the page only.
  }
}

function restoreActiveTab() {
  let saved = null;
  try {
    saved = sessionStorage.getItem(ACTIVE_TAB_KEY);
  } catch (e) {
    return;
  }
  if (!saved) return;
  const btn = document.querySelector('.tab[data-tab="' + saved + '"]');
  if (btn) activateTab(btn);
}

document.addEventListener("DOMContentLoaded", function () {

  restoreActiveTab();

  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      activateTab(btn);
      rememberActiveTab(btn.dataset.tab);
    });
  });

  // The selected sub-tab is view state in `aria-selected` (buttons) and `hidden`
  // (panels). Both are in `data-keep`, so a load of the Settings panel keeps them.
  document.addEventListener("click", (event) => {
    const subtab = event.target.closest(".settings-subtab");
    if (!subtab) return;
    const panel = subtab.closest(".settings-panel");
    if (!panel) return;
    panel.querySelectorAll(".settings-subtab").forEach((b) => {
      b.setAttribute("aria-selected", b === subtab ? "true" : "false");
    });
    const target = subtab.getAttribute("aria-controls");
    panel.querySelectorAll(".settings-subtab-panel").forEach((p) => {
      p.hidden = p.id !== target;
    });
  });
});
