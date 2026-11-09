import { type ConfigSection, RERUN_STATES, type Spec } from "../ir.ts";

export const SCENARIO_END = "ChartEnd";

export function configText(sections: ConfigSection[], extra: string[]) {
  const lines: string[] = [];
  for (const s of sections) lines.push(`${s.key} ${s.tokens.join(" ")}`);
  lines.push(...extra);
  return lines.join("\n");
}

export const boundRun = (module: string) =>
  [
    `---- MODULE ChartBound ----`,
    `EXTENDS ${module}`,
    `ChartTLC == INSTANCE TLC`,
    `ChartNaturals == INSTANCE Naturals`,
    `ChartBound == ChartNaturals!<(ChartTLC!TLCGet("distinct"), ${RERUN_STATES})`,
    `====`,
  ].join("\n");

// The lines of `head` in scenarioRun: step k of a scenario is on line k + SCENARIO_STEP_LINE.
export const SCENARIO_STEP_LINE = 5;

export function scenarioRun(
  { module, variables }: Spec,
  name: string,
  { init, next }: { init: string; next: string },
  steps: string[],
) {
  const arms: string[] = [];
  for (const [k, step] of steps.entries())
    arms.push(`  \\/ (ChartStep = ${k} /\\ (${step.trim()}) /\\ ${next} /\\ ChartStep' = ${k + 1})`);
  const head = [
    `---- MODULE ${name} ----`,
    `EXTENDS ${module}`,
    `VARIABLE ChartStep`,
    `ChartInit == ${init} /\\ ChartStep = 0`,
    `ChartNext ==`,
  ];
  return [
    ...head,
    ...arms,
    `ChartSpec == ChartInit /\\ [][ChartNext]_<<${[...variables, "ChartStep"].join(", ")}>>`,
    `${SCENARIO_END} == ChartStep # ${steps.length}`,
    `====`,
  ].join("\n");
}

const SCENARIO_DROPS = [
  "SPECIFICATION",
  "INIT",
  "NEXT",
  "CONSTRAINT",
  "ACTION_CONSTRAINT",
  "VIEW",
  "SYMMETRY",
];

const constantEntries = (tokens: string[]) => {
  const entries: string[][] = [];
  let i = 0;
  const value = () => {
    if (tokens[i] !== "{") return void i++;
    for (let depth = 0; i < tokens.length; i++) {
      depth += tokens[i] === "{" ? 1 : tokens[i] === "}" ? -1 : 0;
      if (!depth) return void i++;
    }
  };
  while (i < tokens.length) {
    const start = i;
    while (i < tokens.length && tokens[i] !== "=" && tokens[i] !== "<-") {
      if (tokens[i] === "(") while (i < tokens.length && tokens[i] !== ")") i++;
      i++;
    }
    i++;
    if (tokens[i] === "[") i += 3;
    value();
    entries.push(tokens.slice(start, i));
  }
  return entries;
};

export function scenarioConfig(
  sections: ConfigSection[],
  invariants: string[],
  constants: { [name: string]: string },
) {
  const lines: string[] = [];
  const set = new Set<string>();
  for (const s of sections) {
    if (SCENARIO_DROPS.includes(s.key)) continue;
    if (s.key !== "CONSTANT") {
      lines.push(`${s.key} ${s.tokens.join(" ")}`);
      continue;
    }
    const kept: string[] = [];
    for (const entry of constantEntries(s.tokens)) {
      const name = entry[0];
      if (Object.hasOwn(constants, name) && entry[1] === "=") {
        set.add(name);
        kept.push(`${name} = ${constants[name]}`);
      } else kept.push(entry.join(" "));
    }
    lines.push(`CONSTANT ${kept.join(" ")}`);
  }
  const missing = Object.keys(constants).filter((name) => !set.has(name));
  lines.push("SPECIFICATION ChartSpec", `INVARIANT ${[...invariants, SCENARIO_END].join(" ")}`);
  return { text: lines.join("\n"), missing };
}

export function chartRun({ module, variables, invariants }: Spec, tag: string, bound: number) {
  const every: string[] = [];
  for (const name of invariants) every.push(`(IF ~${name} THEN {"${name}"} ELSE {}) \\cup `);
  return [
    `---- MODULE ChartRun ----`,
    `EXTENDS ${module}`,
    `ChartTLC == INSTANCE TLC`,
    `ChartNaturals == INSTANCE Naturals`,
    `ChartBudget == ChartNaturals!<=(ChartTLC!TLCGet("distinct"), ${bound})`,
    `ChartBroken == ${every.join("")}{}`,
    `ChartMark == ChartBroken = {} \\/ ChartTLC!PrintT(<<"${tag}", <<${variables.join(", ")}>>, ChartBroken>>)`,
    `====`,
  ].join("\n");
}
