# Determine applicability scoring rubric and final output format

Type: grilling
Status: resolved
Assignee: pi
Blocked by:

## Question
What scoring or categorization scheme should the Uncle Bob 2026 repository applicability study use, and what should the final artifact be (report, documentation updates, code changes, or a mix)?

## Answer

The following rubric and output plan were approved by the map owner.

### Applicability rubric

| Category | Meaning |
|---|---|
| **Directly usable** | Could be integrated or run against Chronicler Engine with reasonable effort. |
| **Practice/technique/tooling idea to adopt** | Concrete practice, tool, or implementation idea to translate into Rust/Chronicler. |
| **Already covered** | Chronicler Engine already has equivalent guardrails, tooling, or patterns. |
| **Not applicable** | Does not match Chronicler Engine's language, architecture, or workflow. |
| **Future spike** | Interesting but needs more investigation before a firm applicability call. |

### Final artifact

A single markdown report in `docs/project/` (tentative filename to be decided when the report ticket is created). It will summarize the 34 pushed repositories, place each under the rubric, and highlight actionable takeaways.

### Scope and depth

- All 34 pushed repositories get a brief one- or two-line categorization.
- Extra depth goes to items landing in **Practice/technique/tooling idea to adopt** or **Future spike**.

### Remaining work before writing the report

Do a quick-pass assessment of the remaining unassessed repository groups — games/examples (`missile-command`, `spacewar`, `Pharaoh`, `Pharaoh-js`, `empire-2025`) and acceptance testing (`fitnesse`) — to confirm they are "Not applicable" / "Already covered" or to flag any that deserve a deeper ticket.