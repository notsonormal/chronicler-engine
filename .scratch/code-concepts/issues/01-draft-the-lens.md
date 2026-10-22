# Draft the lens: the axes where an LLM knows more than one way

Type: research
Status: resolved

## Question

On which axes can a piece of Rust web-server code take more than one valid shape, and what are the poles of each axis? Examples of an axis: error handling (`Result` with error enums, `anyhow`, panics), polymorphism (enum dispatch, `dyn Trait`, generics), concurrency (synchronous core in `spawn_blocking`, async all the way), persistence (transaction script, repository, active record), state (pure functions, mutable services, actors), validation (at the boundary, at every layer).

Cover all three levels: stance, pattern and idiom (map Notes). Give each pole its pretrained name, so it works as a leading word.

Draw on the whole field of software design, not on one catalogue. The lens is only as wide as its sources, and a narrow lens makes every area ticket narrow. Families to cover, as a floor and not a ceiling: architecture styles; application and enterprise patterns; domain modelling; object-oriented and functional design; type-driven design; error handling; concurrency and state ownership; persistence and transactions; module and API design; configuration and composition; web and hypermedia; Rust idioms; testing schools and test design; documentation design. Add every family you find that this list misses. The [pstack principles](../assets/pstack-principles.md) are one small, opinionated example of the form. Use them as a check at the end: they are not a starting list.

Also cover the shapes of tests and of docs, because `tests/` and `docs/diataxis/` are in scope.

Read the code only lightly, to see which axes are live. Do not record what the code does; the area tickets do that.

## Done when

`assets/research-01-lens.md` lists the axes grouped by level and by family. Each axis has two or more named poles, one line each. Every family above has axes, or a line that says why it is not live in this repo. The asset lists the families it added. The asset says in its first lines that it is a starting checklist and that the area tickets record what it misses. It names the axes that it thinks are most likely to show meeting points in this repo, with the reason (inferred).

## Answer

Asset: [research-01-lens.md](../assets/research-01-lens.md). It has 178 axes, each with an ID: stance S1–S26, pattern P1–P69, idiom I1–I50, tests T1–T18 and docs D1–D15.

- The 14 given families all have live axes. The researcher added 11 families: evolution and change management, background work, serialization, data-driven content and templating, LLM-application design, observability, resource limits, quality gates, naming and comments, time/identity/randomness, and security and trust.
- Not live (inferred): distributed systems, authentication and authorisation, microservices, active record and ORM, event sourcing and plugin loading. Internationalisation is also not live (guessed).
- The asset ranks 14 axes that are likely to show meeting points (inferred). The first three are: transaction script, domain model and functional core; the writer rule (single writer, runtime guard, `Arc<Mutex>`); and ports and adapters against layering.
- The pstack check found one gap with no ID: data-first against behaviour-first.

Decisions that the researcher made on points the ticket left open:
- Each axis sits at one level. A family that spans levels is split.
- Tests and docs get their own letters (`T`, `D`).
- Poles that the repo does not use stay in the lens.

**Conflict for the area tickets (open):** tickets 02–09 say "every lens axis has a row". With 178 axes, that is too much for one session. The researcher expects each area ticket to use the likely-axes table and the family heads instead. This needs an owner decision before ticket 02 starts.

**Owner ruling (same session, after the answer above):** the lens is not an input to the research tickets. The researchers must dig the concepts out of the code. A list given in advance points them at what the list names. Tickets 02–10 no longer read the lens. The conflict above is void. The asset stays as a record. The map's "The outside check" fog names it as a possible check after the research.
