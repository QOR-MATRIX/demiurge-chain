# ADR-105: QQ makes real games, in 3D and in 2D, from one runtime

**Status:** **Accepted**, 19 October 2026, under the owner's direction that day: "Ensure that the game engine is high
quality and can produce real games in 3D and 2D … This Entire project should appeal to gamers and developers while
providing them the best possible suite of tools in existence", with the order of work and the engineering choices left
to the recommendation ("proceed with the next steps forward as best recommended in sequence"). The route and order below
are those engineering choices. **Extends** [ADR-083](ADR-083-qq-on-qt.md) (QQ on Qt 6) and
[ADR-084](ADR-084-qq-player-on-the-web.md) (the Player's web build); **keeps** [ADR-087](ADR-087-strangers-games-are-confined.md)
(a stranger's game is confined) and [ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) decision 5 (a rewarded run is
replayed by the server). It decides **no economic value**. **Decisions 1 and 5 amended** by [ADR-106](ADR-106-2d-scenes-say-whether-their-art-is-pixel-art.md) (19 October 2026): whole-number scaling is for pixel art, which a scene says it has, and a tile layer is one mesh rather than instanced.

## Context

QQ (`products/qq/`, about 10,700 lines) plays 3D worlds well: physically based materials lit by a procedural sky, soft
shadows, filmic tonemapping, bloom and ambient occlusion; PhysX bodies and a character controller; keyboard, mouse and an
XInput gamepad reduced to move, look, jump and run; one particle plume; spatial sound; logic files rebuilt on each save;
a Studio, a locked-down Player that also runs in a browser, and fourteen tools an agent drives. Eight Qt Test suites, 73
cases, pass on the owner's PC.

What it cannot do yet is what a game needs beyond a world (surveyed 19 October 2026):

- **Gameplay.** Logic gets a target, the scene, the elapsed time and a frame signal. It cannot hear a collision, see a
  trigger, cast a ray, make or remove an entity, wait, or push a body.
- **An interface.** Nothing draws a score, a health bar, a title screen, a pause menu or a message over the world.
- **Input.** Four fixed intentions. No named actions, no rebinding, no pointer or click into the world, no touch.
- **2D.** None. There is only a perspective camera, and no sprite, sheet, tile map or 2D body. The launcher's TypeScript
  preview draws 2D shapes, but ADR-083 decision 6 retires it.
- **Determinism.** Logic steps by the frame's time, so a run cannot be replayed exactly, which ADR-102's rewarded runs
  need.
- Also missing: animation, an audio mix, textures, editor basics (undo, duplicate, snapping, a hierarchy, an asset
  browser), a save API and a profiler.

Two routes to 2D were weighed:

- **2D inside the same Qt Quick 3D world:**
  - What it is: an orthographic camera, sprites as textured quads, tile maps as instanced quads, and PhysX bodies locked
    to the plane.
  - For it: the Studio, the agent's tools, the scene format, the lockdown, the web build, lights, bloom and particles all
    serve 2D unchanged, and a 2D game can light its sprites and mix 3D props in.
  - Against it: PhysX locked to a plane is not Box2D. A platformer's feel comes from a kinematic controller rather than
    from rigid bodies, which is also how Box2D platformers are made.
- **A separate Qt Quick 2D world:**
  - What it is: Qt Quick's own items, with Box2D added for physics.
  - Against it: a second runtime, a second scene vocabulary, Studio and set of agent tools, and a new C library in the
    confined Player.

## Decision

1. **One runtime.** QQ's 2D is the same Qt Quick 3D world seen through an orthographic camera, not a second engine. A
   scene says which it is (`view: "3d"` or `"2d"`); a 2D scene's camera looks down the depth axis with one world unit to
   a chosen number of screen pixels, scaled only by whole numbers so pixel art stays sharp.
2. **Gameplay belongs to logic, through a small API every kind shares** (P3.7):
   - contacts, and **Trigger**, a volume that reports what enters and leaves it;
   - **raycasts** against bodies;
   - **making and removing** entities while the world plays, from a kind and its fields;
   - **timers**: once after a time, or every so often;
   - **forces and impulses** on dynamic bodies, and **kinematic** bodies that logic moves.

   A game is a scene and logic files, never a change to the engine.
3. **An interface layer** (P3.8): a **Screen** kind drawn over the world in Qt Quick, holding text, bars, images and
   buttons whose values logic sets. A game can have a title screen, a pause menu and a message. It follows the design
   system's type and colour tokens by default, and honours reduced motion.
4. **Input actions** (P3.9): a game names its actions (`fire`, `interact`, `pause`, …) with default bindings for
   keyboard, mouse, gamepad and touch. A player can rebind them. The pointer can pick what it is over in the world. The
   four intentions stay as the built-in actions, so today's games keep working.
5. **2D kinds** (P3.10):
   - **Sprite**: an image or a frame of a sheet, with frame animation, nearest filtering where pixel art asks for it,
     and lit or unlit;
   - **TileMap**: a grid of tiles from a tile sheet, instanced, imported from Tiled's JSON map format;
   - **bodies locked to the plane** for shapes and sprites;
   - a **2D player**: a kinematic platformer or top-down controller.
6. **Fixed steps and replay** (P3.11):
   - Logic steps at a fixed rate (60 a second), whatever the frame rate.
   - Random numbers come from a seed the scene or ARQADE gives.
   - A run's inputs can be recorded and replayed, and a replay with no window reaches the same outcome.
   - A rewarded outcome (ADR-102 decision 5) depends only on logic and QQ's own kinematics, never on PhysX, whose results
     are not promised to match across machines.
7. **Then, in this order** (P3.12 to P3.16):
   - animation: glTF skeletal and morph playback, tweens on any field, and a state machine;
   - sound: one-shots from logic, music, and volume groups, panned in a browser where it can be;
   - materials: images on shapes, spot lights, and HDR skies;
   - the Studio's basics: undo and redo, duplicate, snapping, a hierarchy with parenting and reusable prefabs, an asset
     browser, a logic editor, and a statistics overlay with physics shown;
   - a save API: one small store per game that the lockdown allows.
8. **Each step is proven by a game.** Samples in `products/qq/samples/`, made only from kinds and logic files, are played
   by the Qt Test suites:
   - a 3D arena (gameplay, interface, input);
   - a 2D platformer (sprites, tiles, the 2D player);
   - a top-down 2D game (pointer, triggers, timers).

   The agent's tools learn each new kind in the same change, so a description can produce a 2D game too.
9. **What stays:**
   - the lockdown: a game reaches no network and no file outside its package, and a save is the one store decision 7
     gives it;
   - the canonical scene format: one changed field is one changed line;
   - the Studio and the Player share the runtime;
   - suites run on the owner's PC until CI can build Qt (ADR-083).

## Consequences

- `docs/DIRECTION.md` gains P3.7 to P3.16 in this order.
- `products/qq/README.md` and `docs/blueprints/qq.md` describe 2D as part of QQ.
- P7.20 (ARQADE's four games rebuilt in QQ) depends on P3.7, P3.8, P3.9 and P3.11.
- The TypeScript preview still retires at P3.5. Its 2D scenes import to 2D QQ scenes once P3.10 is built.
- Networked multiplayer is not decided here. ADR-087 forbids a game the network, and P7.24's arenas need a record of
  their own.
- The cost is one runtime's complexity rather than two. A 2D game carries Qt Quick 3D's size, about 44 MB for the web
  build (ADR-084), until a smaller web build is measured.
