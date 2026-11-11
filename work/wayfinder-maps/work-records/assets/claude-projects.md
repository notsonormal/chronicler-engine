# How Claude Projects, ChatGPT Projects and Claude Code memory organize work and memory

Access date of every source: 2026-11-05. These products change often. Check a claim again before a decision rests on it.

Labels: **known** means a source says it. **inferred** means I drew it from known facts. **guessed** means no source says it. **I don't know** means the sources are silent.

## Sources

- **S1.** [What are projects?](https://support.claude.com/en/articles/9517075-what-are-projects) (Claude help).
- **S2.** [How can I create and manage projects?](https://support.claude.com/en/articles/9519177-how-can-i-create-and-manage-projects) (Claude help).
- **S3.** [Use Claude's chat search and memory](https://support.claude.com/en/articles/11817273-use-claude-s-chat-search-and-memory-to-build-on-previous-context) (Claude help).
- **S4.** [Let Claude coordinate ongoing work with Projects](https://code.claude.com/docs/en/claude-projects) (Claude Code docs; public beta).
- **S5.** [How Claude remembers your project](https://code.claude.com/docs/en/memory) (Claude Code docs: `CLAUDE.md`, `AGENTS.md`, auto memory).
- **S6.** [Projects in ChatGPT](https://help.openai.com/en/articles/10169521-projects-in-chatgpt) (OpenAI help; it includes "Memory in projects" and the Projects FAQ).
- **S7.** [Memory in ChatGPT](https://help.openai.com/en/articles/8590148-memory-in-chatgpt) (OpenAI help).
- **S8.** The pi docs on this machine, `docs/configuration.md`, section "Context files". Pi is the harness of this repo.

My fetch of S6 and S7 returned HTTP 403. I read their text through search results.

## The word "project" means three things

- **known** (S1, S2) Claude.ai chat: a project is a workspace with chats, a knowledge base, instructions and its own memory.
- **known** (S4) Claude Code on the web: a project is one coordinating conversation plus the threads it starts. It has a goal, instructions, memory files, repositories and a Library of files. Anthropic is moving some Claude.ai projects to this version (S1).
- **known** (S5) The Claude Code CLI uses `~/.claude/projects/<project>/` as the key of one repository. It is a folder per repository. It is not a goal.

The S4 project is the closest match to an expedition. The S1 and S6 projects are closer to a folder of chats. Below, I name the tool each time.

## Point 1. The parts of a project

| Part | Claude.ai project (S1–S3) | Claude Code project (S4) | ChatGPT project (S6, S7) |
|---|---|---|---|
| **Instructions** | Written by the user. Used in every chat of the project. | Written by the user, up to 16,000 characters. Sent to each new thread and to the coordinator. | Written by the user. Apply only in the project. They override the global custom instructions. |
| **Knowledge or files** | Uploaded by the user. On a paid plan, retrieval (RAG) takes over near the context limit (S1). | Repositories, files and folders that the user adds, and the files that threads produce. All are in the **Library**. | Uploaded by the user. I don't know how ChatGPT picks what it reads. |
| **Memory** | Written by Claude, as separate topics. The user can read, edit and delete each topic (S3). | Written by Claude, as files. Every cloud thread reads the index `MEMORY.md` at the start and opens the other files when needed. The user edits the files in the project settings. | Written by ChatGPT. The project can use its chats and files as context. The user cannot see a list of project memories (S6). |
| **Chats** | Chats inside the project. | One long **conversation** that coordinates, and **threads** that do the work. A thread has its own context and reports back. | Chats inside the project. A chat can use the other chats of the project. |

- **known** (S4) The coordinator "works from recent messages, recent threads, and project memory rather than its full history". S4 says: "Put anything that must never be dropped in project memory."
- **known** (S4) The stores split by who writes them and by what they hold. Instructions are the brief. Project memory holds "requirements, decisions, and pitfalls". The `CLAUDE.md` of each repository holds the rules of that repository. S4: "put instructions about a repository in its `CLAUDE.md` and notes about the project in project memory."
- **inferred** The expedition wiki matches project memory plus project instructions. The maps and tickets match threads. A chief matches the coordinator conversation. One mismatch: a thread is a session, and a map is a plan with many tickets.

## Point 2. Memory across projects

- **known** (S3) Claude.ai: each project has its own memory. In a project, Claude uses the project memory and the account memory from chats outside projects. It does not use the memory of other projects. The user can switch off "Use account memory" on the project page to keep a project fully separate.
- **known** (S6) ChatGPT: each project has **default memory** or **project-only memory**. Project-only: saved memories are not used in the project, chats see only chats of the same project, and chats outside do not see the project. A shared project is always project-only. There is no global switch.
- **known** (S5) Claude Code: one auto memory folder for each git repository, shared by its worktrees, on one machine. It is not shared across machines.
- **known** (S5) Claude Code instructions come in layers: managed, user (`~/.claude/CLAUDE.md`), project, and local. All found files are **concatenated**. They do not override each other. The order is broad to specific, so the file closest to the working directory is read last.
- **inferred** Two designs exist for rules that span projects. Claude **adds** a broader layer (the user file in Claude Code, account memory in Claude.ai). ChatGPT **replaces** it inside a project. Need 3 favours the first design.

## Point 3. Work outside a project

- **known** (S2) Claude.ai: a chat outside a project is a standalone chat. The user moves it with "Add to project", and can remove it or move it again.
- **known** (S6) ChatGPT: "Move to project" or a drag moves a chat in. The chat then "inherits the project's instructions and file context". "Remove from project" moves it back. Chats made with a GPT and temporary chats cannot move.
- **known** (S4) Claude Code: a cloud session moves in with "Continue in project", as a new project or into an existing one. The original session stays in the list. Claude reads the session and proposes a setup.
- **inferred** No tool gives the work outside projects a named place. The words are "standalone chat" (S2) and "general ChatGPT conversations" (S6). It is a flat list. A menu action moves an item in later.

## Point 4. Closing

- **known** (S2) Claude.ai: **archive** hides a project, and the user can **unarchive** it. **Delete** is a separate action. S2 says archive suits "completed projects or those you're not actively working on but may need to reference in the future".
- **known** (S4) Claude Code projects have **Pause** (stops all threads and scheduled tasks, reversible), **Archive** (hides the project, archives its threads, reversible) and **Delete** (removes the project, its threads, its memory and its files, with no undo). Pushed branches and pull requests stay on GitHub. After Unarchive, the threads stay archived until the user unarchives them one by one.
- **known** (S4) A thread has a **Resolved** state. The user sets it, or Claude sets it after the last step (for example a merge), or it sets itself after a week with no activity. The user can reopen it.
- **known** (S6) ChatGPT: **delete** is the only project-closing action I found. It permanently removes chats, instructions and files stored only in the project. Files saved in Library or Space stay. I found "archive" for chats only.
- **known** (S5) Claude Code repository memory has no close step. Transcripts are cleaned after a retention period. Memory files are excluded from the cleanup and stay until someone edits or deletes them.
- **I don't know** What happens to Claude.ai project memory when the project is archived.

## Point 5. Claude Code memory files and ChatGPT

**Claude Code (S5).** Two systems load at the start of every session.

| | `CLAUDE.md` (or `AGENTS.md`) | Auto memory |
|---|---|---|
| Who writes it | The user | Claude |
| Holds | Instructions and rules | Preferences, corrections, and context that the code does not show |
| Scope | Managed, user, project, local | One folder for each repository, outside the repository |
| Loaded at start | The whole file | `MEMORY.md` only: the first 200 lines or 25 KB |
| On demand | Files in subdirectories, when Claude reads a file there. Rules with `paths:` frontmatter, when a matching file is read. | The topic files |

- **known** (S5) Auto memory has four note types: `user`, `feedback`, `project` and `reference`. Claude skips what it can derive from the code, such as architecture or file paths.
- **known** (S5) `/doctor prompt-audit` looks for "references to files or commands that don't exist" and for files that contradict each other. An `@path` import keeps one shared file but does not cut the context cost. Claude Code reads `AGENTS.md` only when no `CLAUDE.md` exists above the working directory.
- **known** (S5) `MEMORY.md` is an index, "one line per memory". Near the limit, Claude Code tells Claude to "keep one line per entry, move detail into topic files, and merge or drop stale entries". Text past the limit is dropped at the next load.
- **known** (S5) Claude Code writes a `modified` timestamp into each memory file. S5: "The timestamp shows how current the fact is."
- **known** (S8) Pi loads context files from the agent directory, the working directory and its parent directories. A file applies "whenever Pi runs in its directory or anywhere below it". The pi docs do not say that a subdirectory file loads when the agent reads a file there. Claude Code does this (S5).
- **I don't know** Whether pi loads a subdirectory `AGENTS.md` on demand. The docs I read do not say.

**ChatGPT (S6, S7).** A project has instructions, files, chats and a memory mode. Project memory has no list view. Claude exposes its topics and files for edit. S7 says deleting a chat does not necessarily delete a saved memory made from it. The user must remove the fact from each place.

## Point 6. The needs

Rating: **yes**, **part**, **no**, or **?** (the sources are silent). Each rating is **inferred** from the known facts above.

| Need | Claude.ai project (S1–S3) | Claude Code project (S4) | Claude Code repo memory (S5) | ChatGPT project (S6, S7) |
|---|---|---|---|---|
| 1. See each goal and where it stands | **part**: a projects page with tabs and stars. No state. | **yes**: a Goal field, and an Overview pane that groups threads (Waiting on you, Working, Ready for review, Idle, Resolved) | **no**: no goal concept | **part**: a project list. No state. |
| 2. One obvious place for a one-off item | **yes**: a standalone chat, moved in later | **yes**: a session outside a project, moved in with "Continue in project" | **part**: a session in the repo. No item store. | **yes**: a chat outside a project, moved in later |
| 3. Rules that span projects, in one place | **yes**: account memory is used beside project memory | **?**: S4 does not name a cross-project layer | **yes**: the user file `~/.claude/CLAUDE.md`, added to the project layer | **part**: global custom instructions are replaced inside a project, not added |
| 4. A chief owns one folder and finds everything from there | **yes**: one project holds knowledge, instructions, memory and chats | **yes**: every thread starts with the instructions, memory, repositories and Library | **part**: a directory `CLAUDE.md` loads when Claude works there. Auto memory is outside the repo. | **yes**: one project holds chats, files and instructions |
| 5. Close a goal as one unit | **yes**: archive (reversible) or delete | **yes**: pause, archive (reversible) or delete | **no**: no close step | **part**: delete only, and it is permanent |
| 6. A fresh agent finds any record from one doc | **part**: topics are listed and editable. Chats are found by search. | **yes**: the `MEMORY.md` index is read first, and topic files open on demand | **yes**: one instruction file with imports, and the `MEMORY.md` index with a size limit | **no**: no list of project memories |

## Facts for tickets 04 to 08

### Ticket 04. The names

- **known** (S1–S7) All three tools call the unit of one long goal a **project**. They call the work inside a **chat** (Claude.ai, ChatGPT) or a **thread** (Claude Code). Model-written notes are **memory**. User-written rules are **instructions**. Files are **knowledge**, **sources** or **Library**.
- **known** (S4) The Claude Code project has a **Goal** field. S4 says a project is worth creating "when the work has a goal that outlasts one session". "Goal" is also the word of the chief-of-staff skill.
- **known** (S2, S6) The no-project space is "standalone chat" (S2) and "general ChatGPT conversations" (S6). No tool names it as a folder.
- **known** I found none of "expedition", "errand" or "base camp" in these sources. No tool supports or rejects them.
- **inferred** "Project" has the best outside support. It costs here: the charting facts count 540 uses of it in `.scratch/` prose, and the Claude Code CLI uses it for a repository.
- **inferred** **Resolved** is also a known word. S4 uses it for a thread, and wayfinder uses it for a ticket.

### Ticket 05. The wiki contents

- **known** (S5) The model-written store holds what the code cannot show: preferences, corrections, ongoing work, decisions, and pointers to places outside the repo. It skips architecture and file paths. This supports the charting proposal of no subsystem pages.
- **known** (S4, S5) The user-written store holds standing rules. The model-written store holds decisions and pitfalls. S5 warns that two conflicting instruction files make Claude "pick one arbitrarily". So a rule should live in one file.
- **known** (S5) The store is the current state, not a history. Claude merges or drops stale entries. Each file has a `modified` timestamp. I found no "superseded" marker.
- **inferred** This is closer to the current-state voice of this repo than to the dated-claim voice of the chief-of-staff skill. One `modified` date for each file is a middle way. It is a point for ticket 05, not a decision.
- **known** (S5) The index is short and always loaded. The detail is in files that open on demand. The index has a hard limit of 200 lines or 25 KB.
- **known** (S6) ChatGPT project memory is not listed and not editable. So a user cannot keep it true. Claude exposes its files for edit and delete.

### Ticket 06. The layout

- **known** (S4) The unit of one goal holds its instructions, memory, files and threads together, and threads inherit all of them. The code repository is a link. The repository keeps its own `CLAUDE.md`.
- **known** (S5) Claude Code loads files above the working directory at launch. It loads files in subdirectories on demand. Pi loads only from the working directory and above (S8).
- **inferred** A folder-level `AGENTS.md` in an expedition folder loads under pi for a chief that starts in that folder. It does not load for a chief that starts at the repo root and then reads a file there. I did not test this.
- **known** (S4) "Closed" has three forms: paused, archived (reversible) and deleted. **Resolved** is for a thread, not for a project.
- **known** (S2, S6) A one-off item stays in a flat list of chats until a menu action moves it into a project. The move keeps the content and adds the context of the project.

### Ticket 07. Which maps go where

- **known** (S4) A project fits "one goal across many repositories", "an area you keep feeding" (for example the bugs and stack traces of one service), and "a build or migration bigger than a session". It does not fit "one task that fits in a session".
- **inferred** A bucket map such as `autoplay-follow-ups` or `retro-issue-bucket` matches "an area you keep feeding". So a bucket may be an expedition. It does not have to be an errand. I did not read those maps.
- **inferred** A one-off item with a clear end matches "one task that fits in a session". That is an errand.
- **I don't know** Which of the live maps pass the S4 test.

### Ticket 08. The names in docs and skills

- **known** (S5) Claude Code keeps one shared instruction file and points other files to it with an `@path` import. A later rename then changes one file.
- **known** (S5) S5 puts "multi-step" procedures in skills and facts that hold in every session in the instruction file. A skill that names a folder path mixes a procedure with a fact.
- **known** (S5) The tool has an audit, `/doctor prompt-audit`. It finds "references to files or commands that don't exist". So the tool treats a stale path in an instruction file as a fault to check. Today `scripts/validate_docs.py` checks no `.scratch/` path (charting facts).
- **inferred** The pattern "one doc names the path, and other files point to it" has support in S5. It matches the working view of ticket 08.
- **I don't know** How a skill sync from upstream treats a local path change. The sources do not cover it.
