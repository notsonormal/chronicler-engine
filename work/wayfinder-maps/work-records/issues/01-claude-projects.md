# Find how Claude Projects and similar tools organize work and memory

Type: research
Status: resolved
Blocked by: —

## Question

How do Claude Projects, ChatGPT Projects and the per-project memory of Claude Code organize work and memory for each project? What happens to work that belongs to no project? Which of their parts meet [the needs](../map.md#notes), and how would each part become a folder or a file in this repo?

## Context

- The owner named Claude Projects as one model for this map (2026-11-04). A project there has instructions, knowledge files, memory and chats.
- The working views of [the map](../map.md) compare a project's instructions and knowledge with an expedition's wiki, and its chats with the maps and errands. Test that comparison. Do not assume it.

Points to cover:

1. **The parts of a project.** Instructions, knowledge files, memory, chats. For each part: what is always in the context, what is found on demand, and who writes it (the user or the model).
2. **Memory across projects.** Whether memory is kept for each project, shared, or both. What a chat outside a project can see.
3. **Work outside a project.** Where it goes, and how a user moves it into a project later.
4. **Closing.** How a project is archived or deleted, and what happens to its memory.
5. **Claude Code and ChatGPT.** The same points for ChatGPT Projects, and for the memory files that Claude Code keeps for each repo. Add one more tool only if it adds a new idea.
6. **The needs.** A table: each tool against each of the six needs in the map's Notes.

Use the vendor docs and help pages as sources. Date each claim, because these products change often. Write the result to `assets/claude-projects.md`, and link it from the answer.

## Answer

Asset: [How Claude Projects, ChatGPT Projects and Claude Code memory organize work and memory](../assets/claude-projects.md). It holds one section for each of the 6 points, the needs table (4 tools against 6 needs), and the facts for tickets 04 to 08. Access date of every source: 2026-11-05.

- **known** The unit of one long goal has one name in all three tools: **project**. The word means three things. In Claude.ai it is a folder of chats with instructions, knowledge and memory. In Claude Code on the web (beta) it is one coordinating conversation with threads, a Goal, instructions, memory files and a Library. In the Claude Code CLI it is one repository (`~/.claude/projects/<project>/`).
- **known** Only the Claude Code project (beta) holds a goal, instructions, memory, files and work in one unit, with a state view of its threads (Waiting on you, Working, Ready for review, Idle, Resolved). It is the closest match to an expedition. The Claude.ai and ChatGPT projects are closer to a folder of chats.
- **known** The tools split stores by who writes them. The user writes instructions (standing rules). The model writes memory (decisions, pitfalls, preferences). The Claude Code docs say "put instructions about a repository in its `CLAUDE.md` and notes about the project in project memory".
- **known** Claude Code memory is a current-state store, not a history. `MEMORY.md` is a short index, "one line per memory", with a limit of 200 lines or 25 KB. The detail is in topic files that open on demand. Claude is told to "merge or drop stale entries". Each file has one `modified` timestamp. The sources show no "superseded" marker.
- **known** Rules that span projects are added as a broader layer in Claude Code (the user `CLAUDE.md` is concatenated with the project file) and in Claude.ai (account memory is used beside project memory). ChatGPT replaces them inside a project. ChatGPT project memory has no list view.
- **known** Work outside a project has no named place. It is a "standalone chat" or a "general ChatGPT conversation" in a flat list. A menu action moves it into a project later, and the chat keeps its content ("Add to project", "Move to project", "Continue in project").
- **known** Closing is a reversible archive in Claude.ai and Claude Code (Claude Code also has Pause and a permanent Delete that removes memory and files). I found only a permanent delete for a ChatGPT project. Claude Code repository memory has no close step.
- **known** The Claude Code docs say a project fits "one goal across many repositories", "an area you keep feeding" and "a build or migration bigger than a session". They say it does not fit "one task that fits in a session".
- **inferred** A bucket map (`autoplay-follow-ups`, `retro-issue-bucket`) matches "an area you keep feeding", so it may be an expedition. A one-off item with a clear end matches an errand. I did not read the maps.
- **inferred** The expedition wiki matches project memory plus project instructions. The maps and tickets match threads. A chief matches the coordinator conversation. The match is loose in one place: a thread is a session, and a map is a plan with many tickets.
- **inferred** Pi loads context files from the working directory and its parents. Claude Code also loads a subdirectory file when Claude reads a file there. So a folder-level `AGENTS.md` in an expedition folder may not reach a chief that starts at the repo root.
- **I don't know** Whether pi loads a subdirectory `AGENTS.md` on demand, what happens to Claude.ai project memory on archive, and how a skill sync from upstream treats a local path change. I found none of "expedition", "errand" or "base camp" in these sources.
