import {
  type CheckedView,
  checked,
  type Lane,
  list,
  type Said,
  type Value,
  type Vars,
} from "../ir.ts";

type Json = boolean | number | string | Json[] | { [key: string]: Json };

const projected = new WeakMap<object, Json>();

export const json = (v: Value): Json => {
  if (typeof v !== "object") return v;
  if (v.kind === "model") return v.name;
  let j = projected.get(v);
  if (j === undefined) {
    if (v.kind === "interval") {
      const members: Json[] = [];
      for (let k = v.lo; k <= v.hi; k++) members.push(k);
      j = members;
    } else if (v.kind === "set") {
      const members: Json[] = [];
      for (const m of v.members) members.push(json(m));
      j = members;
    } else {
      const fields: [string, Json][] = [];
      for (let i = 0; i < v.keys.length; i++) {
        const k = json(v.keys[i]);
        fields.push([typeof k === "object" ? JSON.stringify(k) : String(k), json(v.values[i])]);
      }
      j = Object.fromEntries(fields);
    }
    projected.set(v, j);
  }
  return j;
};

const projectedVars = new WeakMap<Vars, { [name: string]: Json }>();

const jsonVars = (vars: Vars) => {
  let j = projectedVars.get(vars);
  if (j === undefined) {
    const fields: [string, Json][] = [];
    for (const [name, v] of Object.entries(vars)) fields.push([name, json(v)]);
    j = Object.fromEntries(fields);
    projectedVars.set(vars, j);
  }
  return j;
};

const THEME_COLORS = ["blue", "orange", "purple", "yellow", "teal", "tan", "pink"];

const isFunction = (v: unknown): v is (...args: unknown[]) => unknown => typeof v === "function";

const isPlainObject = (v: unknown): v is { [key: string]: unknown } => {
  if (typeof v !== "object" || v === null) return false;
  const proto = Object.getPrototypeOf(v);
  return proto === Object.prototype || proto === null;
};

const isString = (v: unknown) => typeof v === "string";

const VIEW_EXPORTS = ["state", "step", "group", "colors", "title", "lanes", "message"];

const isArray = (v: unknown): v is unknown[] => Array.isArray(v);

const LANE_ID = /^[A-Za-z][A-Za-z0-9_]*$/;

const MERMAID_WORDS = new Set(
  (
    "participant actor as note over left right of loop alt else opt par and critical break end " +
    "rect box autonumber activate deactivate title link links create destroy"
  ).split(" "),
);

const ARROWS = ["call", "answer", "refused", "failed"] as const;

const SAID_KINDS = [...ARROWS, "note"];

function readLanes(lanes: unknown[], literal: (v: unknown) => string): Lane[] {
  if (!lanes.length) throw new Error("The view's lanes are empty. Declare at least one lane.");
  const read: Lane[] = [];
  const seen = new Set<string>();
  for (const lane of lanes) {
    const id = isPlainObject(lane) ? lane.id : undefined;
    const label = isPlainObject(lane) ? (lane.label ?? id) : undefined;
    if (
      !isPlainObject(lane) ||
      typeof id !== "string" ||
      !LANE_ID.test(id) ||
      MERMAID_WORDS.has(id.toLowerCase()) ||
      typeof label !== "string" ||
      !label.trim() ||
      Object.keys(lane).some((k) => k !== "id" && k !== "label")
    )
      throw new Error(
        `The view's lanes have ${literal(lane)}. Give each lane as { id, label }: an id of letters, digits and _ that starts with a letter and is not a Mermaid keyword, and a label if you want one.`,
      );
    if (seen.has(id)) throw new Error(`The view's lanes give the id ${literal(id)} twice.`);
    seen.add(id);
    read.push({ id, label });
  }
  return read;
}

function readSaid(
  item: unknown,
  lanes: Set<string>,
  where: string,
  literal: (v: unknown) => string,
): Said {
  const refuse = (words: string) => new Error(`${where} returned ${literal(item)}. ${words}`);
  if (!isPlainObject(item))
    throw refuse("Each item must be a plain object, such as { call: TEXT, from: LANE, to: LANE }.");
  const kinds = Object.keys(item).filter((k) => SAID_KINDS.includes(k));
  if (kinds.length !== 1)
    throw refuse(`Each item must have exactly one of the keys ${list(SAID_KINDS, "or")}.`);
  const kind = kinds[0];
  const text = item[kind];
  if (typeof text !== "string" || !text.trim()) throw refuse(`The ${kind} must be a text.`);
  const lane = (id: unknown) => {
    if (typeof id !== "string" || !lanes.has(id))
      throw refuse(`It names the lane ${literal(id)}, which lanes does not declare.`);
    return id;
  };
  if (kind === "note") {
    if (Object.keys(item).some((k) => k !== "note" && k !== "over"))
      throw refuse("A note has only the keys note and over.");
    const over = typeof item.over === "string" ? [item.over] : item.over;
    if (!isArray(over) || over.length < 1 || over.length > 2)
      throw refuse("A note is over one lane or two: over: LANE or over: [LANE, LANE].");
    return { kind, text, over: over.map(lane) };
  }
  if (Object.keys(item).some((k) => k !== kind && k !== "from" && k !== "to"))
    throw refuse(`An arrow has only the keys ${kind}, from and to.`);
  return { kind: kind as (typeof ARROWS)[number], text, from: lane(item.from), to: lane(item.to) };
}

type Inspect = (value: unknown, options: { breakLength: number }) => string;

export function checkView(
  view: { readonly [name: string]: unknown },
  inspect: Inspect,
): CheckedView {
  const literal = (v: unknown) =>
    typeof v === "string" ? JSON.stringify(v) : inspect(v, { breakLength: Infinity });
  const extra: string[] = [];
  for (const name of Object.keys(view)) if (!VIEW_EXPORTS.includes(name)) extra.push(name);
  if (extra.length)
    throw new Error(
      `The view exports ${list(extra)}, which the command does not read. Export only ${list(VIEW_EXPORTS)}.`,
    );
  const wrong: string[] = [];
  const pick = <T>(name: string, kind: string, fits: (v: unknown) => v is T) => {
    const v = view[name];
    if (v === undefined || fits(v)) return v;
    wrong.push(`${name} must be ${kind}`);
    return undefined;
  };
  const state = pick("state", "a function", isFunction);
  const step = pick("step", "a function", isFunction);
  const group = pick("group", "a function", isFunction);
  const exported = pick("colors", "a plain object", isPlainObject);
  const title = pick("title", "a string", isString);
  const declared = pick("lanes", "a list", isArray);
  const message = pick("message", "a function", isFunction);
  if (wrong.length) throw new Error(`The view's ${list(wrong)}.`);
  if ((declared === undefined) !== (message === undefined))
    throw new Error(
      "The view exports only one of lanes and message(). A sequence diagram needs both.",
    );
  const lanes = declared && readLanes(declared, literal);
  const ids = new Set<string>();
  for (const lane of lanes ?? []) ids.add(lane.id);
  if (group !== undefined && exported === undefined)
    throw new Error("The view exports group() but no colors. Give each group a color in colors.");
  const colors: { [group: string]: string } = Object.create(null);
  const bad: string[] = [];
  for (const [name, color] of Object.entries(exported ?? {})) {
    if (typeof color === "string" && THEME_COLORS.includes(color)) colors[name] = color;
    else bad.push(`${JSON.stringify(name)} the color ${literal(color)}`);
  }
  if (bad.length)
    throw new Error(
      `The view's colors give ${list(bad)}. Give each group ${list(THEME_COLORS, "or")}.`,
    );
  const named =
    group &&
    ((vars: Vars) => {
      const g = group(jsonVars(vars));
      if (g !== undefined && !Object.hasOwn(colors, typeof g === "symbol" ? g : String(g)))
        throw new Error(
          `The view's group() returned ${literal(g)}, which colors does not name. Give each group a color in colors.`,
        );
      return g;
    });
  return {
    state: state && ((vars: Vars) => state(jsonVars(vars))),
    step:
      step &&
      ((action: string, from: Vars, to: Vars) => step(action, jsonVars(from), jsonVars(to))),
    group: named,
    colors,
    title,
    lanes,
    message:
      message &&
      ((action: string, from: Vars, to: Vars) => {
        const said = message(action, jsonVars(from), jsonVars(to));
        const where = `The view's message() for ${action}`;
        if (!isArray(said))
          throw new Error(
            `${where} returned ${literal(said)}. Return a list of calls, answers, refused and failed arrows, and notes.`,
          );
        return said.map((item) => readSaid(item, ids, where, literal));
      }),
    [checked]: true,
  };
}
