import type { Lane, Said, Vars } from "../ir.ts";
import { esc, titleLines } from "./mermaid.ts";

export type Move = { action: string; from: Vars; to: Vars };

const ARROW = { call: "->>", answer: "-->>", refused: "-x", failed: "--x" };

export function sequenceLines(
  title: string,
  lanes: Lane[],
  moves: Move[],
  message: (action: string, from: Vars, to: Vars) => Said[],
  broken: string | undefined,
) {
  const used = new Set<string>();
  const body: string[] = [];
  for (const { action, from, to } of moves)
    for (const said of message(action, from, to)) {
      if (said.kind === "note") {
        for (const id of said.over) used.add(id);
        body.push(`  Note over ${said.over.join(",")}: ${esc(said.text)}`);
      } else {
        used.add(said.from);
        used.add(said.to);
        body.push(`  ${said.from}${ARROW[said.kind]}${said.to}: ${esc(said.text)}`);
      }
    }
  if (broken !== undefined) {
    if (!used.size && lanes.length) used.add(lanes[0].id);
    const shown = lanes.filter((l) => used.has(l.id));
    const ends = shown.length > 1 ? [shown[0].id, shown[shown.length - 1].id] : [shown[0].id];
    body.push(`  Note over ${ends.join(",")}: ${esc(`rule ${broken} breaks here`)}`);
  }
  const head: string[] = [];
  for (const lane of lanes)
    if (used.has(lane.id)) head.push(`  participant ${lane.id} as ${esc(lane.label)}`);
  return [...titleLines(title), "sequenceDiagram", ...head, ...body];
}
