# Research 15 web sources: clean-room method and prior art

Supporting asset for [Research: method and inputs for a clean-room design of the generation model](../issues/15-research-clean-room-design.md). Written by a `researcher` agent; reviewed by the resolving session. The main asset is [research-15-clean-room-method.md](research-15-clean-room-method.md).

**Re-verification (2026-10-12).** The resolving session fetched and matched these quotes: the Wikipedia roles sentence and its NEC summary; Pilgrim's text in chardet issue #327; Blanchard's concession and process (through Willison's quote of the comment); Willison on `metadata/charsets.py` and training data; Ronacher on the different design and the test suite; the Jas quote on specification gaps. Not re-verified (fetch blocked): the court quote through 10 Computer L.J. 453, the SPECA quotes, the Computerworld and Phoenix quotes.


Two questions. (1) How legal clean-room design works, worked through the Phoenix BIOS and *NEC v. Intel*. (2) Prior art for a clean-room / spec-only redesign done **not** for IP reasons — including LLM agents as the clean team — and what went wrong.

Labels: **known** = read in the cited source, with a short quote. **inferred** = my reasoning over known facts. **guessed** = unverified.

---

## Method

**Roles (known).** Wikipedia's summary matches every account I found:

> "Typically, a clean-room design is done by having someone examine the system to be reimplemented and having this person write a specification. This specification is then reviewed by a lawyer to ensure that no copyrighted material is included. The specification is then implemented by a team with no connection to the original examiners."
> — https://en.wikipedia.org/wiki/Clean-room_design

So four roles: **examiner team** ("dirty"/"contaminated", writes the spec), the **specification** (says *what*, not *how*), the **reviewer/lawyer** (screens the spec across the wall), the **clean team** (implements from the spec alone).

**The wall (known).** A widely mirrored account frames it as a procedural separation of knowledge: "One group of engineers, the 'dirty' side, studies the copyrighted material and writes down only what the firmware must do, an interface specification stripped of IBM's actual code. A second group, the 'clean' side, has never seen IBM's listing and is sometimes deliberately walled off from anyone who has." (https://www.centerconsulting.com/code-library/stories/phoenix-bios-clean-room)

**What counts as contamination (known).** Contamination is *access*, not similarity. *NEC v. Intel*'s clean-room engineer Gary Davidian "never had access to any microcodes created by either Intel or NEC, nor did he have access to any microcode implementing the 8086 instruction set" (NEC Post-Trial Brief, quoted at https://jolt.law.harvard.edu/assets/articlePDFs/v03/03HarvJLTech209.pdf).

The mirror rule holds: a clean room fails if the separation is not real.
- *SAS Inst. v. S & H Computer Sys.*, 605 F. Supp. 816 (M.D. Tenn. 1985) — "the court found defendant's clean room to be inadequately insulated from access to plaintiffs program."
- *Pearl Sys. v. Competition Electronics*, 8 U.S.P.Q.2d 1520 (S.D. Fla. 1988) — "clean room evidence was used to show that similarities … were **not** dictated by functional constraints, but were due to copying."

(both per the same Harvard JOLT source)

**How the wall is evidenced (known).** With contemporaneous records, because the thing being proved is the *absence* of access:

> "In *NEC v. Intel*, the clean room documentation was 'many thousand[s] of pages' long. It included 'every single piece of paper which Mr. Davidian saw,' plus records of all his written and electronic communications."
> — Harvard JOLT, quoting NEC's Post-Trial Brief

Documentation cuts both ways: "there were very few development notes maintained for the specific NEC microprograms in issue as compared to the numerous notes showing development of other microprograms by the same NEC programmer" (https://digitalcommons.law.scu.edu/cgi/viewcontent.cgi?article=1030&context=chtlj).

**Two kinds of clean room (known).** "Such 'made for litigation' clean rooms are primarily evidentiary tools. The second type of clean room is a standard operating procedure ('SOP') clean room." (Harvard JOLT)

**Personnel (known).** Harvard JOLT, citing Derwin (1989): "Clean rooms require at least three groups of people: a specification team, a design team, and a coordination team." The same article notes the difficulty of finding programmers never exposed to the subject program.

**Not found.** No primary Phoenix or NEC document describing a *lawyer reviewing the specification*. Wikipedia asserts it; the press and law-journal accounts describe engineers, specs and audit trails. Searches: `Phoenix BIOS clean room lawyer review specification`, `NEC v. Intel clean room procedures evidence guidelines`. Treat the reviewer role as **known from Wikipedia only** and **unconfirmed** for Phoenix/NEC.

---

## Phoenix BIOS

**known** — near-contemporary trade press:

> "First, a team of engineers studied the IBM BIOS—about 8KB of code—and described everything it did as completely as possible without using or referencing any actual code. Then Phoenix brought in a second team of programmers who had no prior knowledge of the IBM BIOS and had never seen its code. Working only from the first team's functional specifications, the second team wrote a new BIOS that operated as specified."
> — Schwartz, Computerworld, 2001-11-12, https://www.computerworld.com/article/1349695/reverse-engineering.html

**known** — team structure and spec format (Wikipedia, citing *PC Magazine* 10 July 1984 and Schmidt 1994):

> "Engineers read the BIOS source listings in the *IBM PC Technical Reference Manual*. They wrote technical specifications for the BIOS APIs for a single, separate engineer—one with experience programming the Texas Instruments TMS9900, not the Intel 8088 or 8086—who had not been exposed to IBM BIOS source code. The single engineer developed code to mimic the BIOS APIs. By recording the audit trail of the two groups' interactions, Phoenix developed a defensibly non-infringing IBM PC compatible ROM BIOS."
> — https://en.wikipedia.org/wiki/Phoenix_Technologies

Two details matter for your experiment: **the spec was behavioural, not a design** ("technical specifications for the BIOS APIs"; "described everything it did … without using or referencing any actual code"), and **the clean team was one person, with the isolation evidenced by an audit trail**.

**known** — the clean engineer's deliberate ignorance was a marketed feature: "Phoenix expressly emphasized the clean-room process through which their BIOS code had been written by a programmer who did not even have prior exposure to Intel microprocessors, himself having been a TMS9900 programmer beforehand" (*PC Magazine*, 1984-07-10, "Phoenix Says Its BIOS May Foil IBM's Lawsuits", p. 56, via Wikipedia). Phoenix licensed the BIOS at $290,000 and bought a $2M copyright-infringement insurance policy from The Hartford (Wikipedia, citing Langdell 1984).

**inferred — the strongest evidence for your *epistemic* question:** Phoenix's confidence came from the *existence of the wall*, not from any measured difference between the spec and the original. They insured the process.

**known** — Compaq, briefly. Compaq's BIOS came first (1982) but was for its own products; Phoenix's was the first *sold* (https://historyofinformation.com/detail.php?entryid=3846). A secondary account: "Compaq's answer was clean-room design, sometimes called Chinese-wall engineering. One team studied the IBM BIOS and wrote a precise specification of everything it did, every input and output, without describing how. A second team of programmers, who had never seen IBM's code…" (https://www.geschichte-der-informatik.de/articles/compaq_and_the_bios_clean_room/).

---

## NEC v. Intel

**known — identity and timing.** *NEC Corp. v. Intel Corp.*, No. C-84-20799, N.D. Cal., decided 6 Feb 1989, Judge William P. Gray (1989 WL 67434; 10 U.S.P.Q.2d 1177); filed 21 Dec 1984. Judge Ingram's earlier partial findings (645 F. Supp. 590, 22 Sept 1986) were **vacated** (835 F.2d 1546 (9th Cir. 1988)) after he disclosed a small Intel holding; Gray retried. Contemporary coverage: NYT and LA Times, both 1989-02-08.

**(a) The clean-room process (known).** Harvard JOLT:

> "NEC hired an independent engineer (Gary Davidian) to develop a set of microcode for the V20/30 without access to any other microcode. Because Davidian's version of the microcode was similar in many regards to **both** the Intel and NEC microcodes, the court found it likely that those similarities were dictated not by copying of Intel's microcode, but rather by functional constraints of the hardware, the architecture, and the need for 8086/88 compatibility."

The court's own words, as reproduced in 10 Computer L.J. 453 (1990):

> "[T]he Clean Room microcode constitutes compelling evidence that the similarities between the NEC microcode and the Intel microcode resulted from constraints. The Clean Room microcode was governed by the same constraints of hardware, architecture and specifications as applied to the NEC microcode, and copying clearly was not involved."

**known — the clean room was a litigation fallback, and the *first* revision was found to be infringing.** Judge Gray held that "even if the first version of the V20/30 microcode ('Rev. 0') resembled Intel's microcode very closely, the only version of NEC's code which should have been compared to Intel's was the final 'Rev. 2' version" (Harvard JOLT). And: "NEC did not even begin developing the code until April 1986, just before the start of the first trial. The clean room microcode was initially developed for 'business reasons'" (10 Computer L.J. 453).

**known — small, fast, and only possible because the spec was complete.** "So Davidian started work on the development of the new microcode in August 1986. He was given a deadline of six months … he made rapid progress and finished his work, with code that implemented the full V20/30 instruction set and passed all the tests, in just 15 days." (https://thechipletter.substack.com/p/intel-vs-nec-the-case-of-the-v20s — well-sourced newsletter, not primary.) **inferred**: a 15-day clean-room rewrite of a mature artifact was possible only because the specification (V20/30 instruction semantics + a software emulator + a test suite) was already complete and unambiguous. That precondition rarely holds.

**(b) Convergence (known).** Judge Gray's test: a similarity that a specification-constrained independent implementation also produces is unexplained by copying, and is therefore attributed to functional constraint. Harvard JOLT records the caveats: "Judge Gray never stated precisely which aspects of the microcode were idea and which were expression"; similarities in "simple, straightforward operations in which close similarity in approach is not surprising" were discounted; and the court insisted on not "los[ing] sight of the forest for the trees." The decision was **not appealed**, and Intel separately lost on forfeiture (~3 million of 28 million chips unmarked). So the constraint holding is broad but not appellate-tested.

**known — first acceptance of clean-room evidence in a US trial** (Harvard JOLT). The doctrine is about *access*: "The use of clean room procedures as evidence … denies the infringement plaintiff the possibility of satisfying the necessary element of access."

---

## Does the convergence test transfer

**inferred**, from the facts above. Short answer: partly, and only after the independence premise is repaired.

The transferable core is an asymmetry:
- A clean design **agrees** with the existing implementation on some structure, and the clean designer could not have seen it → copying is one less explanation, constraint is one more. Legal term *constraint*; your term **forced by the requirements**.
- A clean design **disagrees** → the existing shape is not forced. It is a free choice, a defect, or a gap in the spec.
- The stronger form, and the one *NEC* used: when the new design resembles **both** the original and the incumbent implementation, you learn about a constraint surface neither alone reveals.

Limits, most severe first:

1. ***NEC*'s independence premise does not hold for your setup.** The court's clean team was walled off from *NEC's* artifacts too, and the spec came from the contaminated side of the same organisation — but still a different party from the implementer. In your case both sides share one owner reading one requirements doc, likely one assistant model. **inferred**: shared authorship of the spec is a large shared-influence channel, and it is exactly the channel *NEC* closed.
2. **Training-data contamination is the LLM analogue of "the examiner had access."** Not hypothetical (see chardet below), and the one channel a wall cannot close — whereas all of *NEC*'s leaks were closable.
3. **Convergence is not proof of constraint, nor divergence proof of freedom.** Knight & Leveson (below) shows independent designers converge on things that are *not* forced. SPECA notes the reverse: differential testing "provides little signal when all implementations converge on the same incorrect interpretation of an ambiguous requirement" (https://arxiv.org/pdf/2602.07513).
4. **The clean team can re-derive the old design for non-copying reasons** — language conventions, the domain, the test suite. Not contamination, but it destroys discriminating power.
5. **Granularity decides the answer.** *NEC* judged the whole and discounted similarity in trivial routines.

**inferred — practical repair.** To make this an experiment and not a ritual you need (a) a written requirements document independent of the existing code; (b) an evidenced wall (the clean agent's plan/logs, and a hard rule that it never sees `src/`); and (c) the **list of disagreements** as the primary output. Agreements are the sanity check; disagreements are the result.

---

## Prior art: non-legal clean-room / spec-only redesign

**IETF "two independent and interoperable implementations" (known) — the clearest formal statement of the idea in engineering.** RFC 2026 §4.1.2:

> "A specification from which at least two independent and interoperable implementations from different code bases have been developed, and for which sufficient successful operational experience has been obtained, may be elevated to the 'Draft Standard' level."
> — https://www.rfc-editor.org/rfc/rfc2026.txt

Two consequences. The requirement bound *features*, not just existence: "The requirement for at least two independent and interoperable implementations applies to all of the options and features of the specification. In cases in which one or more options or features have not been demonstrated in at least two interoperable implementations, the specification may advance to the Draft Standard level only if those options or features are removed." And it survived the 2011 simplification: RFC 6410 §2.2 keeps "at least two independent interoperating implementations with widespread deployment" and adds "There are no unused features in the specification that greatly increase implementation complexity" (https://www.rfc-editor.org/rfc/rfc6410.txt). A second implementation is treated as a **specification-quality instrument**, not only an interoperability one.

**W3C / WHATWG (known).** "In order to advance beyond Candidate Recommendation, each normative specification is expected to have at least two independent interoperable implementations" (https://w3c.github.io/charter-drafts/2026/webtransport-wg-charter.html). WHATWG Stages: "Advancing beyond Stage 1 requires support for the decision by at least two implementers (via their browser engine representatives)" (https://whatwg.org/stages).

**Second implementation as spec audit, LLM era (near-known — I could not fetch the page; quote is from the search index).**
> "A second Core-profile implementation was written from the specification text rather than from the Rust compiler. The goal was not to replace the reference implementation. It was to force two independent readers to answer the same normative questions. They disagreed four times. The most serious disagreement changed the byte identity of a contract."
> — https://rokoss21.tech/en/posts/facet-second-implementation/

**Differential testing as routine spec audit (known).** "We present SPECA, a SPEcification-to-Checklist Auditing framework that turns normative requirements into property-based checklists, maps them to implementation locations…" (https://arxiv.org/pdf/2602.07513). "Validating a finding … means checking whether two independent implementations resolve the underspecification differently on production code" (https://arxiv.org/html/2609.38752v1). And the honest limit: "Prose specifications help, but natural language is ambiguous. Differential testing can detect how implementations diverge, but it cannot say which implementation is correct." (https://arxiv.org/html/2608.18039)

**Formal-method relatives (known).** Raft ships both a TLA+ specification and a detailed implementation guide, and hundreds of independent implementations exist; the TLA+ literature frames the spec as removing "ambiguities in the protocol design" (https://arxiv.org/html/2302.02703v2). Spec-first rather than spec-only-from-code, but the same goal.

**"Zero-based design" / "greenfield thought experiment" (partly known).** These phrases exist in process-consulting and UX writing meaning "justify every element from first principles rather than incrementally" (e.g. https://flevy.com/blog/zero-based-redesign-zbr-cost-transformation/). **Not found:** any established software-engineering practice *named* for hiding the existing design from a reviewer so as to expose anchoring. Searches: `design review hide current design anchoring`, `zero-based design software architecture`, `requirements-only design exercise compare existing code`. **inferred**: the anchor-removal use you propose is a re-purposing of the clean-room method, not a recognised named practice — so you do not inherit a validated protocol, including its validation of *when the result means anything*.

---

## Prior art: LLM agents as the clean team

**The canonical test case: chardet 7.0.0 (March 2026).** All **known**.

- Maintainer Dan Blanchard released chardet 7.0.0 as a "Ground-up, MIT-licensed rewrite of chardet. Same package name, same public API" (https://github.com/chardet/chardet/releases/tag/7.0.0).
- Original author Mark Pilgrim (absent since 2011) opened issue #327: "They have no such right; doing so is an explicit violation of the LGPL. Licensed code, when modified, must be released under the same LGPL license. Their claim that it is a 'complete rewrite' is irrelevant, since they had ample exposure to the originally licensed code (i.e. this is not a 'clean room' implementation). Adding a fancy code generator into the mix does not somehow grant them any additional rights."
- Blanchard conceded the wall: "You're right that I have had extensive exposure to the original codebase: I've been maintaining it for over a decade. A traditional clean-room approach involves a strict separation between people with knowledge of the original and people writing the new implementation, and that separation did not exist here." (https://github.com/chardet/chardet/issues/327#issuecomment-4005195078)
- He substituted *measurement* for *process*: JPlag similarity against the old code — max 1.29% vs the previous release, 0.64% vs 1.1 (the ShiftMag account cites "0.04% average similarity").
- His process: "I then started in an empty repository with no access to the old source tree, and explicitly instructed Claude not to base anything on LGPL/GPL-licensed code."

**Four failure modes that are directly relevant.**

1. **Leaks into the spec — documented, not hypothetical.** Simon Willison read the published rewrite plan: "There is one example where Claude Code referenced parts of the codebase while it worked, as shown in the plan — it looked at `metadata/charsets.py`, a file that lists charsets and their properties expressed as a dictionary of dataclasses." (https://simonw.substack.com/p/can-coding-agents-relicense-open)
2. **Training-data contamination — the wall cannot close it.** Willison: "Claude itself was very likely trained on chardet as part of its enormous quantity of training data - though we have no way of confirming this for sure. Can a model trained on a codebase produce a morally or legally defensible clean-room implementation?" ShiftMag states the consequence: "The fact that a plagiarism detector can't find matching tokens doesn't mean the work is independent. It means the laundering was effective." (https://shiftmag.dev/license-laundering-and-the-death-of-clean-room-8528/)
3. **A spec written by the contaminated side pre-loads the old design.** Blanchard used a "brainstorming skill to create a design document specifying the architecture and approach I wanted" — the "dirty" side produced a *design document*, not a pure behaviour spec. Armin Ronacher, sympathetic to the outcome, describes the clean agent's input as "only pointing it to the API and the test suite" (https://lucumr.pocoo.org/2026/3/5/theseus/). **inferred**: this is the *NEC* fallback pattern with much thinner walls and an examiner unaware of being one.
4. **The clean team re-derived a different design — which is the useful signal.** Ronacher: "he had an AI port one of my libraries to another language and it ended up choosing a different design for that implementation. In many ways, the functionality was the same, but the path it took to get there was different." For chardet: "It's significantly faster than the original implementation, supports multiple cores and uses a fundamentally different design." **This is the experiment working.**

**Known caution against over-reading similarity.** Bruce Perens (OSI co-founder), in the thread: "The courts have not sided with plaintiffs in finding AI work to be infringing so far, because the law as it stands today is built primarily around the concept of literal copying, cut and paste of the actual text." And: "I do not recommend rejecting an AI-mediated Open Source program with verified low-similarity to other works on the basis of legal risk at this time." (expanded at https://www.theregister.com/2026/03/06/ai_kills_software_licensing/). **inferred**: for your non-legal use this cuts the other way — token-similarity tools measure the wrong thing. *Decision* convergence is what you want, and no off-the-shelf tool measures it.

**Other LLM data points.**
- **Agent-built C compiler (Anthropic, Feb 2026, known).** 16 Claude Opus 4.6 instances, ~2,000 sessions, ~$20,000 in API fees, two weeks, ~100,000 lines of Rust; compiles the Linux 6.9 kernel on x86/ARM/RISC-V; 99% on the GCC torture suite; ran *Doom* (https://www.anthropic.com/engineering/building-c-compiler). Ars Technica's caveat is the one to carry into your ticket: "a C compiler is a near-ideal task for semi-autonomous AI model coding: The specification is decades old and well-defined, comprehensive test suites already exist, and there's a known good reference compiler to check against. Most real-world software projects have none of these advantages. The hard part of most development isn't writing code that passes tests; it's figuring out what the tests should be in the first place." (https://arstechnica.com/ai/2026/02/sixteen-claude-ai-agents-working-together-created-a-new-c-compiler/)
- **MirrorCode (METR + Epoch AI, Apr 2026, known).** A benchmark for "rebuilding complete programs from specifications"; preliminary results include "reimplementing a 16,000-line codebase" (https://epoch.ai/publications/mirrorcode-preliminary-results, https://arxiv.org/html/2606.30182v1). **inferred**: infrastructure for the *capability* of an agent clean team, not for its *independence*.
- **"Jas" case study (2026, known) — revives the N-version frame for LLM-authored ports.** Five working ports of a vector illustration application (Rust, Swift, OCaml, Python, browser), one developer. It reports exactly your intended finding: "The most informative divergences are not implementation bugs but specification gaps. The specification permits a reasonable interpretation, two ports adopt different reasonable interpretations, and the divergence forces the specification to be sharpened." (https://arxiv.org/html/2606.07828v1)

---

## Failure modes seen

**inferred** collapses; the facts are above.

1. **Spec leaks** — the clean side reads the original anyway. *chardet* (`metadata/charsets.py` in the plan).
2. **Training-data contamination** — the clean side already knows the original. *chardet*.
3. **Spec too thin or too design-y** — the "dirty" side ships a design, not a behaviour spec. *chardet* (architecture design doc); Ars Technica on why real projects lack complete specs.
4. **The clean team re-derives the old design** from convention, language, or tests. Ronacher's two examples; Knight & Leveson.
5. **The same spec writer on both sides** collapses the wall. *chardet*; this is the channel *NEC* closed and your setup does not.
6. **Similarity metrics measure the wrong thing.** chardet JPlag debate; ShiftMag's "the laundering was effective."
7. **Convergence mistaken for proof of constraint.** Knight & Leveson.
8. **Divergence mistaken for freedom** when the spec is merely ambiguous. SPECA / arXiv 2602.07513.
9. **"Made for litigation" clean rooms are evidentiary artifacts, not design processes.** Harvard JOLT.
10. **Even a real clean room can be found inadequate.** *SAS Inst. v. S & H*.
11. **Provenance contamination outlives the technical question.** chardet issue #331, where users call v7 "unacceptable legal risk."

**inferred — the most important consequence for your ticket.** The *NEC* convergence test is sound, but it depends on a premise your setup cannot reproduce: a clean team that has never been near the original, *and* a spec whose author sits behind the same wall. Your run would therefore produce a different artifact than *NEC* did. *NEC* produced evidence of *non-copying*. Your run would produce a **list of requirements-forced decisions versus history-forced decisions** — a design-reasoning result, not an independence result. State that up front or the comparison will be over-claimed.

**inferred — the single highest-value change to the method.** Preserve the clean agent's *reasoning*: its plan, its questions, and every point where it says "the spec is silent." Those are the spec gaps. Ronacher and the "Jas" authors both converge on this, and it is the one thing the *NEC* record also preserved ("every single piece of paper which Mr. Davidian saw").

---

## Related methods (to separate)

**N-version programming (Avizienis, 1985).** Multiple independently written versions run in parallel with a voter, assuming independently developed programs fail independently. A *fault-tolerance* method, not a spec-audit method; needs N full implementations and a voter. "[A] 1980s methodology that called for multiple independent implementations to improve reliability, but was largely abandoned because the cost of producing N independent implementations exceeded the reliability benefit" (https://arxiv.org/html/2606.07828v1).

**Knight & Leveson, 1986 — the independence assumption is false.** "A total of 27 versions of a program were prepared independently from the same specification at two universities. The results of executing the versions revealed that the versions were individually extremely reliable but that the number of input cases in which more than one failed was substantially more than would be expected if they were statistically independent." (https://www.csc.kth.se/utbildning/kth/kurser/DA2210/vettig13/Seminarier/KnightLeveson.pdf; IEEE TSE 12(1):96–109.) "[The] experiment did a service to the computing community as a whole by showing that failure independence of design faults cannot be assumed." (https://www.adelard.com/media/vfngprxq/divchap.pdf). **inferred**: agreement between two same-spec LLM designs is weaker evidence of "forced by requirements" than it looks, because agreement itself is not independent.

**Cleanroom software engineering (Mills, Dyer, Linger; IBM).** A *different* method with a confusingly similar name: formal specification, incremental development under statistical quality control, statistically designed testing to certify reliability. No walls, no reverse engineering, no two teams. (https://en.wikipedia.org/wiki/Cleanroom_software_engineering; primary: https://ntrs.nasa.gov/api/citations/19820016143/downloads/19820016143.pdf)

**Second-system effect (Brooks, 1975).** The successor of a successful first system becomes over-engineered because the designer is more confident and dumps in every deferred idea. Included only to separate it: a hazard of *building* a second system, not a method for auditing a first. It predicts exactly the failure where the clean design "improves" on the existing one and the comparison becomes unfair (https://en.wikipedia.org/wiki/Second-system_effect). Brooks's "plan to throw one away" is the reverse framing: here the *first* implementation is the pilot and the clean design is the lesson. **inferred**.

---

## Sources

Primary / authoritative:
- RFC 2026 §4.1.2 — https://www.rfc-editor.org/rfc/rfc2026.txt
- RFC 6410 §2.2 — https://www.rfc-editor.org/rfc/rfc6410.txt
- Harvard JOLT, *NEC v. Intel: Breaking New Ground in the Law of Copyright*, 3 Harv. J.L. & Tech. 209 (1990) — https://jolt.law.harvard.edu/assets/articlePDFs/v03/03HarvJLTech209.pdf
- *NEC v. Intel: A Guide to Using "Clean Room" Procedures as Evidence*, 10 Computer L.J. 453 (1990) — https://repository.law.uic.edu/cgi/viewcontent.cgi?article=1423&context=jitpl
- *NEC v. Intel: A Challenge to the Developing Law of Copyright…* — https://digitalcommons.law.scu.edu/cgi/viewcontent.cgi?article=1030&context=chtlj
- Knight & Leveson, IEEE TSE 12(1):96–109 (1986) — https://www.csc.kth.se/utbildning/kth/kurser/DA2210/vettig13/Seminarier/KnightLeveson.pdf
- Dyer & Mills, "Cleanroom Software Development" (IBM FSD, 1981) — https://ntrs.nasa.gov/api/citations/19820016143/downloads/19820016143.pdf
- W3C / WHATWG process rules — https://www.w3.org/2022/02/aria-charter ; https://whatwg.org/stages ; https://w3c.github.io/charter-drafts/2026/webtransport-wg-charter.html

Contemporary press and near-primary accounts:
- LA Times, 1989-02-08 — https://www.latimes.com/archives/la-xpm-1989-02-08-fi-2149-story.html
- NYT, 1989-02-08 — https://www.nytimes.com/1989/02/08/business/intel-loses-copyright-case-to-nec.html
- Computerworld (Schwartz), "Reverse-Engineering", 2001-11-12 — https://www.computerworld.com/article/1349695/reverse-engineering.html
- *PC Magazine*, "Phoenix Says Its BIOS May Foil IBM's Lawsuits", 1984-07-10 — https://books.google.com/books?id=Bwng8NJ5fesC&pg=PA56 (page not extractable)
- History of Information — https://historyofinformation.com/detail.php?entryid=3846
- The Chip Letter — https://thechipletter.substack.com/p/intel-vs-nec-the-case-of-the-v20s
- CenterConsulting, "The Clean Room That Cloned the PC" — https://www.centerconsulting.com/code-library/stories/phoenix-bios-clean-room
- "Compaq and the BIOS Clean-Room" — https://www.geschichte-der-informatik.de/articles/compaq_and_the_bios_clean_room/
- The Digital Antiquarian, "Send in the Clones" — https://filfre.net/2014/10/send-in-the-clones/
- Gary Davidian oral history (CHM, Hsu, 2019) — https://www.computerhistory.org/collections/catalog/102740488 (403 to this tool)

LLM clean-room and spec-only redesign:
- chardet #327 and Blanchard's reply — https://github.com/chardet/chardet/issues/327#issuecomment-4005195078
- chardet 7.0.0 — https://github.com/chardet/chardet/releases/tag/7.0.0
- Simon Willison — https://simonw.substack.com/p/can-coding-agents-relicense-open
- ShiftMag — https://shiftmag.dev/license-laundering-and-the-death-of-clean-room-8528/
- The Register — https://www.theregister.com/2026/03/06/ai_kills_software_licensing/
- Armin Ronacher — https://lucumr.pocoo.org/2026/3/5/theseus/
- Heather Meeker — https://heathermeeker.com/2026/04/09/the-chardet-controversy-open-source-and-the-ai-clean-room/
- Reed Smith — https://www.reedsmith.com/our-insights/blogs/technology-law-dispatch/102nbig/clean-room-coding-in-the-age-of-ai-who-owns-the-code/
- Anthropic C compiler — https://www.anthropic.com/engineering/building-c-compiler
- Ars Technica — https://arstechnica.com/ai/2026/02/sixteen-claude-ai-agents-working-together-created-a-new-c-compiler/
- METR / Epoch AI MirrorCode — https://metr.org/blog/2026-04-10-mirrorcode-preliminary-results/ ; https://epoch.ai/publications/mirrorcode-preliminary-results ; https://arxiv.org/html/2606.30182v1
- "Jas: AI-Paired Engineering as a Revival of N-Version Programming" — https://arxiv.org/html/2606.07828v1
- SPECA — https://arxiv.org/pdf/2602.07513
- "Can LLMs help find Ambiguities in Protocol Specifications?" — https://arxiv.org/html/2609.38752v1
- "The Polyglot's Dilemma" — https://arxiv.org/html/2608.18039
- "Why a Second FACET Implementation Found What Tests Missed" — https://rokoss21.tech/en/posts/facet-second-implementation/
- TLA+ / Raft reliability — https://arxiv.org/html/2302.02703v2
- Adelard, "Review of Software Design Diversity" — https://www.adelard.com/media/vfngprxq/divchap.pdf

Encyclopaedic (definitional framing only):
- https://en.wikipedia.org/wiki/Clean-room_design
- https://en.wikipedia.org/wiki/Phoenix_Technologies
- https://en.wikipedia.org/wiki/Cleanroom_software_engineering
- https://en.wikipedia.org/wiki/Second-system_effect

**Retrieval limitations.** `law.justia.com`, `courtlistener.com`, `casetext.com`, `retrocomputing.stackexchange.com`, `computerhistory.org` and `digitalcommons.law.scu.edu` all returned 403/410 to the fetch tool. The 1989 *NEC v. Intel* opinion is therefore quoted here **as reproduced in the two law-journal articles and Wikipedia**, not read directly. The Gary Davidian oral history is the best primary source on the clean room's practical execution and I could not read its transcript directly.
