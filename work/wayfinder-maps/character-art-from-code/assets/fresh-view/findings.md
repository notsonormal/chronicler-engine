# Art from code with a fresh view: findings

Ticket: [Research: art from code with a fresh view](../../issues/10-research-fresh-view.md).
Written 2026-11-03. All links were fetched in this session.

Labels: **known** = read in a primary source this session. **inferred** = a reasoned step from known facts, or from memory that I did not re-check. **guessed** = no evidence yet.

## Short answer

No family gives all of these together: (a) the painted look of `data/images/`, (b) a new Character with no image model, (c) Expressions, (d) clothes. The pixels of the painted look come from an artist or an image model. Every family that reaches that look takes its pixels from one of these two sources (inferred). Every family that makes new Characters from code alone stays well below that look (inferred, and the owner's six rounds agree).

The best fresh routes keep the painted picture as the source of the pixels. Code and small tools then add Expressions and motion. Three such routes exist today:

1. A neural animator moves one painted picture with a few numbers (THA3, THA4).
2. A layer splitter cuts a painted picture into parts, and a 2D rig moves the parts (See-through, then Stretchy Studio or Inochi2D).
3. An image model makes a whole sprite set once, and code only selects a frame (VNCCS).

Route 1 and route 2 write each Expression as data (a list of numbers). A coding agent can author that data (inferred). Neither route gives clothes cheaply.

One question belongs to the owner before any prototype: does goal 3 ("new Characters without an image model") mean "no image model at all", or "no image model for each Expression and each outfit"? The Characters of this project already have painted pictures from an image model (inferred from `data/images/` and the `profile_image` and `headshot_image` fields in the map Notes). Under the second meaning, routes 1 and 2 fit.

## What the target looks like

I looked at `data/images/lisette_headshot.png` and `data/images/gabriella_headshot.png` first (known).

- Soft gradients on skin and hair. No flat cel tones.
- Glossy hair with large, curved highlights and many thin strands. Hair is the largest part of the picture.
- Eyes are large but not the only focus. Fine lash lines, a small catch-light, a dark ring around the iris.
- A rim light, a lit patch on the nose and cheek, and a blurred room behind the head. Warm colour grade.
- Both pictures are close crops. The head is tilted. Lisette looks down with a lowered chin. Gabriella tilts the head and smiles. Hair covers part of the face in both.

Inferred: the pictures come from an anime diffusion model (the soft light, the lens blur and the glossy hair are typical of such output). The owner can confirm.

Consequence (inferred): the look depends on lighting, blur and strand detail more than on the shape of the face. Flat shapes, chibi or pixel art cannot reach it. This matches the failure in tickets 04 and 06.

## The families

Ten families. Families 1–8 come from the ticket's list. Families 9–11 are not in the list.

### 1. Parametric avatar libraries (in list)

- **Examples (known).** [DiceBear](https://www.dicebear.com/): 61 avatar styles, one seed gives one SVG. Libraries for JS, PHP, Python, Rust, Go, Dart, C# and a CLI. See the [style list](https://www.dicebear.com/styles/).
- **Look.** Flat vector. The page I fetched listed the "Minimalist" group only (blobs, shapes, identicons). I know from memory that other styles are cartoon people (Lorelei, Avataaars, Micah). I did not see an anime style (inferred).
- **How it is made.** An artist draws the style once as parts. A seed picks the parts. A new Character is a seed and some options. An Expression is a "mouth" or "eyes" option, if the style has one. Outfit options exist in some styles.
- **Tools and licence.** DiceBear code is MIT. Each style has its own licence: 42 styles are CC0, others are CC BY or MIT ([licences](https://www.dicebear.com/licenses/)) (known).
- **Image model.** No.
- **Fit.** Very good. A Rust crate exists. The SVG output is plain. The art could be made when the World is written (known: the seed is deterministic).
- **Closeness to the painted look.** Far. Flat vector faces (inferred).
- **Goal conflicts.** Goals 1–4 are all met in form. The look is the problem. The owner ruled out an outside kit in ticket 05.
- **Tried here?** The composition idea: yes (ticket 05). The look: no style of this kind was tried.

### 2. Layered character makers and visual-novel sprites (in list)

- **Examples (known).** [Ren'Py layered images](https://www.renpy.org/doc/html/layeredimage.html). The docs say a character with 4 outfits, 4 hairstyles and 6 emotions has 96 combinations, and layers remove the need to paint each one. `show augustina happy` swaps one attribute group. [Wildermyth](https://wildermyth.com/wiki/Image_layers) builds each hero from PNG layers sorted by a depth value, in five groups. Its look is hand-painted paper cutouts ([GamingOnLinux](https://www.gamingonlinux.com/2019/10/fantasy-tactical-rpg-wildermyth-blends-a-mix-of-hand-painted-2d-and-3d-art-arrives-on-steam-soon/)). The artist describes a Photoshop file per character ([blog](https://yonderinglands.blogspot.com/2018/06)).
- **Look.** Whatever the artist paints. For visual novels this is often painted anime. The look is as close to the target as the layers are.
- **How it is made.** A human artist paints every layer. A new Character, outfit or Expression is a new set of layers. Code only selects and stacks them.
- **Tools and licence.** Ren'Py is free (MIT, from memory: inferred). Layers: any paint tool.
- **Image model.** Not needed, but the layers can come from one (see family 8 and 10).
- **Fit.** Good. The stack is plain PNG or SVG. Ticket 05 already showed this works on a shared canvas.
- **Closeness.** Exactly as close as the layer art. The art is the cost.
- **Goal conflicts.** Goal 3: every new part needs an artist or an image model. The owner ruled out an outside kit.
- **Tried here?** Yes: tickets 05 and 07. The composer works. Agent-drawn pixel parts are in review.

### 3. 2D rigs with parameters (in list)

- **Examples (known).** [Live2D Cubism](https://www.live2d.com/en/sdk/license/), [Inochi2D](https://github.com/Inochi2D/inochi-creator) with [Inox2D](https://github.com/Inochi2D/inox2d), [Spine](https://esotericsoftware.com/spine-purchase). [Open-LLM-VTuber](https://github.com/Open-LLM-VTuber/Open-LLM-VTuber) ships a Live2D avatar with an LLM-driven expression map (it has `prompts/utils/live2d_expression_prompt.txt` and `.exp3.json` files).
- **Look.** The look of the source layers. Live2D VTuber models look like painted anime with soft shading. A rig adds motion without changing the pixels.
- **How it is made.** A rigger takes layered art (PSD). The rigger cuts meshes, binds parameters (eye open, mouth open, brow, head angle) and adds physics. An Expression is a preset of parameter values. This is human work, many hours per model (inferred; Live2D riggers say so, I did not measure).
- **Tools and licence.**
  - Live2D Cubism Editor is closed. The FREE edition has limits. The SDK is free to download. A release licence is needed when you publish. Individuals and small businesses are exempt, with exceptions for "Expandable Applications" ([licence page](https://www.live2d.com/en/sdk/license/)) (known). The runtime is proprietary (inferred).
  - Inochi Creator: BSD-2-Clause (known). Inox2D: BSD-2-Clause, "experimental", prototype state. It reads `.inp` and `.inx`, renders with OpenGL and WebGL, has parameters, deforms and physics. Animations and mesh groups are not done (known).
  - Spine: paid licence. The licence includes the runtimes. Enterprise is required above USD 500,000 revenue (known).
- **Image model.** Not needed for the rig. The layers come from an artist, or from family 10.
- **Fit.** Good if baked. Bake each Expression to PNG at World-writing time. Do not run a runtime in the server. A runtime in the browser needs glue code (Inox2D has a WASM and WebGL example; inferred).
- **Closeness.** Very close, because it moves painted art.
- **Goal conflicts.** Goal 1, 2: strong fit (blink, idle, mouth, brow are native). Goal 3: a human rigger per Character, plus layered art. Goal 4: outfit means new layers, and a new rig.
- **Tried here?** No.

### 4. 3D anime models with toon shading (in list)

- **Examples (known).** [VRoid Studio](https://vroid.com/en/studio) (free, from pixiv). It makes `.vrm` files. [three-vrm](https://github.com/pixiv/three-vrm) (MIT) loads them with the MToon toon material. [Amica](https://github.com/semperai/amica) (MIT) is an AI companion on three-vrm. It generates "response text that includes emotional expressions". VRM 1.0 has fixed Expression presets: `happy`, `angry`, `sad`, `relaxed`, `surprised`, `neutral`, `blink`, mouth shapes `aa ih ou ee oh`, and look directions ([spec](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md)).
- **Look.** A 3D figure with toon shading and outlines. Genshin-style games add a face shadow trick: a pre-drawn SDF "lightmap" gives clean anime shadows on the face at any light angle ([NoiRCCC blog](https://noirccc.net/blog/posts/33), [shader repo](https://github.com/NoiRC256/URPSimpleGenshinShaders)). The art team draws the lightmap and the face mesh (known). Default VRoid output looks like a figurine, not like a painting (inferred, not seen this session).
- **How it is made.** A human builds a Character in the VRoid Studio GUI: face sliders, hair, clothes, textures. The result is a file. An Expression is a VRM preset (a blend shape). New hair and clothes are VRoid items or textures. A coding agent cannot drive the VRoid GUI (inferred). A coding agent can edit the VRM JSON (colours, material values) but cannot sculpt a face (inferred).
- **Tools and licence.** VRoid Studio is free. Models "are yours to use freely", and the author sets terms ([VRoid](https://vroid.com/en/studio)). The Stable edition of the software has a commercial-use limit ([FAQ](https://vroid.pixiv.help/hc/en-us/articles/7818775338649-Can-I-use-VRoid-Studio-Stable-Ver-for-commercial-purposes)). three-vrm: MIT. [bevy_vrm](https://docs.rs/bevy_vrm/latest/bevy_vrm/): MIT or Apache-2.0, but it does **not** support expressions yet (known).
- **Image model.** No.
- **Fit.** Possible as a bake. A headless render (Blender, or three.js in a headless browser) writes one PNG per Expression when the World is written. Rust has no mature path to VRM expressions (known for bevy_vrm). Not suitable for a live render (inferred).
- **Closeness.** Medium. The shapes read as anime. The finish reads as 3D (inferred). A hand-tuned MToon material with a painted hair texture would go closer (guessed).
- **Goal conflicts.** Goal 1: the five VRM presets match the test list (neutral, happy, angry, surprised, sad) almost one to one. Goal 2: strong (3D idle, blink, look-at). Goal 3: a human with a GUI makes each Character. Goal 4: strong (VRoid has clothes as items).
- **Tried here?** No.

### 5. Procedural characters in shipped games (in list)

- **Examples (known).** Crusader Kings III builds each portrait from a DNA of more than ten times as many genes as CK II. Genes drive 3D face shape, textures and age. Children inherit the genes ([dev diary 34](https://forum.paradoxplaza.com/forum/developer-diary/ck3-dev-diary-34-its-all-about-appearances.1406933/page-8), [GDC 2021 talk](https://www.gdcvault.com/play/1027354/Creating-a-Portrait-System-Based)). Wildermyth: see family 2.
- **Look.** CK3 is realistic 3D with a painted-style finish. It is not anime. The pipeline needs a 3D team and hundreds of modelled assets (inferred from the talk summary).
- **How a Character is made.** A gene list. A human modelled every morph target and texture once.
- **Image model.** No.
- **Fit.** Poor for a small project. The assets are the cost.
- **Closeness.** Far from the target look. Useful lesson: a gene list per Character plus one shared asset set is the pattern that ticket 06 already built.
- **Goal conflicts.** Goal 3 is met only after a large upfront art cost.
- **Tried here?** The gene-list idea: yes (ticket 06).

### 6. Shader-based and distance-field faces (in list)

- **Examples (known).** The Genshin face-shadow SDF above (needs a 3D face). Pure 2D shader faces appear on shader sites (Shadertoy). I did not survey them.
- **Look.** 3D toon shader: see family 4. Pure shader faces: abstract and geometric (guessed, not surveyed).
- **How it is made.** Code and a few textures. A graphics programmer writes the shader. A coding agent can write shaders (inferred).
- **Image model.** No.
- **Fit.** A shader needs a GPU or a headless GL render at bake time. Heavy for a Rust and Askama server (inferred).
- **Closeness.** The 3D toon shader helps family 4. A pure code face is the "model writes every shape" case again (inferred).
- **Tried here?** No. I record this family as a helper for family 4, not as a route of its own.

### 7. Models that write SVG or drawing code (in list)

- **Examples (known).** Public benchmarks compare how well language models draw SVG: [Rapidata human-preference benchmark of 42 models](https://huggingface.co/datasets/Rapidata/text-to-svg-human-feedback-benchmark/blob/bbb5fde7d01c82bad242fd555a9ed2a2fad38b06/README.md) and a [dragon benchmark of 43 models](https://counterfiction.com/articles/dragon-benchmark). The dragon article describes the field as "a display of children's drawings" (its own subtitle). Top models score better.
- **Look.** Clean vector shapes with simple gradients. Models draw without seeing, unless they render and look (known from the dragon article).
- **How it is made.** A coding agent writes the SVG. It can add Expressions and outfits by editing the file. This is the cheapest route for goal 3, 1 and 4.
- **Image model.** No.
- **Fit.** Perfect: plain files, inline SVG.
- **Closeness.** Far. The owner's own six rounds show it (known: tickets 04, 06).
- **Tried here?** Yes. Rejected for the whole picture.

### 8. A kit made once by an image model, composed by code (in list)

- **Examples (known).** [VNCCS](https://github.com/AHEKOT/ComfyUI_VNCCS) (MIT, ComfyUI nodes). It makes a base Character, clothing sets, emotion sets and poses with a consistent face. It targets visual-novel sprites. It names Illustrious as a base model.
- **Look.** The look of the image model. This is the look of the target pictures (inferred).
- **How it is made.** A person runs a ComfyUI workflow on a GPU. The tool keeps the face consistent. A new Character, outfit or Expression is a new run. Code selects the frame.
- **Image model.** Yes, at World-writing time. That is the whole method.
- **Fit.** Good for runtime: plain PNG, no code. The author needs a GPU and ComfyUI.
- **Closeness.** Closest of all (inferred). Consistency across Expressions can still fail (inferred; the tool exists because it fails).
- **Goal conflicts.** Goal 3 fails by definition. Goal 2 fails (frames, no motion beyond a swap). Goals 1 and 4 are met.
- **Tried here?** No.

### 9. Single-image neural animators (NOT in the ticket list)

- **Examples (known).** [Talking Head(?) Anime from a Single Image 3](https://github.com/pkhungurn/talking-head-anime-3-demo) (THA3) and [version 4](https://github.com/pkhungurn/talking-head-anime-4-demo) by a pixiv researcher. The input is one anime picture. The output is the same picture with a changed face and head. A [project page for THA4](https://pkhungurn.github.io/talking-head-anime-4/) has comparison videos.
- **Look.** It is the input picture, so the look is the painted look. Artefacts appear on hard poses (inferred).
- **How it works (known).** THA3 takes a pose vector. Its parameter file lists: `eyebrow_troubled`, `eyebrow_angry`, `eyebrow_lowered`, `eyebrow_raised`, `eyebrow_happy`, `eyebrow_serious`; `eye_wink`, `eye_happy_wink`, `eye_surprised`, `eye_relaxed`, `eye_unimpressed`, `eye_raised_lower_eyelid`, `iris_small`; `mouth_aaa`, `_iii`, `_uuu`, `_eee`, `_ooo`, `_delta`, `_lowered_corner`, `_raised_corner`, `_smirk`; iris rotation; `head_x`, `head_y`, `neck_z`; `body_y`, `body_z`; `breathing`. Each Expression is one vector of numbers. That is data an author or a coding agent can write. A blink or an idle loop is a short series of vectors.
- **Input limits (known).** 512×512 with an alpha channel. One humanoid, upright and facing forward. Hands away from the head. The head roughly in a 128×128 box in the top half. Background must be transparent.
- **Tools and licence (known).** Code is MIT. THA3 models: CC BY 4.0 (commercial use allowed, with credit). THA4 models: CC BY-NC 4.0 (non-commercial). Needs an Nvidia GPU with CUDA and PyTorch. THA4 can train a small per-character "student" model of under 2 MB. Training took about 30 hours on an RTX A6000. The web conversion code is not released.
- **Image model.** A neural model runs at bake time, and the input picture usually comes from an image model. It is not a diffusion model.
- **Fit.** Good. Bake each Expression to a PNG when the World is written. The Rust server never runs it. The GPU is needed only on the author's machine (the sandbox has none: I ran `nvidia-smi`, it is not installed).
- **Closeness.** Very close for the base face, because it moves the real picture. Risk (inferred): the two headshots break the input rules. They are close crops, tilted, with a background, and hair covers part of the face. Prepare the input by cutting the head out and placing it in the 512×512 frame. Results on these two pictures are unknown.
- **Goal conflicts.** Goal 1: strong. Goal 2: blink and breathing are in the parameter list. Goal 3: each Character needs a painted picture. Goal 4: no clothes. The model cannot change an outfit.
- **Tried here?** No. Ticket 08 converts the painted pictures with colour reduction and edges (a classical filter). It does not use a learned model.

### 10. Layer splitting of a painted picture, then a rig (NOT in the ticket list)

- **Examples (known).**
  - [See-through](https://github.com/shitagaki-lab/see-through) (paper [arXiv 2602.03749](https://arxiv.org/abs/2602.03749)). One anime picture in, a PSD out with up to about 23 layers: front hair, back hair, face, eyes, clothes. Hidden regions (the head behind the hair, the body under clothes) are painted in. A [hands-on review](https://lilting.ch/en/articles/see-through-anime-layer-decomposition) calls it "an automatic PSD draft generator", not a full Live2D maker.
  - [Qwen-Image-Layered](https://github.com/QwenLM/Qwen-Image-Layered): splits any image into RGBA layers. Apache 2.0.
  - [Stretchy Studio](https://github.com/pelmentor/stretchystudio) (MIT). A web editor. It imports a tagged PSD and builds a rig with face parallax, eye closure, mouth open, head angle and hair physics. Layers named `mouth.smile` or `topwear.winter` become crossfade parameters (variants). It exports Live2D runtime files, Spine JSON, and PNG, WEBP or JPG sequences. It has a CLI for idle, talking and glance motions.
  - [Bunraku](https://bunraku-live2d.github.io/) (paper [arXiv 2607.27348](https://arxiv.org/abs/2607.27348)). One illustration to a layer stack, then to a Live2D rig with predicted vertex offsets. A research demo.
- **Look.** The painted picture itself, plus inpainted hidden parts. The inpainted parts come from SDXL-class models, so they match the style (inferred).
- **How it is made.** A person or script runs See-through on a picture. A person fixes layers by hand. Stretchy Studio tags the layers and builds the rig. An Expression is a set of parameter values, or a variant layer such as `mouth.smile`. An outfit is a variant layer, but the hidden body under the new outfit must be painted in (inferred).
- **Tools and licence.** See-through code: Apache 2.0 (known). The licence of its model weights: not checked. Needs about 12–16 GB of VRAM at 1280 px, or about 8 GB with 4-bit weights, and about 14 GB of models (known).
- **Image model.** Yes, at bake time (SDXL-based). The input picture also comes from an image model, usually.
- **Fit.** Good if baked. The output is PNG sequences. The authoring tool is a web app with Node (Vite and React). The tool is for the author, not for the Chronicler server. How stable the tool is: guessed (the repo is young and has one main author).
- **Closeness.** Very close at rest, since it is the painted picture. Motion is limited to small mesh warps (inferred from the Bunraku page: summing many large parameter moves pulled layers apart).
- **Known limits of See-through (known).** Extremely complex frills. Hands over the face. Full scenes with background. Chibi. The two headshots have hair over the face and a background. This is a risk (inferred). The tool is trained on full-body VTuber-style standing pictures (inferred from the "A-pose" emphasis of the Bunraku page).
- **Goal conflicts.** Goal 1, 2: strong. Goal 3: a picture is needed. Human tidy-up per Character. The hands-on review sets "3 hours of manual splitting become 1 hour" as a test goal, not as a measured result. Goal 4: variants are possible, but each outfit is new art.
- **Tried here?** No. Ticket 08 is the nearest, but it flattens colours and does not split layers.

### 11. GAN latent spaces for anime faces (NOT in the ticket list)

- **Examples (known).** [This Waifu Does Not Exist](https://www.thiswaifudoesnotexist.net/) (StyleGAN 2, 512 px). [Gwern's write-up](https://gwern.net/face) says the faces are CC0. [AnimeFaceNotebooks](https://github.com/halcy/animefacenotebooks/) edits tags such as smile with directions in latent space. The directions come from regression against tagger scores.
- **Look.** Soft-shaded anime faces, 2019 level. Hair and eyes show model artefacts at the edges (inferred, from memory of the samples).
- **How it is made.** A seed gives a face. A direction changes one attribute. A new Character is a seed.
- **Image model.** Yes, but a small, old, local one (a GAN, not a diffusion model).
- **Fit.** Runs at bake time on a GPU. Faces only. No clothes. No stable identity when an attribute changes (inferred). The Character looks different in each Expression unless the edit direction is small.
- **Closeness.** Medium. Style is closer than vector art. Quality is below the target (inferred).
- **Goal conflicts.** Goal 3: yes, from a seed, with no prompt of the picture. But you cannot say "this hair colour and this fringe" exactly. Goals 1 and 4: weak.
- **Tried here?** No.

## Noted, not surveyed

- **A painted-finish pass over code-drawn shapes** (soft gradients, rim light, grain, blur, with SVG filters or a raster pass). It would go after the look gap, since the gap is in the lighting (inferred). I found no shipped example. I did not test it. Status: guessed.
- **A code-drawn guide and an image model that repaints it** (for example ControlNet from line art). It keeps the pose and the Expression as data, and the finish comes from the model. It needs an image model. Not researched.
- **Vector tracing of a painted picture** ([VTracer](https://github.com/visioncortex/vtracer), Rust). Not researched. It is the classical side of ticket 08.

## Fit with the ranked goals

Table. "Y" = met, "~" = partly, "N" = conflict.

| Family | Look vs target | 1 Expressions | 2 Animation | 3 New Character, no image model | 4 Clothes | Needs GPU at bake |
|---|---|---|---|---|---|---|
| 1 Avatar libraries | far | Y | ~ | Y | Y | no |
| 2 Layered makers | as good as the layers | Y | ~ | N (artist) | Y | no |
| 3 2D rigs | very close | Y | Y | N (rigger) | ~ | no |
| 4 3D toon (VRM) | medium | Y | Y | ~ (GUI by a human) | Y | no |
| 5 Game gene systems | far | Y | ~ | ~ (after big art cost) | Y | no |
| 6 Shader and SDF | helper only | ~ | ~ | ~ | ~ | maybe |
| 7 Model writes SVG | far (tried) | Y | ~ | Y | Y | no |
| 8 Image-model kit | closest | Y | N | N | Y | yes |
| 9 Neural animator | very close | Y | Y | N (picture) | N | yes |
| 10 Layer split and rig | very close | Y | Y | N (picture) | ~ | yes |
| 11 GAN faces | medium | ~ | N | ~ | N | yes |

## What this map already tried (read after the survey)

I read tickets 04 to 08 and the map index last.

- Family 7: tried in tickets 04 and 06. Rejected.
- Family 2 (composition only): tried in tickets 05 and 07. The composer works. The agent-drawn pixel parts are in review.
- Family 1: the parametric idea was tried as a generator in ticket 06. No library look was tried.
- A classical conversion of the painted pictures: ticket 08, running.
- Families 3, 4, 5, 6, 8, 9, 10 and 11: not tried.

None of the six rounds used a learned model or a rig.

## Recommended for prototypes

Pick routes where the pixels already look painted. Test whether the two weak points hold: the awkward crops, and the cost of a third Character.

### A. Single-image neural animator on the two painted headshots (family 9)

**Question.** After a head cut-out and a 512×512 frame, can THA3 turn `lisette_headshot.png` and `gabriella_headshot.png` into five Expressions (neutral, happy, angry, surprised, sad) that a reader tells apart at 80 px, and a blink pair, with no visible tearing? Each Expression is one parameter vector.

Why: the shortest path to the painted look. The Expression data is a small JSON that an agent can write. Cost of a third Character: one picture and one cut-out. Needs a rented GPU or Colab (the THA3 README offers a Colab notebook) (known). Choose THA3 first because its models are CC BY 4.0. THA4 is non-commercial.

### B. Layer split and rig on the same two pictures (family 10)

**Question.** Does See-through (the Hugging Face demo or a rented GPU) split the two headshots into layers that are clean enough, and does Stretchy Studio's auto-rig then give a blink, a mouth change and brow moves that bake to PNG at 80 px? Record the hand-fixing time for each Character.

Why: it tests the one route that gives idle motion (goal 2) and outfit variants (goal 4) on the painted look. It also shows whether the crops in `data/images/` need to be re-made as bust pictures.

### C. A baked VRM toon model (family 4)

**Question.** Can a VRM model, rendered headless with its five Expression presets, read as "somewhat like" the painted pictures at 80 px, and what is the cheapest way to make a second face with a different hair colour, fringe and outfit, from a coding agent and no GUI?

Why: it is the only family here where new Characters, Expressions and clothes are all parts and numbers, without a painted picture. It is the weakest on the look (inferred). The prototype should decide this by sight. Use an existing free VRM model first. Do not learn VRoid Studio before the look passes the owner.

### Comparison

| | A. Neural animator | B. Layer split and rig | C. Baked VRM |
|---|---|---|---|
| Look vs target | very close | very close | medium |
| Needs a painted picture per Character | yes | yes | no |
| Image model in the loop | yes, at bake | yes, at bake | no |
| Goal 2 (animation) | blink, breathing | strong | strong |
| Goal 4 (clothes) | no | variants, with work | strong |
| Main risk | tilted crops break the input rules | layer errors on hair over face | looks like a 3D figure |
| Main cost | a GPU for the author | hand fixing per Character | a human with a GUI per Character |
| Licence flag | THA4 weights non-commercial | check See-through weights | check the licence of each VRM |

## Open points

- I did not run any tool. All statements about quality on the two headshots are inferred.
- The sandbox has no Nvidia GPU (known). A prototype of A or B needs a GPU elsewhere.
- The licence of the See-through model weights and of the Bunraku code is not checked.
- The state of Live2D web runtime licensing for a local, self-hosted app is not settled. The "Expandable Application" rule could apply to a tool with user content (guessed). Ask Live2D, or avoid the Live2D runtime and bake to PNG.
- I did not survey: Picrew and charat.me (human-drawn layered makers), MMD, Blender-based anime pipelines, Sims and Mii style systems. Each is a variant of family 2 or 4 (inferred).
