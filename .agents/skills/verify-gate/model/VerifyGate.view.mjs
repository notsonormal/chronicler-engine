// The view of the gate model for `/statechart`: the words on each state and arrow.
// A state is what the dashboard of the first Game shows, and whether a Job that a
// stop, a reset or a delete cut off still runs.

export const title = "The gate of one Game";

const live = (j) => j.pc !== "idle" && j.pc !== "done";

const holderWords = (h) => {
  if (h.kind === "job") return "a Job runs";
  if (h.kind === "run") return h.job === 0 ? "a run holds it, between two Jobs" : "a Job of a run runs";
  return "idle";
};

export const state = (v) => {
  const games = Object.keys(v.holder);
  const shown = games.find((g) => v.exists.includes(Number(g))) ?? games[0];
  let words = `Game ${shown}: ${holderWords(v.holder[shown])}`;
  if (v.failLine[shown] !== 0) words += ", with a failure line";
  const cutLive = Object.keys(v.job).some((j) => v.cut.includes(Number(j)) && live(v.job[j]));
  if (cutLive) words += "; a cut-off Job still runs";
  if (v.pending.delete.length > 0) words += "; a delete passed its check";
  if (v.pending.switch.length > 0) words += "; a switch read its Game";
  return words;
};

export const group = (v) => {
  const busy = Object.values(v.holder).some((h) => h.kind !== "none");
  return busy ? "Busy" : "Idle";
};

export const colors = { Idle: "teal", Busy: "blue" };

const labels = {
  YouStartAJob: "you send an Action",
  TheAcceptCannotRead: "the accept cannot read the story",
  YouStartAutoplay: "you start Autoplay",
  YouPressStop: "you press Stop",
  YouResetTheGame: "you reset the Game",
  TheResetMovesThePointer: "the new Game becomes active",
  YouCreateAGame: "you create a Game",
  TheCreateMovesThePointer: "the created Game becomes active",
  YouDeleteAGame: "you delete another Game, and it is not the active one",
  TheDeleteRemovesTheGame: "the delete removes the Game",
  YouSwitchTheGame: "you switch to another Game, and it exists",
  TheSwitchSetsThePointer: "the switch moves the pointer",
  YouEditAMessage: "you edit a Message",
  YouSwitchOrDeleteASwipe: "you switch a Swipe or delete the last Message",
  YouChangeTheConfig: "you change the config",
  AJobStartsAGeneration: "the Job starts its narrator call",
  TheNarratorAnswers: "the narrator answers",
  TheNarratorFails: "the narrator call fails",
  TheTextIsSaved: "the text is saved",
  TheAnalysisStarts: "the analysis starts",
  TheAnalysisAnswers: "the analysis answers",
  TheStateIsSaved: "the new state is saved",
  TheOptionsRunFollows: "the Options run follows",
  TheJobSkipsTheOptionsRun: "the Job has no Options run",
  TheOptionsRunStarts: "the Options call starts",
  TheOptionsCallEnds: "the Options call ends",
  TheOptionsAreSaved: "the Options are saved",
  TheFailureLineIsSaved: "the failure line is saved",
  TheJobEnds: "the Job ends",
  TheRunAccepts: "the run takes its next step",
  TheRunSeesItsJobEnd: "the run sees its Job end",
  TheRunEnds: "the run ends",
};

export const step = (action) => labels[action] ?? action;
