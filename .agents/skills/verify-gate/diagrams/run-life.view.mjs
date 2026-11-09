// The life of one Autoplay run (Autoplay run 1). Its Jobs are drawn as one state.
export const title = "The life of one Autoplay run";
const R = 1;
const ends = { stopped: "stopped", budget: "its budget is spent", jobFailed: "its Job failed", notAccepted: "its accept failed" };

export const state = (v) => {
  const r = v.run[R];
  const g = r.g;
  const holds = g !== 0 && v.holder[g].kind === "run" && v.holder[g].run === R;
  switch (r.pc) {
    case "idle": return "Not started";
    case "accept": return holds ? ["Holds the Game", "At its accept"] : "Stopped: waits to reach its accept";
    case "job": return holds ? ["Holds the Game", "Its Job runs"] : "Stopped: its Job runs on, cut off";
    case "end": return holds ? ["Holds the Game", "Ends"] : `Holds the Game no more: ${ends[r.end]}`;
    case "over": return `Ended: ${ends[r.end]}`;
  }
  return r.pc;
};

export const group = (v) => {
  const r = v.run[R];
  if (r.pc === "idle") return "Before";
  if (r.pc === "over") return "Ended";
  const g = r.g;
  return v.holder[g].kind === "run" && v.holder[g].run === R ? "Holds the Game" : "Holds it no more";
};
export const colors = { Before: "tan", "Holds the Game": "blue", "Holds it no more": "orange", Ended: "teal" };

export const step = (action, from, to) => {
  const a = from.run[R], b = to.run[R];
  switch (action) {
    case "YouStartAutoplay": return "the player starts Autoplay";
    case "YouPressStop": return "the player presses Stop";
    case "TheRunAccepts":
      if (b.pc === "job") return "the Autoplay run starts its next Job";
      if (b.end === "budget") return "the budget is spent";
      if (b.end === "notAccepted") return "the accept of the Autoplay run cannot read the story";
      return "the Autoplay run holds the Game no more";
    case "TheRunSeesItsJobEnd":
      if (b.pc === "accept") return "its Job ends";
      if (b.end === "jobFailed") return "its Job fails";
      return "its Job was cut off";
    case "TheRunEnds": return "the Autoplay run ends";
  }
  return null;
};
