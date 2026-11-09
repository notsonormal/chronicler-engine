import { isRecord, list, type Scenario } from "../ir.ts";

type Inspect = (value: unknown, options: { breakLength: number }) => string;

const NAME = /^[a-z][a-z0-9-]*$/;
const ACTION = /^[A-Za-z_][\w!]*/;
const CONSTANT = /^\w*[A-Za-z]\w*$/;
const KEYS = ["title", "steps", "constants", "rule", "spec"];

export const actionName = (step: string) => ACTION.exec(step.trim())?.[0] ?? step;

export function checkScenarios(
  module: { readonly [name: string]: unknown },
  inspect: Inspect,
): Scenario[] {
  const literal = (v: unknown) =>
    typeof v === "string" ? JSON.stringify(v) : inspect(v, { breakLength: Infinity });
  const extra = Object.keys(module).filter((k) => k !== "scenarios");
  if (extra.length)
    throw new Error(
      `The scenarios file exports ${list(extra)}, which the command does not read. Export only scenarios.`,
    );
  const all = module.scenarios;
  if (!isRecord(all) || !Object.keys(all).length)
    throw new Error(
      "The scenarios file must export scenarios: an object with one entry for each scenario, keyed by its name.",
    );
  const read: Scenario[] = [];
  for (const [name, entry] of Object.entries(all)) {
    const refuse = (words: string) => new Error(`The scenario ${literal(name)} ${words}`);
    if (!NAME.test(name))
      throw refuse(
        "has a name that cannot be a file name. Use lowercase letters, digits and -, and start with a letter.",
      );
    if (!isRecord(entry))
      throw refuse("must be an object with title, steps and, if you want, constants, rule and spec.");
    const unknown = Object.keys(entry).filter((k) => !KEYS.includes(k));
    if (unknown.length)
      throw refuse(`has ${list(unknown)}, which the command does not read. Use only ${list(KEYS)}.`);
    const { title, steps, constants = {}, rule, spec } = entry;
    if (typeof title !== "string" || !title.trim()) throw refuse("must have a title.");
    if (!Array.isArray(steps) || !steps.length)
      throw refuse("must have steps: a list of TLA+ actions, such as \"YouPressStop(1)\".");
    for (const step of steps)
      if (typeof step !== "string" || !ACTION.test(step.trim()) || /[\n\r]/.test(step))
        throw refuse(
          `has the step ${literal(step)}. Write each step on one line, as a TLA+ action that starts with its name.`,
        );
    if (!isRecord(constants))
      throw refuse("must give constants as an object, such as { MaxUploads: \"1\" }.");
    const values: { [name: string]: string } = {};
    for (const [constant, value] of Object.entries(constants)) {
      if (!CONSTANT.test(constant))
        throw refuse(`sets ${literal(constant)}, which is not a constant name.`);
      if (typeof value !== "string" || !value.trim() || /[\n\r]/.test(value))
        throw refuse(
          `sets ${constant} to ${literal(value)}. Give each value as config text on one line, such as "{1, 2}".`,
        );
      values[constant] = value;
    }
    if (rule !== undefined && (typeof rule !== "string" || !CONSTANT.test(rule)))
      throw refuse(`names the rule ${literal(rule)}, which is not a rule name.`);
    if (spec !== undefined && (typeof spec !== "string" || !spec.trim()))
      throw refuse(`names the spec scenario ${literal(spec)}. Give it as a text.`);
    read.push({ name, title, steps: steps as string[], constants: values, rule, spec });
  }
  return read;
}
