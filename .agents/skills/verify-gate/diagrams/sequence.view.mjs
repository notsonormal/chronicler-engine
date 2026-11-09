// The lanes and the messages of the gate, for the sequence diagrams of `/statechart`: the
// scenarios of `scenarios.mjs`, and the path of a broken rule. An action has no arguments
// here, so each message reads the Job, the Autoplay run or the Game from the change between the two
// states.
export const lanes = [
  { id: "You", label: "Player" },
  { id: "C", label: "GameCatalogue" },
  { id: "P", label: "ActiveGame" },
  { id: "G", label: "GameWriter" },
  { id: "R1", label: "Autoplay run 1" },
  { id: "R2", label: "Autoplay run 2" },
  { id: "J1", label: "Job 1" },
  { id: "J2", label: "Job 2" },
  { id: "J3", label: "Job 3" },
  { id: "L", label: "LLM" },
];

const changed = (a = {}, b = {}) => {
  for (const k of Object.keys(b)) if (JSON.stringify(a[k]) !== JSON.stringify(b[k])) return k;
  return undefined;
};
const added = (a, b) => b.find((x) => !a.includes(x));

// The Game that a request names: the first Game whose holder or failure fields change,
// else the active Game.
const requestGame = (from, to) =>
  changed(from.holder, to.holder) ??
  changed(from.failLine, to.failLine) ??
  changed(from.optFail, to.optFail) ??
  String(from.active);

const holderWords = (h) => {
  if (h.kind === "job") return `Job ${h.job}`;
  if (h.kind === "run") return `Autoplay run ${h.run}`;
  return "nobody";
};

const failures = (v, g) => {
  const line = v.failLine[g] === 0 ? "no failure line" : "a failure line";
  const options = v.optFail[g] === 0 ? "no Options failure" : "an Options failure";
  return `The Game shows ${line} and ${options}`;
};

const jobKinds = { action: "an Action", options: "Options", impersonate: "an Impersonate Input", continue: "a Narration" };

const gone = (v, g) => {
  const rows = v.exists.includes(Number(g)) ? `Game ${g} still exists` : `Game ${g} is gone`;
  return v.holder[g].kind === "none" ? `${rows}, with no holder` : `${rows}, and ${holderWords(v.holder[g])} still holds it`;
};

export const message = (action, from, to) => {
  const j = changed(from.job, to.job);
  const r = changed(from.run, to.run);
  const J = `J${j}`;
  const R = `R${r}`;
  const moved =
    j !== undefined &&
    to.job[j].pc === "done" &&
    to.job[j].fail === "moved" &&
    from.job[j].pc !== "done";
  const refused = (what) => [
    { refused: `${what}: refused, the holder is not Job ${j}`, from: J, to: "G" },
    { note: `Job ${j} ends and writes nothing more`, over: J },
  ];
  const step = (call, llm) =>
    moved
      ? refused("set the step")
      : [
          { call, from: J, to: "G" },
          { call: llm, from: J, to: "L" },
        ];
  switch (action) {
    case "YouStartAJob": {
      const kind = to.job[from.nextJob].kind;
      return [
        { call: kind === "options" ? "ask for Options" : "send an Action", from: "You", to: "G" },
        { note: `Job ${from.nextJob} holds Game ${to.job[from.nextJob].g}`, over: "G" },
      ];
    }
    case "TheAcceptCannotRead":
      return [
        { call: "send an Action", from: "You", to: "G" },
        { note: "the accept cannot read the story: it saves a failure line, and no Job starts", over: "G" },
      ];
    case "YouStartAutoplay":
      return [
        { call: "start Autoplay", from: "You", to: "G" },
        { note: `Autoplay run ${from.nextRun} holds Game ${to.run[from.nextRun].g}`, over: "G" },
      ];
    case "YouPressStop": {
      const g = requestGame(from, to);
      const h = from.holder[g];
      const what =
        h.kind === "none" ? "the Game is idle" : `the holder (${holderWords(h)}) goes, and its work is cut off`;
      return [
        { call: "press Stop", from: "You", to: "G" },
        { note: `${what}. ${failures(to, g)}`, over: "G" },
      ];
    }
    case "YouResetTheGame": {
      const n = added(from.exists, to.exists);
      return [
        { call: `reset Game ${from.active}`, from: "You", to: "C" },
        { call: "take the lock of the pointer", from: "C", to: "P" },
        { call: `create Game ${n}, remove Game ${from.active}`, from: "C", to: "G" },
        { note: `in one hold: Game ${n} exists. ${gone(to, from.active)}`, over: "G" },
      ];
    }
    case "TheResetMovesThePointer":
      return [
        { call: `make Game ${to.active} active`, from: "C", to: "P" },
        { note: `Game ${to.active} is active. The lock goes`, over: "P" },
      ];
    case "YouCreateAGame": {
      const n = added(from.exists, to.exists);
      return [
        { call: "create a Game", from: "You", to: "C" },
        { call: "take the lock of the pointer", from: "C", to: "P" },
        { call: `create Game ${n}`, from: "C", to: "G" },
      ];
    }
    case "TheCreateMovesThePointer":
      return [
        { call: `make Game ${to.active} active`, from: "C", to: "P" },
        { note: `Game ${to.active} is active. The lock goes`, over: "P" },
      ];
    case "YouDeleteAGame": {
      const g = added(from.pending.delete, to.pending.delete);
      return [
        { call: `delete Game ${g}`, from: "You", to: "C" },
        { call: "take the lock of the pointer", from: "C", to: "P" },
        { answer: `Game ${from.active} is active`, from: "P", to: "C" },
      ];
    }
    case "TheDeleteRemovesTheGame": {
      const g = added(to.pending.delete, from.pending.delete);
      return [
        { call: `remove Game ${g}`, from: "C", to: "G" },
        { note: `in one hold: ${gone(to, g)}`, over: "G" },
        { note: "the lock of the pointer goes", over: "P" },
      ];
    }
    case "YouSwitchTheGame": {
      const g = added(from.pending.switch, to.pending.switch);
      return [
        { call: `switch to Game ${g}`, from: "You", to: "C" },
        { call: "take the lock of the pointer", from: "C", to: "P" },
        { note: `Game ${g} exists`, over: "C" },
      ];
    }
    case "TheSwitchSetsThePointer":
      return [
        { call: `make Game ${to.active} active`, from: "C", to: "P" },
        { note: `Game ${to.active} is active. The lock goes`, over: "P" },
      ];
    case "YouEditAMessage":
      return [
        { call: "edit a Message", from: "You", to: "G" },
        { note: to.storyBusy ? "the Game is busy, and the edit lands" : "the Game is idle, so the edit lands", over: "G" },
      ];
    case "YouSwitchOrDeleteASwipe":
      return [
        { call: "switch a Swipe", from: "You", to: "G" },
        {
          note: `the Swipe switch lands. ${failures(to, requestGame(from, to))}`,
          over: "G",
        },
      ];
    case "YouChangeTheConfig":
      return [
        { call: "change the config", from: "You", to: "G" },
        {
          note: to.cfgRun
            ? "an Autoplay run holds the Game, and the config write lands"
            : "no Autoplay run holds the Game, so the config write lands",
          over: "G",
        },
      ];
    case "AJobStartsAGeneration": {
      const first = from.job[j].kind === "action" && from.job[j].gen === "first";
      return step(first ? "save the Input, set the step" : "set the step", "narrator call");
    }
    case "TheNarratorAnswers":
      return [{ answer: "the text", from: "L", to: J }];
    case "TheNarratorFails":
      return [{ failed: "the call fails", from: "L", to: J }];
    case "TheTextIsSaved":
      return moved ? refused("save the text") : [{ call: "save the text", from: J, to: "G" }];
    case "TheAnalysisStarts":
      return step("set the step", "Quantifier call");
    case "TheAnalysisAnswers":
      return [{ answer: "the Quantifier result", from: "L", to: J }];
    case "TheStateIsSaved": {
      if (moved) return refused("save the state");
      const fires = to.job[j].gen === "event" && from.job[j].gen === "first";
      return [
        { call: "save the state", from: J, to: "G" },
        ...(fires ? [{ note: "a Trigger fired: an event Narration follows", over: J }] : []),
      ];
    }
    case "TheOptionsRunFollows":
      return [{ note: "the Options run follows", over: J }];
    case "TheJobSkipsTheOptionsRun":
      return [{ note: "the Job has no Options run", over: J }];
    case "TheOptionsRunStarts":
      return step("set the step", "Options call");
    case "TheOptionsCallEnds":
      return to.job[j].optRes === "offered"
        ? [{ answer: "the Options", from: "L", to: J }]
        : [{ failed: "the call fails", from: "L", to: J }];
    case "TheOptionsAreSaved":
      if (moved) return refused("save the Options");
      return [
        {
          call: from.job[j].optRes === "unavailable" ? "save the Options failure" : "save the Options",
          from: J,
          to: "G",
        },
      ];
    case "TheFailureLineIsSaved":
      return moved
        ? refused("save the failure line")
        : [{ call: "save the failure line", from: J, to: "G" }];
    case "TheJobEnds": {
      const h = to.holder[from.job[j].g];
      const stays =
        h.kind === "none"
          ? "the holder goes"
          : h.kind === "job"
            ? `the holder is Job ${h.job}, so it stays`
            : `the holder is Autoplay run ${h.run}, so it stays`;
      return [
        { call: "let go of the Game", from: J, to: "G" },
        { note: stays, over: "G" },
      ];
    }
    case "TheRunAccepts": {
      const b = to.run[r];
      let answer;
      if (b.pc === "job") answer = `accepted: Job ${b.job} writes ${jobKinds[to.job[b.job].kind]}`;
      else if (b.end === "budget") answer = "the budget is spent. The holder goes";
      else if (b.end === "notAccepted") answer = "the story cannot be read. A failure line, and the holder goes";
      else answer = "the Autoplay run holds the Game no more";
      return [
        { call: "accept the next Job", from: R, to: "G" },
        ...(b.done > from.run[r].done ? [{ note: `cycle ${b.done} is counted`, over: "G" }] : []),
        { answer, from: "G", to: R },
      ];
    }
    case "TheRunSeesItsJobEnd": {
      const b = to.run[r];
      let note = "its Job ended, so the Autoplay run accepts the next Job";
      if (b.pc === "end") note = b.end === "jobFailed" ? "its Job failed, so the Autoplay run ends" : "its Job was cut off, so the Autoplay run ends";
      return [{ note, over: R }];
    }
    case "TheRunEnds": {
      const h = to.holder[from.run[r].g];
      let note = "the holder goes";
      if (h.kind !== "none") note = `the holder is ${holderWords(h)}, so it stays`;
      else if (from.holder[from.run[r].g].kind === "none") note = "the Game is already idle";
      return [
        { call: "end the Autoplay run", from: R, to: "G" },
        { note, over: "G" },
      ];
    }
  }
  return [{ note: action, over: "G" }];
};
