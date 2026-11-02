# tibo.dog: what the shipped game shows

Collected on 2026-11-02 from the files that `https://tibo.dog/` serves. The source repo is not public, so everything here comes from the build output.

## The claim

@poteto's trivia post (2026-11-01): "all the pixel art is code. every sprite is a file drawn with one shared palette and packed into a sprite atlas at build time." The post does not say how a sprite file is written.

## What the build output shows (known)

- **One PNG atlas.** `assets/atlas-l1vojoXn.png` is 1024×1728, indexed colour (PNG mode `P`), 102 colours, and 131 KB. The "one shared palette" is the palette of this PNG.
- **A generated manifest.** `assets/atlas.generated-*.js` exports `ATLAS`. Each sprite has named clips. A clip has a `kind` (`loop`, `once` or `variants`) and an `fps`. Each frame has a rectangle `x, y, w, h`, an anchor `ax, ay`, and for some sprites a hand point `hx, hy`. There is also a late atlas that loads after the first screen.
- **No pixel work in the browser.** The browser loads the PNG and draws rectangles from it. It does not recolour or composite layers.
- **Two layers for the hero.** `tibo-body` and `tibo-arm` are separate sprites. The game joins them at the hand point, so the arm swings and holds weapons. There are no separate eye, mouth or clothes layers.
- **Expressions are whole frames.** `tibo-wake` has the poses `out`, `twitch`, `push`, `kneel`, `rub` and `stand`, and each pose is one complete drawing.
- **Size.** A Tibo body frame is about 38×87 px, and the face is about 20 px wide. The faces still show expressions: [crop of the atlas, 2× zoom](atlas-tibo-crop.png), [the whole atlas at half size](atlas-half-size.png).
- **Not every image is a sprite.** Cutscene backgrounds ship as separate 240×240 indexed PNG and WebP files. It is not known whether code makes them.

## Inferred

- At 38×87 px with readable faces, the source is probably not a text grid that a person types by hand. It can be drawing code, an editor export, or grids that an AI writes.

## A second source: Frostward

[Frostward devlog](https://digitalacresstudios.itch.io/frostward/devlog/1595764/my-pixel-art-game-has-no-art-files-or-assets-here-is-how-and-why) (2026-07-20). Each sprite is a grid of characters plus a palette map, and `.` is transparent. The game bakes each grid into a canvas once. Shape and colour are kept apart, so one shape takes many palettes, which gives recolours and seasons. A converter of about 300 lines turns an Aseprite PNG into a grid, with a strict `--palette` mode. The author's stated limit: the method works for 16×16 to 32×32 sprites with 4 to 10 colours. At 64×64 with gradients, PNG is smaller, and a person can no longer read or edit the grid.
