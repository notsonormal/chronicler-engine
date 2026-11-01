# Excerpt: Lauren Tan on feature maps

Status: reference

This excerpt keeps the feature-map passage of a talk. The full transcript is in `tmp/`, which is git-ignored, so it can be lost. The map entry **Docs organized as feature maps** cites this passage.

| Field | Value |
|---|---|
| Talk | "how I shipped 2,500 PRs last month to production" |
| Speaker | Lauren Tan (poteto) |
| Recorded | 2026-09-21 |
| Source | https://x.com/poteto/status/2102050467505430555 |
| Full transcript | `tmp/poteto-2500-prs-talk-2026-09-21.md` (sha256 of the source in its front matter: `e6e5d4c32e0526f974eb88dae7ee62ad66bc9c9d853ccb4b421730b58200e660`) |
| Transcription | Local ASR (faster-whisper large-v3-turbo). Verbatim, with the normalisations listed in the full transcript. The text has ASR errors and is not edited here. |

The passage starts at [08:55], where she describes the two parts of a verification skill: a CLI and a feature map. It ends at [11:23], one paragraph before the talk turns to `pstack`.

## Passage

**[08:55]** And it does that through the Chrome DevTools protocol. Something interesting about that skill is, and I kind of iterated my way to this. It didn't start out this way. But the control or verification skill really has two components to it. The first part is obviously a CLI, so you want your agent to be able to reproducibly be able to run the application and collect traces and collect evidence that, you know, empirical evidence that, you know, code is working, that your performance bar is being met. And rather than have your agents create scripts every time, you know, that can differ between agent sessions, you can actually create a CLI that's within the skill directory, and then your agents will just use that every time. And then, of course, you need to actually invest in it and make it good so that it can handle all sorts of different use cases

**[10:05]** and be able to run the application correctly.

**[10:16]** Another thing that's also really important is this idea of a feature map. So a feature map really is something that I kind of coined, I guess, where, you know, we started using these control skills within Cursor. Then we quickly realized that, you know, a Slack report would come in and a user would post a very vague screenshot, right? Like a very small slice of the UI and just like three question marks. And the agents using our control skills are just no idea. like it could run the application but it was just guessing at what exactly the the user meant and so i had this idea to create something called a feature map which is i guess kind of inspired by like a site map it essentially is a form of materialized memory you know like how exactly does your application work what features does it have how does a user reach it you know in terms of keyboard shortcuts or what DOM elements to click on, you know, things like that, and what all the

**[11:23]** different features do. And they are these, this feature map is stored in the skill itself in the code base as part of the skill directory. And we have an automation that maintains this feature map as well. But when we combine the CLI and the feature map, we quickly realized that this combination was very very powerful because now agents could not only you know reproducibly control the application and take traces but it could also understand requests that came in from internal users as well as external users and so we very quickly realized that the control verification skills were so useful that they've become more or less a critical infrastructure for team and we constantly maintain it. But the ability for an agent to verify its own work is extremely powerful and we have spent a lot of time on this skill and it's very very powerful for
