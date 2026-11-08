// The busy state of one Game: who holds it, and whether it shows a failure line.
export const title = "The busy state of one Game";

export const state = (v) => {
  const h = v.holder[1];
  let words;
  if (h.kind === "job") words = "A Job holds it";
  else if (h.kind === "run") words = h.job === 0 ? "A run holds it, between two Jobs" : "A run holds it, its Job runs";
  else words = "Idle";
  if (v.failLine[1] !== 0) words += ", with a failure line";
  return words;
};
export const group = (v) => (v.holder[1].kind === "none" ? "Idle" : "Busy");
export const colors = { Idle: "teal", Busy: "blue" };

export const step = (action, from, to) => {
  switch (action) {
    case "YouStartAJob": return "you send an Action";
    case "YouStartAutoplay": return "you start Autoplay";
    case "YouPressStop": return "you press Stop";
    case "TheAcceptCannotRead": return "the accept cannot read the story";
    case "TheFailureLineIsSaved": return "the Job saves its failure line";
    case "TheJobEnds": return "the Job ends";
    case "TheRunAccepts":
      if (to.holder[1].kind === "run") return "the run starts its next Job";
      if (to.failLine[1] !== 0) return "the run's accept cannot read the story";
      return "the run's budget is spent";
    case "TheRunEnds": return "the run ends";
  }
  return null;
};
