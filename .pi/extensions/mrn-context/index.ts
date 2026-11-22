// mrn-context: inject a compact context block (Model, Build Log) into each pi
// turn. Three handlers share one per-source dedup map: session_start resets
// it; before_agent_start collects every source and returns one persistent
// <pi-note> when a source's key is new at prompt time; context injects the
// midTurn sources (the build log) non-persistently when their key changes
// mid-turn — before_agent_start only fires when a prompt is submitted, so
// without this a build run inside the agent loop stayed invisible until the
// next prompt. Both handlers mark the key as seen on emit, so each note
// surfaces exactly once.
//
// The model note posts once per key: at session start (keys reset) and on
// every model change. No periodic re-anchor — after a compaction the
// attribution may drop out of the outgoing context until the next change.
//
// The sources and the key logic live in sources.ts.

import {
  type Source,
  type SourceKeys,
  collectNewLines,
  formatPiNote,
  getBuildLogNote,
  getModelNote,
} from "./sources.ts";

const SOURCES: Source[] = [
  { name: "model", midTurn: false, get: getModelNote },
  { name: "buildLog", midTurn: true, get: getBuildLogNote },
];

const lastKeys: SourceKeys = {};

export default function (pi: any): void {
  pi.on("session_start", () => {
    for (const k of Object.keys(lastKeys)) lastKeys[k] = undefined;
  });

  pi.on("before_agent_start", (_event: unknown, ctx: any) => {
    const lines = collectNewLines(SOURCES, lastKeys, ctx);
    if (lines.length === 0) return undefined;
    return {
      message: {
        customType: "pi-note",
        content: formatPiNote(lines),
        display: false,
      },
    };
  });

  // Within-turn freshness: append the midTurn sources to the outgoing
  // context (non-persistent) the first time their key changes mid-turn.
  // Marking lastKeys here also dedups against before_agent_start — whoever
  // sees the key first emits it, exactly once.
  pi.on("context", (event: any, ctx: any) => {
    const lines = collectNewLines(
      SOURCES.filter((src) => src.midTurn),
      lastKeys,
      ctx,
    );
    if (lines.length === 0) return undefined;
    return {
      messages: [
        ...event.messages,
        {
          role: "custom",
          customType: "pi-note",
          content: formatPiNote(lines),
          display: false,
          timestamp: Date.now(),
        },
      ],
    };
  });
}
