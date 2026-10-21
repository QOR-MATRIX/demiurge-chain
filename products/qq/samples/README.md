# QQ samples

Games made only from QQ's kinds and logic files, with no change to the engine (ADR-105 decision 8). Each one proves a
step of DIRECTION P3, and the Qt Test suites play it. Each is a project as QQ Studio keeps one: `scenes/`, `logic/`,
`assets/` and, where it has them, `prefabs/`, opened with QQ Studio or played with the QQ Player.

| Sample | What it proves | Played by |
| --- | --- | --- |
| [`orb-run/`](orb-run/) | **P3.7, gameplay in logic.** Orbs appear around a walled arena on a timer, the player gathers them by walking through (a Trigger spawned by logic, heard through its `entered` signal), each is removed with its glow and another placed, and a boulder is pushed at the player every few seconds. **P3.8, the game's interface.** It opens on a title screen with the world paused; Play starts it; the score is on the HUD; Pause shows a pause screen over the held world, and Resume lets it go on. **P3.9, input actions.** Its `pause` action is P, the pad's Start, or the Pause button under a finger. **P3.12, animation.** Two lanterns, an animated glTF model written by [`make_lantern.py`](orb-run/make_lantern.py) (a skinned pole whose head sways, and a core that breathes by a morph target), play their `idle` clip and flare as each orb is taken; orbs grow in and shrink away by tweens, the score swells as it counts, and the boulder is a state machine. **P3.13, sound.** The title's music gives way to the arena's when play starts, fading one into the other; a chime sounds where each orb is taken and a rumble where the boulder is rolled; and the pause screen turns the music off and on (its volume group). Its sounds are written by [`make_sounds.py`](orb-run/make_sounds.py), standard library only. **P3.14, materials.** Walls of dressed stone with their relief, a slate floor, a warm spot light over the arena and a cool one washing the north wall, under an HDR dusk sky whose sun stands where the scene's Sun shines from; all written by [`make_looks.py`](orb-run/make_looks.py), standard library only. **P3.15, groups and prefabs.** Its orbs are a prefab, [`prefabs/orb.qml`](orb-run/prefabs/orb.qml), a glow and the trigger that hears the player, placed whole by logic and taken away whole; its walls are a group | `tst_play`, `theOrbRunSampleIsAGameMadeOfKindsAndLogic` |
| [`ember-climb/`](ember-climb/) | **P3.10, 2D.** A pixel-art platformer: a level drawn in Tiled (`assets/level.tmj`, two layers: the wall behind and the rock and planks in front), a hero who runs and jumps with its look played from a sprite sheet by what it is doing, torches flickering on the wall, and eight embers spawned by logic and gathered by running through them. The eye follows the hero and stops at the level's edges; a fall into a pit puts the hero back at the start; the last ember lights the forge and shows the time the climb took, and Again starts over. Title, HUD, pause and win screens, and P or the pad's Start to pause. Its art and level are written by [`make_art.py`](ember-climb/make_art.py), standard library only. **P3.11:** its gameplay runs on steps and its hero on QQ's own kinematics, and its outcome (embers gathered, the steps the climb took) is reached again by a replay with no window | `tst_play`, `theEmberClimbSampleIsA2DGameMadeOfKindsAndLogic`; `tst_player`, `aTwoDimensionalGamePlaysToo` |

```
pwsh products/qq/build.ps1 -Run                                # QQ Studio; open samples/orb-run
<qq-build>/player/qq-player.exe samples/orb-run/scenes/orb-run.qml
```

Press Play; P pauses, and the pause screen turns the music off and on. Move with W A S D or the left stick, look with a drag or the right stick, jump with Space.

```
<qq-build>/player/qq-player.exe samples/ember-climb/scenes/ember-climb.qml
```

A run can be kept and played again with no window (P3.11): the replay exits 0 when it reaches the outcome kept.

```
<qq-build>/player/qq-player.exe samples/ember-climb/scenes/ember-climb.qml --record run.json
<qq-build>/player/qq-player.exe samples/ember-climb/scenes/ember-climb.qml --replay run.json
```

In Ember Climb, run with A and D or the left stick and jump with Space or the pad's A. To change its level, open
`ember-climb/assets/level.tmj` in [Tiled](https://www.mapeditor.org) and save it as JSON; the Studio and the Player read
it as it is.
