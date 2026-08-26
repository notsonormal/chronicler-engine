# Assess AI-language and skill patterns for LLM interaction

Type: research
Status: resolved
Assignee: pi
Blocked by: 01

## Question

What patterns do `AIR-J` (AI-readable language) and `speclj-structure-check` (Claude skill for spec structure) encode for LLM interaction, and are any applicable to Chronicler Engine's prompting, LLM recorder, or skill/agent definitions?

## Answer

### AIR-J: what it is and what it solves

AIR-J is an AI-first, JVM-targeting programming language. Its primary artifact is a canonical, typed, effect-tracked intermediate representation (IR) meant to be written, transformed, checked, and lowered by software agents rather than by humans. The core principle is "one meaning, one representation".

The problem it solves is representation drift and ambiguity when AI agents generate or manipulate code. By persisting programs as canonical s-expressions with explicit imports, exports, types, effects, control flow, mutation, and Java interop, an agent has far less freedom to produce semantically equivalent but syntactically different outputs. This reduces re-analysis cost and makes transformations mechanically checkable.

AIR-J is parsed with a Clojure-based parser (`src/airj/parser.clj`) that reads s-expressions via `clojure.edn`, validates counts/tags, and turns them into an explicit AST. It is then normalized, type-checked, effect-checked, and lowered to JVM bytecode. The language includes first-class design-by-contract forms (`requires`, `ensures`, `invariants`) and structured diagnostics with machine-readable codes, severities, and source spans (`formal-v0-spec.md` §16).

### speclj-structure-check: what it does as a Claude skill

`speclj-structure-check` is a small static-analysis tool that catches Speclj test-structure mistakes before the tests run. It scans Clojure spec files, tracks parentheses and string/comment/regex context, builds a tree of Speclj forms (`describe`, `context`, `it`, `before`, etc.), and reports nesting violations such as `(it)` inside `(it)` or `(describe)` inside `(describe)`.

Packaged as a Claude skill in `.claude/skills/speclj-structure-check/SKILL.md`, it defines:
- When to use: after every spec-file edit, before running tests.
- How to run: `clj -M:spec-structure-check <file-or-dir>` with an optional `--tree` flag.
- What it checks: explicit nesting rules.
- Workflow: edit, check structure, fix parens, then run tests.

The skill is therefore a lightweight pre-flight validator inserted into the agent's workflow, not a test runner itself.

### General patterns for LLM interaction, structured output, validation, and skill/agent definitions

From both repositories, the following patterns emerge:

| Pattern | Source | Meaning for LLM interaction |
| --- | --- | --- |
| Canonical representation | AIR-J | Reduce output variance by giving the model a single, unambiguous target format. |
| Explicit contracts | AIR-J `requires`/`ensures`/`invariants` | Validate that generated artifacts satisfy declared pre/post-conditions. |
| Effect tracking | AIR-J `effects` clauses | Track what side-effects a generated program may perform; analogous to tracking tool/agent side effects. |
| Structured diagnostics | AIR-J `Diagnostic` / formal spec §16 | Return machine-readable error codes, not only human text, so another agent can repair. |
| Lightweight pre-execution check | speclj-structure-check | Validate structure before expensive execution or tests. |
| Skill as workflow step | speclj-structure-check `SKILL.md` | Document when/how/why to run a tool, not just what it does. |

### Comparison against Chronicler Engine

Chronicler Engine's LLM layer is organized around narrative generation rather than code generation, so the mapping is indirect.

**Prompting (`src/application/prompting/`).**
- `assembler.rs` builds prompts from typed layers (`PromptLayer`) — System, GameState, NpcCards, Persona, WorldInfo, History, User, Guide. This is already a form of structured prompt assembly.
- `types.rs` enumerates layers explicitly; `PresetField` enumerates preset parts. Both are more human-facing XML/text than canonical machine IR.
- `builders/sections.rs` wraps preset fields in XML tags and renders templates.
- `sanitize.rs` strips leaked reasoning artifacts from model output.
- `token_budget.rs` and `utils/context.rs` enforce token budgets and truncate history to fit.
- `prompt_merge.rs` merges system + user for models that ignore the system role.

AIR-J's lesson for prompting is not to switch to s-expressions, but to make the *model-facing output format* more canonical: fewer free-form natural-language instructions, more explicit schema and examples. Chronicler already uses XML tags, which is a step in that direction. The gap is that output-format enforcement still relies on preset text rather than a formal, validated schema.

**LLM recorder (`src/application/llm_recorder.rs`).**
- `LlmCallRecorder` wraps an `LlmProvider`, calls `complete`, sanitizes output, and saves a forensic `LlmMessage`.
- It does not validate the *shape* of the response against a contract; it only strips artifacts and records raw request/response JSON.

AIR-J's structured-diagnostic pattern could extend the recorder: capture not only raw JSON but also a parsed/validated result or a machine-readable validation failure, so downstream agents can act on it.

**Agents (`src/application/agents/`).**
- `trait_def.rs` defines a minimal `Agent` trait: name, phase, backend selector, execute.
- `registry.rs` constructs agents from config and dispatches by phase.
- `quantifier/` is the concrete example: it builds a prompt, calls the LLM, and expects a JSON result that is parsed elsewhere.

AIR-J's contract model maps naturally here: the quantifier (and future agents) could declare explicit output schemas and validation contracts. Currently the quantifier relies on system-instruction text (`Respond ONLY with the JSON format specified in the system instructions`) and downstream parsing. A formal schema + validator would make the agent's expected output shape checkable.

**Prompt presets (`src/domain/model/prompt_preset.rs`).**
- `PromptPreset` has free-text fields (`role`, `instructions`, `writing_style`, `output_format`) and renders them into XML-wrapped parts.
- `PresetType` distinguishes System / Quantifier / Impersonate presets.

This is a configuration-driven templating system. AIR-J's canonical-IR idea is overkill for author-edited narrative presets, but the *explicit-field* approach aligns with AIR-J's preference for named, sorted, unambiguous declarations.

**LLM provider port (`src/application/ports/llm_provider.rs`).**
- Defines `LlmProvider` trait and `LlmCallResult`, with named agent constants (`AGENT_NARRATOR`, `AGENT_QUANTIFIER`, etc.).
- Transport-only: it does not know about prompt structure or output schema.

This is a clean seam. AIR-J's effect-tracking idea could be added at this boundary if the engine ever exposes LLM-powered tools that perform side effects, but for narrative generation the provider port should stay transport-only.

### Applicability ratings

| Repository / pattern | Applicability to Chronicler | Rationale |
| --- | --- | --- |
| AIR-J language / s-expression IR | Irrelevant as code | Chronicler does not generate or lower code. |
| AIR-J canonical-output principle | Pattern to adopt | Output schemas for agents (quantifier, future tools) should be canonical and machine-validated. |
| AIR-J design-by-contract (`requires`/`ensures`/`invariants`) | Pattern to adopt | Add explicit output contracts and validation for agent results instead of relying only on prompt text. |
| AIR-J structured diagnostics | Pattern to adopt | Recorder and agents could emit machine-readable parse/validation failures. |
| AIR-J effect tracking | Irrelevant for now | Narrative generation is not effect-tracked; only relevant if LLM agents gain side-effecting tools. |
| speclj-structure-check tool | Directly usable only if using Speclj | Chronicler is Rust, not Clojure. |
| speclj-structure-check as Claude skill packaging | Already followed / pattern to adopt | The repo's `SKILL.md` (when/how/why) matches Chronicler's skill conventions. The structural-check workflow could inspire a pre-test Rust doc/spec linter, but that is outside LLM interaction. |

### Conclusion and recommendations

Neither repository is directly usable in Chronicler Engine today. AIR-J is a programming language, not an LLM interaction library, and `speclj-structure-check` is tied to Speclj/Clojure.

However, AIR-J encodes the most relevant pattern: **treat the model's output as a canonical artifact that should be explicitly typed, validated, and diagnosed.** For Chronicler, the practical adoption is to formalize output schemas and validation for agent results, starting with the quantifier, rather than relying solely on natural-language instructions in presets.

Recommended follow-up direction (no new ticket created, because this needs a deliberate product decision):
- Define JSON Schema or equivalent output contracts for each `ExecutionPhase` agent.
- Add a validation step in the agent executor or `LlmCallRecorder` that parses the model response against the contract and returns a structured `Diagnostic` on failure.
- Keep prompts human-readable where it matters for narrative voice, but make the *machine-interpretable* parts of the prompt (entity lists, room lists, output-format examples) canonical and deterministic.

No new issue is recommended from this research alone; the pattern should be folded into future agent-output work when a concrete agent or schema change is planned.
