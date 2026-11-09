import { mermaidLines } from "./backends/mermaid.ts";
import { plainText } from "./backends/plain-text.ts";
import { report } from "./backends/report.ts";
import { type Move, sequenceLines } from "./backends/sequence.ts";
import { readCounterexample } from "./frontends/marks.ts";
import { actionName } from "./frontends/scenarios.ts";
import {
  type Answers,
  type Bug,
  CHART_STATES,
  type Chart,
  type CheckedView,
  type CheckResult,
  type Graph,
  PART,
  ruleKey,
  type ScenarioResult,
  type Story,
  type TraceState,
  type View,
  type Walk,
} from "./ir.ts";
import { asParents, ask, labelGraph, storiesOf } from "./passes/label.ts";
import { lift } from "./passes/lift.ts";
import { loopCheck } from "./passes/loop-check.ts";
import { bugs, drawOrder, walk } from "./passes/reach.ts";
import { regionsLie } from "./passes/region-check.ts";
import { drawRegions } from "./passes/regions.ts";

function* inParts(lines: string[]) {
  let start = 0;
  let size = 0;
  for (let i = 0; i < lines.length; i++) {
    size += lines[i].length + 1;
    if (size > PART || i === lines.length - 1) {
      yield (start ? "\n" : "") + lines.slice(start, i + 1).join("\n");
      start = i + 1;
      size = 0;
    }
  }
}

const shortestFirst = (a: { path: unknown[] }, b: { path: unknown[] }) =>
  a.path.length - b.path.length;

export function project(graph: Graph, walked: Walk, view: View, found: Bug[]) {
  const { order, rank } = drawOrder(walked);
  const draw = (answers: Answers) => {
    const labels = labelGraph(graph, walked, answers, rank, found);
    return { labels, chart: drawRegions(walked, labels, order, found, view) };
  };
  const answers = ask(graph, walked, view);
  const first = draw(answers);
  const lie = regionsLie(first.chart.regions, first.labels, walked, order);
  const { labels, chart } = lie ? draw(asParents(walked, answers)) : first;
  loopCheck(chart, labels, walked, found);
  return {
    chart: lift(lie ? { ...chart, lie } : chart),
    stories: storiesOf(graph, found, walked, first.labels),
  };
}

export const chartSize = (chart: Chart) => {
  let size = 0;
  for (const r of chart.regions) size += r.states.length;
  return size;
};

function draw(result: CheckResult, view: CheckedView) {
  const walked = walk(result.graph);
  const found = [...(result.paths ?? bugs(result.graph, walked))].sort(shortestFirst);
  for (const p of result.properties) if (p.bug) found.push({ rule: p.rule, ...p.bug });
  const { chart, stories } =
    found.length || !result.paths
      ? project(result.graph, walked, view, found)
      : { chart: undefined, stories: new Map<Bug, Story>() };
  return { found, chart, stories };
}

export function compile(whole: CheckResult, view: CheckedView) {
  let result = whole;
  let made = draw(result, view);
  let over: number | undefined;
  if (whole.fallback && made.chart && chartSize(made.chart) > CHART_STATES) {
    over = chartSize(made.chart);
    result = { ...whole, ...whole.fallback, drawing: undefined, fallback: undefined };
    made = draw(result, view);
  }
  const { found, chart, stories } = made;
  let text = report(result, found, chart, stories, over);
  const files = new Map<string, Iterable<string | Uint8Array>>();
  if (chart) files.set("chart.mmd", inParts(mermaidLines(chart)));
  const sequences: string[] = [];
  const { lanes, message } = view;
  if (lanes && message)
    for (const { rule, outcome, messages } of result.rules)
      if (outcome === "broken" && messages) {
        const file = `sequence-${rule.name}.mmd`;
        const trace = readCounterexample(messages);
        const moves = movesOf(trace, (i) => trace[i].action);
        const title = `Path to the ${rule.name} bug`;
        files.set(file, inParts(sequenceLines(title, lanes, moves, message, rule.name)));
        sequences.push(`- ${file}: the path to ${rule.name}, as TLC reports it.`);
      }
  if (sequences.length) text += ["", "", "## Sequence diagrams", ...sequences].join("\n");
  files.set("report.md", [text]);
  files.set("tlc.out", plainText(result.messages));
  const drawn = new Set<string>();
  for (const b of found) if (b.rule.kind !== "property") drawn.add(ruleKey(b.rule));
  for (const { rule, outcome, messages } of [...result.rules, ...result.properties])
    if (outcome === "broken" && messages && !drawn.has(ruleKey(rule)))
      files.set(`tlc-${rule.name}.out`, plainText(messages));
  return { report: text, files, exit: result.exit };
}

const movesOf = (trace: TraceState[], actionAt: (i: number) => string) => {
  const moves: Move[] = [];
  for (let i = 1; i < trace.length; i++)
    moves.push({ action: actionAt(i), from: trace[i - 1].vars, to: trace[i].vars });
  return moves;
};

export function compileScenarios({ spec, configFile, runs, exit }: ScenarioResult, view: CheckedView) {
  const { lanes, message } = view;
  if (!lanes || !message)
    throw new Error("--scenarios needs a view that exports lanes and message().");
  const files = new Map<string, Iterable<string | Uint8Array>>();
  const out = [
    `# ${spec.module}: scenarios`,
    "",
    `TLC took the steps of each scenario on ${configFile}, with the constants of the scenario, and checked each INVARIANT in each state on the way. A scenario does not check a PROPERTY.`,
  ];
  for (const { scenario, trace, broken, messages } of runs) {
    const { name, title, steps, rule, spec: specScenario } = scenario;
    const moves = movesOf(trace, (i) => actionName(steps[i - 1]));
    files.set(`${name}.mmd`, inParts(sequenceLines(title, lanes, moves, message, broken)));
    files.set(`tlc-${name}.out`, plainText(messages));
    out.push("", `## ${name}`, title);
    const shows: string[] = [];
    if (rule !== undefined) shows.push(`Rule: ${rule}.`);
    if (specScenario !== undefined) shows.push(`Spec scenario: ${specScenario}.`);
    if (shows.length) out.push(shows.join(" "));
    const reached = trace.length - 1;
    out.push(
      broken === undefined
        ? `It takes all ${steps.length} steps, and every rule holds. The diagram is ${name}.mmd.`
        : `${broken} breaks ${reached ? `after step ${reached} of ${steps.length}, ${steps[reached - 1].trim()}` : "in the start state"}. The diagram ${name}.mmd stops there.`,
    );
  }
  const text = out.join("\n");
  files.set("report.md", [text]);
  return { report: text, files, exit };
}
