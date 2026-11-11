// PROTOTYPE (ticket 03 of .scratch/statechart-models). Throwaway.
export const title = "The gate of one Game, written from the docs only";

const ids = (o) => Object.keys(o ?? {});
const has = (set, x) => Array.isArray(set) && set.includes(x);

const stepWords = {
  narrating: "narrating",
  quantifying: "reading the new text",
  options: "making the Options",
};

const running = (job) => job.pc !== "unborn" && job.pc !== "ended";

const holderWords = (v) => {
  if (!v.gameExists) return "The Game is gone";
  const h = v.holder;
  const step = stepWords[h.step];
  if (h.kind === "job") return step ? `A Job holds the Game, ${step}` : "A Job holds the Game";
  if (h.kind === "run") {
    if (!h.job) return "A run holds the Game between Jobs";
    return step ? `A run holds the Game, its Job ${step}` : "A run holds the Game, its Job starts";
  }
  return v.failure.on ? "Idle with a failure line" : "Idle";
};

// What still runs in the background, as the reader sees it.
const backgroundWords = (v) => {
  const parts = [];
  const jobs = ids(v.jobs).map((k) => [Number(k), v.jobs[k]]);
  if (jobs.some(([j, job]) => running(job) && has(v.cut, j)))
    parts.push("a stopped Job still runs");
  if (jobs.some(([j, job]) => running(job) && !has(v.cut, j) && v.holder.job !== j))
    parts.push("a Job runs with no entry");
  const loops = ids(v.loops).map((k) => [Number(k), v.loops[k]]);
  if (loops.some(([r, loop]) => loop.pc !== "unborn" && loop.pc !== "ended" && has(v.endedRuns, r)))
    parts.push("a stopped run's loop still runs");
  return parts;
};

export const state = (v) => {
  const bg = backgroundWords(v);
  return bg.length ? `${holderWords(v)}; ${bg.join("; ")}` : holderWords(v);
};

export const group = (v) => {
  if (!v.gameExists) return "Gone";
  if (v.holder.kind === "run") return "Autoplay";
  if (v.holder.kind === "job") return "Busy";
  return "Idle";
};

export const colors = { Idle: "teal", Busy: "blue", Autoplay: "purple", Gone: "tan" };

// The Job or the run that a step moved.
const movedJob = (from, to) =>
  ids(to.jobs).find((k) => JSON.stringify(from.jobs[k]) !== JSON.stringify(to.jobs[k]));

export const step = (action, from, to) => {
  const j = movedJob(from, to);
  const stale = j && has(from.cut, Number(j));
  const who = stale ? "the stopped Job" : "the Job";
  switch (action) {
    case "YouSendAnAction": return "you send an Action";
    case "YouStartAutoplay": return "you start Autoplay";
    case "YouPressStop": return "you press Stop";
    case "YouResetTheGame": return "you reset the Game";
    case "YouDeleteTheGame": return "you delete the Game";
    case "YouSwitchToAnotherGame": return "you switch to another Game";
    case "YouSwitchBack": return "you switch back";
    case "YouChangeTheConfig": return null;
    case "YouEditDeleteOrSwitchASwipe": return "you edit, delete or switch a Swipe";
    case "TheEngineRestarts": return "the engine restarts";
    case "TheInputIsSaved": return `${who} saves its Input`;
    case "AStepStarts": return `${who} starts its next step`;
    case "TheNarratorAnswers": return `the narrator answers ${who}`;
    case "TheNarratorCallFails": return `the narrator call of ${who} fails`;
    case "TheAnalysisAnswers": return `the analysis answers ${who}`;
    case "TheOptionsCallAnswers": return `the Options call of ${who} answers`;
    case "TheOptionsCallFails": return `the Options call of ${who} fails`;
    case "TheTextIsSaved": return `${who} saves its text`;
    case "TheStateIsUpdated": return `${who} saves the new state`;
    case "TheStateUpdateFails": return `the state update of ${who} fails`;
    case "TheFailureLineIsSaved": return `${who} saves a failure line`;
    case "TheOptionsAreSaved": return `${who} saves its Options`;
    case "TheOptionsFailureIsSaved": return `${who} saves an Options failure`;
    case "TheOptionsFailureGoesToTheLog": return `the Options failure of ${who} goes to the log`;
    case "ASaveIsRefused": return `a save of ${who} is refused`;
    case "TheJobEnds": return `${who} ends`;
    case "TheRunIsRefused": return "the run finds another holder and ends";
    case "TheRunFinishes": return "the run ends";
    case "TheRunAcceptsItsNextJob": return "the run starts its next Job";
    case "NothingMoreHappens": return "nothing more happens";
    default: return action;
  }
};
