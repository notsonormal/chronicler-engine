---
diataxis: explanation
title: Two State Channels
---

## The two signals

The status of a Game comes from two places. One answers "is a Job running for this Game?". The other answers "did the last Job or the last refused write fail, and why?" The two questions have different lifetimes, so each has its own store. Neither store holds the other's answer, so the two cannot disagree.

## Channel A — the job entry

The job entry lives in memory, in the registry that `GameWriter` owns. A Game is busy when its job entry or its run hold exists.

Busy is true only while a process runs a Job or holds an Autoplay run. A restart ends every Job, so a saved busy mark would be wrong after it. Memory holds the mark, so a restart leaves every Game idle with no start-up repair. The engine's deployment contract is one process against one database, which is what makes a memory-only mark sufficient.

## Channel B — the failure line

The failure line lives on the Game record. It is none, or a failure kind with its raw text. A save unit writes it, so it survives a restart and a page reload. `GameWriter::refuse` writes it when a request answers `500`: an accept that cannot run, and an edit, a delete or a swipe switch whose save fails.

A failure has no process to end it. It stays until the player acts: the accept of the next request, a stop, a swipe switch and a delete each clear it. The accept clears it in storage, because a restart during the new Job would otherwise show the old failure again.

## Where the channels meet

Busy comes first, so the status names a step whenever a step runs, and never the line a running Job is about to replace.

A Job ends by removing its job entry. Before that, a failing step writes the failure line. The Game is busy until the entry goes, and it shows the failure line from then on. A stop removes the entry and clears the failure line in one hold of the registry. The status therefore moves from the step to idle, with no moment in between that shows a stale value.

## Document References

- [`../reference/game_flow.md`](../reference/game_flow.md) — the Job, the job entry and the status rule.
