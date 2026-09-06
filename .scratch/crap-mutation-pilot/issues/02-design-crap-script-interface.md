# Design the CRAP report script interface and output

Type: grilling
Status: open
Assignee:
Blocked by: 01

## Question

What is the interface, output shape, and configuration of `scripts/crap_report.py` (or chosen form), and how does it relate to the existing coverage scripts?

## Context

Ticket 01 determines the data sources and whether a companion Rust binary is needed. This ticket decides the user-facing design once that is known.

Decisions to make:

- **Language and shape:** single Python script (consistent with `scripts/`), Python script that shells out to a CC tool, or Python script plus a small Rust binary. Prefer the simplest form the CC source allows.
- **Input:** how the coverage JSON is located or generated (reuse `parse_coverage.py` paths? add a `--generate` flag that runs `cargo llvm-cov nextest`?).
- **Output format:** a markdown table sorted by CRAP descending, with columns for file, function, lines, CC, coverage %, CRAP. Plus optional JSON sidecar (see map's Not-yet-specified).
- **Output path:** `tmp/crap/report.md` (mirrors `tmp/coverage/` convention).
- **Configuration:** a TOML/JSON config for ignore regexes (reuse `parse_coverage.py`'s ignore pattern?) and an optional soft CRAP threshold for highlighting, **not** for gating.
- **Relationship to existing scripts:** extend `parse_coverage.py`/`coverage_summary.py`, or keep a separate script. Prefer separation unless reuse is clearly cheaper.
- **`build.py` integration:** none for the pilot (on-demand only). Whether to add a subcommand later is a separate, fog item.

## Expected output

A short design spec the implementation ticket (04) can follow directly: chosen language, CLI signature, output path and columns, config file location and fields, and the reuse/extension decision.
