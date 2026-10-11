# Browser environment

Machine setup for driving the Chronicler Engine UI from this container.

`SKILL.md` documents the investigation *method*. This file documents the machinery
that method depends on: which browser, which port, which files, and what breaks.

## What this environment is

A Docker container with no display and a single user (`node`). Two browser
toolchains are available, and both drive the **same** Chrome over CDP on
`127.0.0.1:9222`.

| Toolchain | Surface | Role here |
|---|---|---|
| `@narumitw/pi-chrome-devtools` | `chrome_devtools_*` tools | Primary path for ad-hoc interactive checks |
| `chrome-cdp` | `scripts/cdp.mjs` invoked through `bash` | Zero-dependency fallback for any session or managed agent with `bash` |

The extension registers eight tools: `chrome_devtools_load`, five stable DevTools
capabilities, and two experimental WebMCP gateways. Only the loader starts active
on models with native deferred-tool support; on models without it — the default
`opencode-go/deepseek-v4.1-flash` included — all five capabilities activate before
the first request. `/chrome-devtools enable` activates everything by hand.

## Start the browser

```bash
/home/node/.pi/start-browser.sh
```

`~/.bashrc` invokes it automatically whenever Chrome is down or the port file is
missing, so you normally only need to run it by hand after killing Chrome.

It is idempotent: it replaces any previous instance on the same profile, waits for
CDP to answer, then writes the port file. Overridable with `CDP_PORT`,
`CDP_PROFILE`, `CDP_CHROME`, `CDP_WINDOW`, and `CDP_LOG`.

### Default viewport

Chrome's headless default viewport is **780x437** — a size that matches no real
target, and which silently became the size of most screenshots until it was fixed.
The launcher asks for a desktop viewport instead:

```
--window-size=1280,943  ->  viewport 1280x800
```

headless=new reserves 143px of window chrome, so the window must be 943px tall to
yield an 800px viewport. That offset is a fixed constant, not a ratio. Since the UI
is desktop-first, 1280x800 is the correct default: ad-hoc shots land on the primary
target rather than an arbitrary one. A responsive sweep overrides it per shot: see
*Long reviews and viewports* below.

## Why Chrome cannot start itself

Chrome is installed at `/usr/bin/google-chrome` (154.0.8037.57). Three separate
constraints mean it has to be started deliberately, and each one defeated an
earlier attempt at a simpler setup.

**1. pi's managed launch crashes.** The extension's own launcher starts Chrome
without `--headless` or `--no-sandbox`, which a container requires. It fails with
`Auto-launched browser exited before DevTools became available`, and the managed
browser leaves core dumps behind (`core.<pid>` in the repo root, ~27 MB each, from
`--remote-debugging-port=0 --user-data-dir=/tmp/pi-chrome...`). `autoLaunch` is
therefore set to `false` so a missing browser produces a clear endpoint error
instead of a crash.

**2. Chrome refuses remote debugging on a default profile.** Pointing
`--user-data-dir` at `~/.config/google-chrome` fails outright:

```
DevTools remote debugging requires a non-default data directory. Specify this using --user-data-dir.
```

So the profile lives at `/tmp/pi-chrome-devtools`.

**3. Chrome only writes `DevToolsActivePort` in dynamic-port mode.** With
`--remote-debugging-port=0` it writes the file; with a fixed port it does not. But
`pi-chrome-devtools` needs the fixed port `9222`. The two requirements conflict.

The launcher resolves this by synthesizing the file from the live endpoint. The
file's directory is independent of Chrome's profile, so it is written to the path
`scripts/cdp.mjs` scans first, leaving Chrome free to use a non-default profile:

```
9222
/devtools/browser/861cedf3-7a02-432a-8fda-1b223d9969ed
```

Line 1 is the port, line 2 is the browser WebSocket path. `cdp.mjs` requires
exactly that shape. Because the browser UUID changes on every restart, the file is
rewritten on each launch — never hand-edit it. Writing it here also means no
`CDP_PORT_FILE` environment variable is needed, which matters because the variable
would have to propagate into every spawned subagent process.

## Files

| Path | Role |
|---|---|
| `/home/node/.pi/start-browser.sh` | Launcher: starts Chrome, writes the port file |
| `~/.pi/agent/pi-chrome-devtools.json` | `endpoint: http://127.0.0.1:9222`, `autoLaunch: false` |
| `~/.config/google-chrome/DevToolsActivePort` | Synthesized CDP discovery file read by `cdp.mjs` |
| `/tmp/pi-chrome-devtools` | Chrome profile — ephemeral, recreated on each launch |
| `scripts/cdp.mjs` | Vendored `chrome-cdp` CLI (32 KB, zero dependencies, Node 22+) |
| `/tmp/chrome-launch.log` | Chrome stdout/stderr; dbus errors here are noise |

The launcher and its config are machine-scoped, not repo files. `scripts/cdp.mjs`
is a vendored copy of the `pi-chrome-cdp` skill script, kept at a project-relative
path so the packaged skill's literal `scripts/cdp.mjs` commands resolve from the
repo root and so any `bash`-capable session or agent can drive it. The package also
ships its own copy at
`~/.pi/agent/npm/node_modules/pi-chrome-cdp/skills/chrome-cdp/scripts/cdp.mjs`.

## Headed mode is unavailable

There is no X server. `/tmp/.X11-unix/X0` and `X1` are stale sockets from a
previous run, and Chrome fails against them:

```
ERROR:ui/ozone/platform/x11/ozone_platform_x11.cc:257] Missing X server or $DISPLAY
ERROR:ui/aura/env.cc:246] The platform failed to initialize.  Exiting.
```

Two consequences:

- `HEADED=1 python build.py ...` cannot work here. The Playwright helper only sets
  `headless = false` when `HEADED=1`, so the default headless path is fine, but
  do not reach for headed mode to watch a test. `SLOW_MO` alone still works.
- **A shot is the only way to see the UI.** There is no window to glance at,
  which makes the look-at-the-shot rule in `SKILL.md` (step 3) the only
  visual check available rather than merely the preferred one.

## Coexistence with the Playwright tier

The browser tests do not conflict with this setup. Verified: the full browser tier
passed **20/20 in 125s** while this Chrome was running on 9222.

`tests/test_utils/browser.rs::launch_chrome()` launches through `playwright-rs`
with `channel: Some("chrome")`, so the tests drive the same system Chrome binary —
but Playwright manages its own temporary profile and connects over its own
transport, so there is no contention for port 9222, the profile directory, or the
port file. Nothing in the test harness reads `DevToolsActivePort`.

The one real interaction is resources. A running Chrome holding the dashboard
measures roughly **1978 MB resident across 24 processes** on a 4-CPU, 13 GB
container. The gate caps the browser tier at two parallel tests, so the margin
holds, but it is the reason to stop Chrome when you are done with it rather than
leaving it up indefinitely:

```bash
for pid in $(pgrep -x chrome); do kill "$pid"; done
```

Note the process comm name is `chrome`, not `google-chrome`, so `pgrep -x
google-chrome` matches nothing.

## Vision models and image caps

Providers cap the images one request may carry (`glm-5.3-flash` rejects at 31:
`Too many images in request`). Every `chrome_devtools_screenshot` puts its image
into the context, so a long review reaches that cap. The ticket-24 re-review took
33 inline shots and grew to about 315k tokens. Use `cdp.mjs shot` as *Long reviews
and viewports* shows, and use DOM dumps for structure. `@getpipher/vision`
is configured (`~/.pi/agent/vision.json`) to delegate image description to
`deepseek-v4-flash-vision-exp` for text-only primaries, with `glm-5.3-flash` as
fallback; `opencode-go/deepseek-v4.1-flash`, the current default model, is
multimodal itself and takes images natively.

## Long reviews and viewports

`scripts/cdp.mjs shot` writes the image to a file and returns only text. `read`
only the shots you need to look at. Its per-tab daemon holds one CDP session open.
An `Emulation` override lives as long as that session. A viewport set through
`evalraw` therefore applies to the next `shot`. The `chrome_devtools_*` tools use
their own sessions, so they do not see the override (verified for `evaluate`,
inferred for `screenshot`).

```bash
node scripts/cdp.mjs list                                   # <t> is the tab id prefix, for example 56B046B9
node scripts/cdp.mjs evalraw <t> Emulation.setDeviceMetricsOverride \
  '{"width":1024,"height":700,"deviceScaleFactor":1,"mobile":false}'
node scripts/cdp.mjs eval <t> "innerWidth+'x'+innerHeight"  # 1024x700
node scripts/cdp.mjs shot <t> tmp/<review>/30-game-1024x700.png
node scripts/cdp.mjs evalraw <t> Emulation.clearDeviceMetricsOverride '{}'
```

Verified 2026-10-09. `eval` reported `1024x700`. The PNG was 1024x700.
`clearDeviceMetricsOverride` restored the default. While the override was set,
`chrome_devtools_evaluate` on the same tab still read `1280x856`. The daemon keeps
the override for its idle life (20 min), so clear it before the next default-size
`cdp.mjs shot`.

## Delegated screenshot sweeps

The `visual-tester` agent that used to drive `scripts/cdp.mjs` shipped with
`pi-herdr-subagents` and is gone; pi-herdsman does not replace it. Its five
bundled definitions (`generalist`, `implementer`, `researcher`, `reviewer`,
`scout`) all declare `noSkills: true`, and none carries browser extension tools,
so a sweep needs the method written into the task:

- Use `generalist`: it has `bash` + `read`, and its model
  (`opencode-go/deepseek-v4.1-flash`) accepts image input, so it can run
  `node scripts/cdp.mjs list` / `shot` / `eval` and then look at the PNGs it
  captured. Verified by having one read `tmp/visual-tester-smoke/01-initial.png`
  and describe the layout.
- Give it the viewport sequence (**desktop first**: 1280x800 — the launcher's
  default — then 768, then 375) and the priority scale. The old agent's P0–P3
  went with it, so name the dashboard review's P1–P3
  (`.scratch/dashboard-ui-review/review-2026-09-29.md`).
- For a standing sweep definition instead of writing the method into a prompt
  each time, add a new project definition under `<cwd>/.pi/agents/`, which is
  empty today; a new name is standalone rather than an overlay.

## If the browser stops responding

```bash
/home/node/.pi/start-browser.sh
```

If it reports that Chrome did not expose CDP, read `/tmp/chrome-launch.log` and
ignore the dbus and GCM registration lines — neither service exists in this
container.
