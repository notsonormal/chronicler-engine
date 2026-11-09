// The busy state of one Game: who holds it.
export const title = "The busy state of one Game";

export const state = (v) => {
  const h = v.holder[1];
  if (h.kind === "job") return "A Job holds it";
  if (h.kind === "run") return h.job === 0 ? "An Autoplay run holds it, between two Jobs" : "An Autoplay run holds it, and its Job runs";
  return "Idle";
};
export const group = (v) => (v.holder[1].kind === "none" ? "Idle" : "Busy");
export const colors = { Idle: "teal", Busy: "blue" };

export const step = (action, from, to) => {
  switch (action) {
    case "YouStartAJob": return "the player sends an Action";
    case "YouStartAutoplay": return "the player starts Autoplay";
    case "YouPressStop": return "the player presses Stop";
    case "TheAcceptCannotRead": return "the accept cannot read the story";
    case "TheJobEnds": return "the Job ends";
    case "TheRunAccepts":
      if (to.holder[1].kind === "run") return "the Autoplay run starts its next Job";
      if (to.failLine[1] !== 0) return "the accept of the Autoplay run cannot read the story";
      return "the budget is spent";
    case "TheRunEnds": return "the Autoplay run ends";
  }
  return null;
};
