# Record contention and effect size in benchmark reports

Type: task
Status: open
Blocked by: 02

## Question

`scripts/diagnostic_benchmark.py` reports omit the contention condition and the effect
size, so a single run on a busy host reads as signal. In the window a 663 s gate was
described as "the only thing running on the box" when it was not, and a retro finding was
retracted late — "My 'slot acquisition' framing was hand-waving… Candidate 4 is not a
finding. Drop it" — after the operator had to ask for repeated runs.

## Work

Record in both the JSON and the markdown report:

- the contention condition at run time: load average, and whether the build slot was held;
- the run-to-run spread when a step is repeated;
- an effect-size field against the baseline.

Flag a delta below a stated threshold as within noise. Do not build the
minimum-effect-size gate that blocks filing a finding — that is judgement, and it was
dropped with the rest of F28 in
[Decide which mechanisms to build](02-decide-mechanisms-to-build.md).

## Done when

- A report shows the contention condition and the effect size.
- A sub-threshold delta is labelled as noise, not as a speedup.
