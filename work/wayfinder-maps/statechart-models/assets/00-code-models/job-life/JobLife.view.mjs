export const title = "The life of one Job";

export const state = (v) => {
  if (v.holder === "job") {
    if (v.step === "narrating") return "Writing the narration";
    if (v.step === "quantifying") return "Reading the new text";
    if (v.step === "generating event") return "Writing the event narration";
    return "Offering the Options";
  }
  if (v.stopped) return "Idle, you stopped it";
  if (v.failure === "shown") return "Idle with a failure line";
  return "Idle";
};

export const group = (v) => {
  if (v.holder === "job") return v.step === "options" ? "Offering" : "Writing";
  return "At rest";
};

export const colors = { "At rest": "teal", Writing: "blue", Offering: "purple" };

export const step = (action) => {
  switch (action) {
    case "YouSendAnAction":
      return "you send an action";
    case "TheNarrationIsSaved":
      return "the narration is saved";
    case "ATriggerFires":
      return "a Trigger fires";
    case "TheStateUpdateFiresNoTrigger":
      return "no Trigger fires";
    case "TheEventNarrationIsSaved":
      return "the event narration is saved";
    case "TheOptionsRunEnds":
      return "the Options appear";
    case "TheJobEndsWithNoOptionsOwed":
      return "the Job ends [with the Options toggle off]";
    case "APartFailsAndTheOptionsFollow":
      return "a part fails, and the Options still run";
    case "APartFailsAndTheJobEnds":
      return "a part fails";
    case "YouPressStop":
      return "you press Stop";
    default:
      return action;
  }
};
