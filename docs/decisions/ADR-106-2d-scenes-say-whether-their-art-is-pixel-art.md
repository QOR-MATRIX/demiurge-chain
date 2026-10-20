# ADR-106: A 2D scene says whether its art is pixel art, and a tile layer is one mesh

**Status:** **Accepted**, 19 October 2026, under the owner's direction that day, which left the engineering choices of
QQ's 2D to the recommendation ("proceed with the next steps forward as best recommended in sequence"), as
[ADR-105](ADR-105-qq-makes-real-games-in-3d-and-2d.md) was. **Amends** ADR-105 decisions 1 and 5, as built in DIRECTION
P3.10. It decides **no economic value**.

## Context

ADR-105 decision 1 says a 2D scene's camera is "scaled only by whole numbers so pixel art stays sharp". Building P3.10
showed two kinds of 2D art that one rule cannot serve:

- **Pixel art**, placed pixel by pixel: it must be scaled by whole numbers, or some art pixels are drawn a screen pixel
  wider than others. It must also be shown in exactly the colours it was painted. QQ's 3D look has filmic tonemapping
  and bloom, and the bloom brightened a painted `#28c8c8` to `#65ffff` in the checks, because it spreads all of the
  image's light, not only the brightest.
- **Drawn or rendered art**, and shapes: the 2D scenes the launcher's preview made are this kind, and so are games
  made from smooth sprites or 3D props seen flat. Whole-number scaling gives them nothing and costs their framing. A
  preview scene made for a view 540 pixels tall, at 100 pixels to a unit, shows nearly twice its height in a window
  1,000 pixels tall, because no whole number between 1 and 2 exists. Without bloom and filmic tonemapping, its glowing
  orange core clipped to yellow.

ADR-105 decision 5 says a TileMap is "instanced". A tile layer never moves, so it was built as one mesh instead: a quad
for each tile, with the texture coordinates of that tile's corner of the sheet. It is one draw call, as instancing would
be, with no per-instance data, and every tile lands on exact texels.

## Decision

1. **A 2D scene says whether its art is pixel art** (`pixelArt`, a field of the scene, on by default). With it on:
   - the view is scaled by whole numbers, as ADR-105 decision 1 says;
   - the camera is moved to the nearest device pixel;
   - colours are shown as painted: linear tonemapping, no bloom and no dithering.

   With it off, the view is exactly `viewHeight` units tall at any scale, and keeps the 3D look's tonemapping and
   bloom. Either way a 2D scene has no ambient occlusion. A preview scene imports with it off.
2. **The Studio edits either kind at any zoom**, in the look the scene plays in, and plays each as the Player shows it.
3. **A tile layer is drawn as one mesh**, rebuilt only when its map or its scale changes. Its solid tiles become the
   fewest rectangles that cover them, one collider each.

Everything else in ADR-105 stands.

## Consequences

- A scene file has one more field (`pixelArt`), written on every scene as every field is. The canonical fixtures and the
  Orb Run sample carry it.
- A creator choosing smooth art turns one switch off; nothing else about a 2D game changes.
- `tst_play` checks pixel art pixel for pixel: square art pixels of a whole number of device pixels, sharp edges, and
  colours as painted. `tst_studio` checks that a drawn scene fills its view exactly while it is edited.
