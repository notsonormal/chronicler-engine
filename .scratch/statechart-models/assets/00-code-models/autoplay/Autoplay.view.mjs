export const title = "One Autoplay run";

export const state = (v) => {
  if (v.holder === "run" && v.step === "writing")
    return `A Job of the run is writing (cycle ${v.done + 1} of ${v.budget})`;
  if (v.holder === "run") {
    if (v.done === 0) return "Between two Jobs (no cycle played)";
    if (v.done >= v.budget) return `Between two Jobs (all ${v.budget} cycles played)`;
    return `Between two Jobs (${v.done} of ${v.budget} cycles played)`;
  }
  if (v.ending === "budget") return "Idle, the run played its budget";
  if (v.ending === "failed") return "Idle, a Job of the run failed";
  if (v.ending === "stopped") return "Idle, you stopped the run";
  return "Idle";
};

export const group = (v) => (v.holder === "run" ? "Playing" : "At rest");

export const colors = { "At rest": "teal", Playing: "blue" };

export const step = (action) => {
  switch (action) {
    case "YouStartAutoplay":
      return "you start Autoplay";
    case "TheRunAcceptsAJob":
      return "the run starts the next Job";
    case "AJobEndsACycle":
      return "a Job ends a cycle";
    case "AJobEndsMidCycle":
      return "the story waits for a narration, so no cycle ends";
    case "TheBudgetIsReached":
      return "the run played its budget";
    case "AJobOfTheRunFails":
      return "a Job of the run fails";
    case "YouPressStop":
      return "you press Stop";
    default:
      return action;
  }
};
