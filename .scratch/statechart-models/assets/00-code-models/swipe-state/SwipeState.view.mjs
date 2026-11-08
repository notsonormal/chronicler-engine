export const title = "The state of a new Swipe";

export const state = (v) => {
  if (v.stage === "writing" && v.request === "a retry of a typed input")
    return "The narrator writes the retry";
  if (v.stage === "writing") return "The narrator writes";
  if (v.stage === "saved") return "The text shows, its state is not read yet";
  if (v.stage === "analysed") return "The rules read the new text";
  if (v.current === "pre-analysis") return "Idle, the text keeps a state the rules never read";
  if (v.current === "analysed") return "Idle, the text is read";
  return "Idle, the text keeps its start state";
};

export const group = (v) => {
  if (v.stage === "idle") return v.current === "pre-analysis" ? "Unread" : "At rest";
  return v.stage === "saved" ? "Unread" : "Writing";
};

export const colors = { "At rest": "teal", Writing: "blue", Unread: "orange" };

export const step = (action) => {
  switch (action) {
    case "YouSendAnAction":
      return "you send an action";
    case "YouRetryATypedInput":
      return "you retry a typed input";
    case "TheTextIsSaved":
      return "the text is saved with the pre-analysis state";
    case "TheTextOfARetryIsSaved":
      return "the retry's text is saved and keeps the start state";
    case "TheStateUpdateLands":
      return "the rules read the new text";
    case "TheJobEnds":
      return "the Job ends";
    case "YouPressStop":
      return "you press Stop";
    case "TheServerRestarts":
      return "the server restarts";
    default:
      return action;
  }
};
