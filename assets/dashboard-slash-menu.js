// The slash menu: the command palette rendered from the shipped shell's input listener.

const SLASH_COMMANDS = [
  { cmd: "/impersonate", desc: "Write as the player persona" },
  { cmd: "/guide", desc: "Steer the next generation (Guided Generation)" },
  { cmd: "/options", desc: "Suggest actions (generate pickable options)" },
];
let slashMenu = null;
let slashMatches = [];
let slashActiveIndex = -1;

function commandInput() {
  return document.querySelector('#command-form input[name="command"]');
}

function closeSlashMenu() {
  if (slashMenu) {
    slashMenu.remove();
    slashMenu = null;
  }
  slashMatches = [];
  slashActiveIndex = -1;
  const input = commandInput();
  if (input) {
    input.setAttribute("aria-expanded", "false");
    input.removeAttribute("aria-activedescendant");
  }
}

function slashMatchesFor(value) {
  const v = value.trimStart();
  if (!v.startsWith("/")) return [];
  return SLASH_COMMANDS.filter((c) => c.cmd.startsWith(v));
}

function positionSlashMenu(input) {
  if (!slashMenu) return;
  const rect = input.getBoundingClientRect();
  slashMenu.style.left = rect.left + "px";
  slashMenu.style.bottom = window.innerHeight - rect.top + 4 + "px";
}

function renderSlashMenu(input) {
  if (slashMatches.length === 0) {
    closeSlashMenu();
    return;
  }
  if (!slashMenu) {
    slashMenu = document.createElement("div");
    slashMenu.className = "slash-menu";
    slashMenu.id = "slash-menu";
    slashMenu.setAttribute("role", "listbox");
    slashMenu.setAttribute("aria-label", "Slash commands");
    document.body.appendChild(slashMenu);
  }
  slashMenu.innerHTML = "";
  slashMatches.forEach((m, i) => {
    const item = document.createElement("div");
    item.className = "slash-suggestion" + (i === slashActiveIndex ? " active" : "");
    item.id = "slash-option-" + i;
    item.setAttribute("role", "option");
    item.setAttribute("aria-selected", i === slashActiveIndex ? "true" : "false");
    item.dataset.cmd = m.cmd;
    const cmdSpan = document.createElement("span");
    cmdSpan.className = "slash-cmd";
    cmdSpan.textContent = m.cmd;
    const descSpan = document.createElement("span");
    descSpan.className = "slash-desc";
    descSpan.textContent = m.desc;
    item.appendChild(cmdSpan);
    item.appendChild(descSpan);
    slashMenu.appendChild(item);
  });
  positionSlashMenu(input);
  input.setAttribute("aria-expanded", "true");
  syncSlashActiveDescendant(input);
}

function syncSlashActiveDescendant(input) {
  if (slashActiveIndex >= 0 && slashMenu) {
    input.setAttribute("aria-activedescendant", "slash-option-" + slashActiveIndex);
  } else {
    input.removeAttribute("aria-activedescendant");
  }
}

function openSlashMenu(input) {
  slashMatches = slashMatchesFor(input.value);
  slashActiveIndex = slashMatches.length > 0 ? 0 : -1;
  renderSlashMenu(input);
}

function selectSlash(cmd, input) {
  input.value = cmd + " ";
  input.focus();
  const len = input.value.length;
  input.setSelectionRange(len, len);
  closeSlashMenu();
}

function setSlashActive(index) {
  slashActiveIndex = index;
  if (slashMenu) {
    slashMenu.querySelectorAll(".slash-suggestion").forEach((el, i) => {
      el.classList.toggle("active", i === slashActiveIndex);
      el.setAttribute("aria-selected", i === slashActiveIndex ? "true" : "false");
    });
  }
  const input = commandInput();
  if (input) syncSlashActiveDescendant(input);
}

document.addEventListener("input", function (e) {
  const input = e.target;
  if (!(input instanceof HTMLElement)) return;
  if (!input.matches('#command-form input[name="command"]')) return;
  openSlashMenu(input);
});

document.addEventListener("keydown", function (e) {
  const input = commandInput();
  if (!input || e.target !== input) return;
  if (!slashMenu) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (slashMatches.length > 0) {
      setSlashActive((slashActiveIndex + 1) % slashMatches.length);
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (slashMatches.length > 0) {
      setSlashActive((slashActiveIndex - 1 + slashMatches.length) % slashMatches.length);
    }
  } else if (e.key === "Enter") {
    if (slashMatches.length > 0 && slashActiveIndex >= 0) {
      e.preventDefault();
      selectSlash(slashMatches[slashActiveIndex].cmd, input);
    } else {
      closeSlashMenu();
    }
  } else if (e.key === "Escape") {
    e.preventDefault();
    closeSlashMenu();
  }
});

// mousedown on a suggestion selects (fires before the input blurs).
document.addEventListener("mousedown", function (e) {
  const target = e.target;
  if (!(target instanceof HTMLElement)) return;
  const item = target.closest(".slash-suggestion");
  if (item && slashMenu && slashMenu.contains(item)) {
    e.preventDefault();
    const input = commandInput();
    if (input) selectSlash(item.dataset.cmd, input);
    return;
  }
  if (slashMenu && !slashMenu.contains(target) && target !== commandInput()) {
    closeSlashMenu();
  }
});

// focusout bubbles, unlike blur.
document.addEventListener("focusout", function (e) {
  if (!slashMenu) return;
  if (e.target === commandInput()) {
    setTimeout(() => {
      if (commandInput() !== document.activeElement && slashMenu) {
        closeSlashMenu();
      }
    }, 150);
  }
});

document.addEventListener("submit", closeSlashMenu, true);

// The menu is position:fixed, so scroll or resize leaves it stale.
window.addEventListener("scroll", closeSlashMenu, true);
window.addEventListener("resize", closeSlashMenu);
