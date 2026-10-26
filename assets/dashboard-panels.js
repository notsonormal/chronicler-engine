// Panel chrome: the tab bar, and the Settings sub-tabs.

// Tab state is client-only, so the active tab and the Settings sub-tab are
// remembered for a reload of the page. The Settings sub-tab is also what a load of
// the Settings part restores: the part is empty while the tab is closed.
const ACTIVE_TAB_KEY = "chronicle.activeTab";
const SETTINGS_SUBTAB_KEY = "chronicle.settingsSubtab";

function activateTab(btn) {
  const leaving = document.querySelector(".tab-content.active");
  const panel = document.getElementById(btn.dataset.tab + "-tab");
  document.querySelectorAll(".tab").forEach((b) => {
    b.classList.remove("active");
    b.setAttribute("aria-selected", b === btn ? "true" : "false");
  });
  document
    .querySelectorAll(".tab-content")
    .forEach((c) => c.classList.remove("active"));
  btn.classList.add("active");
  if (!panel) return;
  panel.classList.add("active");
  // The tab that closes holds no content, and the tab that opens loads.
  if (leaving && leaving !== panel) leaveSection(leaving);
  askNow();
}

function rememberSettingsSubtab(subtab) {
  try {
    sessionStorage.setItem(SETTINGS_SUBTAB_KEY, subtab.getAttribute("aria-controls"));
  } catch (e) {
    // Without storage the sub-tab lasts until the Settings tab closes.
  }
}

function selectSettingsSubtab(panel, subtab) {
  panel.querySelectorAll(".settings-subtab").forEach((b) => {
    b.setAttribute("aria-selected", b === subtab ? "true" : "false");
  });
  const target = subtab.getAttribute("aria-controls");
  panel.querySelectorAll(".settings-subtab-panel").forEach((p) => {
    p.hidden = p.id !== target;
  });
}

// A fresh render of the Settings part shows the first sub-tab. The sub-tab the
// player chose comes back in the same task as the morph, so it never shows.
document.addEventListener("screen:loaded", function (evt) {
  if (!evt.detail.parts.includes("settings")) return;
  let saved = null;
  try {
    saved = sessionStorage.getItem(SETTINGS_SUBTAB_KEY);
  } catch (e) {
    return;
  }
  const panel = document.querySelector('[data-part="settings"]');
  if (!saved || !panel) return;
  const subtab = Array.from(panel.querySelectorAll(".settings-subtab")).find(
    (b) => b.getAttribute("aria-controls") === saved,
  );
  if (subtab && subtab.getAttribute("aria-selected") !== "true") {
    selectSettingsSubtab(panel, subtab);
  }
});

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
    selectSettingsSubtab(panel, subtab);
    rememberSettingsSubtab(subtab);
  });
});
