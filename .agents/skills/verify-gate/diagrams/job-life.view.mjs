// The life of one Job (Job 1), as the code runs it. Other work on the Game is not drawn.
export const title = "The life of one Job";

const J = 1;
const live = (j) => j.pc !== "idle" && j.pc !== "done";

const part = (j) => {
  const ev = j.gen === "event";
  switch (j.pc) {
    case "idle": return "Not started";
    case "memory": case "memnext": case "memstep": return ev ? "Event Narration: story memory writer call" : "Narration: story memory writer call";
    case "start": case "call": case "save": return ev ? "Event Narration" : "Narration";
    case "analyse": case "quant": case "update": return "Quantifier call and state update";
    case "afterGens": return "After the generations";
    case "optstart": case "optcall": case "optsave": return "Options run";
    case "endfail": case "drop": return "Ends";
    case "done":
      if (j.fail === "moved") return "Ended: a write was refused";
      if (j.fail === "line") return "Ended with a failure line";
      return "Ended";
  }
  return j.pc;
};

export const state = (v) => {
  const j = v.job[J];
  if (live(j) && v.cut.includes(J)) return "Cut off: runs on, but its next write is refused";
  if (j.pc === "done" && v.cut.includes(J) && j.fail !== "moved") return "Ended after the stop";
  if (live(j)) return ["Holds the Game", part(j)];
  return part(j);
};

export const group = (v) => {
  const j = v.job[J];
  if (j.pc === "idle") return "Before";
  if (j.pc === "done") return "Ended";
  if (v.cut.includes(J)) return "Cut off";
  return "Holds the Game";
};
export const colors = { Before: "tan", "Holds the Game": "blue", "Cut off": "orange", Ended: "teal" };

const labels = {
  YouStartAJob: "the player sends an Action",
  YouPressStop: "the player presses Stop",
  TheMemoryWriterAnswers: "the story memory writer answers",
  TheMemoryWriterFails: "the story memory writer call fails",
  TheNextMemoryCallStarts: "the next story memory writer call starts",
  TheNarrationStepReturns: "the narrator call starts",
  TheNarratorAnswers: "the narrator answers",
  TheNarratorFails: "the narrator call fails",
  TheTextIsSaved: "the text is saved",
  TheAnalysisStarts: "the Quantifier call starts",
  TheAnalysisAnswers: "the Quantifier answers",
  TheOptionsRunFollows: "the Options run follows",
  TheJobSkipsTheOptionsRun: "the Job has no Options run",
  TheOptionsRunStarts: "the Options call starts",
  TheOptionsCallEnds: "the Options call ends",
  TheOptionsAreSaved: "the Options are saved",
  TheFailureLineIsSaved: "the failure line is saved",
  TheJobEnds: "the Job ends",
};

export const step = (action, from, to) => {
  const a = from.job[J], b = to.job[J];
  if (b.pc === "done" && b.fail === "moved" && a.pc !== "done") return "its next write is refused";
  if (action === "AJobStartsAGeneration") return b.pc === "memory" ? "the story memory writer call starts" : "the narrator call starts";
  if (action === "TheStateIsSaved") return b.gen === "event" && a.gen === "first" ? "the state is saved, a Trigger fired" : "the state is saved";
  if (action === "TheMemoryWriterAnswers") return b.pc === "memnext" ? "the story memory writer answers, a further call follows" : "the story memory writer answers";
  return labels[action] ?? action;
};
