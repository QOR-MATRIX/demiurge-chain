# QQ samples

Games made only from QQ's kinds and logic files, with no change to the engine (ADR-105 decision 8). Each one proves a
step of DIRECTION P3, and the Qt Test suites play it. Each is a project as QQ Studio keeps one: `scenes/` and
`logic/`, opened with QQ Studio or played with the QQ Player.

| Sample | What it proves | Played by |
| --- | --- | --- |
| [`orb-run/`](orb-run/) | **P3.7, gameplay in logic.** Orbs appear around a walled arena on a timer, the player gathers them by walking through (a Trigger spawned by logic, heard through its `entered` signal), each is removed with its glow and another placed, and a boulder is pushed at the player every few seconds | `tst_play`, `theOrbRunSampleIsAGameMadeOfKindsAndLogic` |

```
pwsh products/qq/build.ps1 -Run                                # QQ Studio; open samples/orb-run
<qq-build>/player/qq-player.exe samples/orb-run/scenes/orb-run.qml
```

Move with W A S D or the left stick, look with a drag or the right stick, jump with Space.
