// What a game's logic file is: a Logic, with handlers for what happens while the world plays. A Behaviour in the scene
// names the file and the entity it drives; the file is built when play starts and again each time it is saved.
//
//   // logic/spin.qml
//   import QQ
//
//   Logic {
//       property real speed: 90          // degrees a second
//       onStep: (dt) => target.eulerRotation.y += speed * dt
//   }
//
// `target` is the entity the Behaviour names, `scene` the scene playing, `elapsed` the seconds of play since this logic
// was built. `Input` (move, look, jump, run) is there for any logic that reads the player's hands.
//
// Steps (P3.11, ADR-105 decision 6): the game moves in steps of 1/60 s, whatever the frame rate. `onStep` is where
// gameplay goes; `onFrame` comes once a drawn frame, for what is only looked at. Timers keep step time, `random()` and
// `randomInt()` draw from the scene's seed, and `overlaps()` measures with QQ's own boxes, so a run can be played again
// with no window and reach the same `outcome`: whatever a logic file sets there is what a replay must match.
//
// Gameplay (DIRECTION P3.7, ADR-105): what the target touches and what enters it, rays, making and removing entities,
// and timers, all from here, so a game is a scene and its logic and never a change to the engine.
//
//   Logic {
//       property int score: 0
//       onEntered: (other) => { if (other && other.kind === "Player") { score += 1; remove(target) } }
//       onStarted: every(2, () => spawn("Shape", { form: "sphere", body: "dynamic", position: Qt.vector3d(0, 6, 0) }))
//   }
//
// Timers run on the logic's own clock, the one `elapsed` counts in steps, so they keep time with the game rather than
// with the wall.
//
// The game's interface (P3.8): the screen kinds are entities like any other, so logic sets a ScreenText's `text` or a
// ScreenBar's `value` through entity(name); it shows and hides whole screens, and pauses the world beneath them.
//
//   onClicked: { onlyScreen("hud"); resume() }        // a Behaviour targeting the title's Play button
//
// Actions (P3.9): the scene's InputAction entities name what a player can do, with their keys, mouse buttons and pad
// buttons. Any logic hears them, whatever it targets:
//
//   onPressed: (action) => { if (action === "fire") spawn("Shape", { body: "dynamic", position: target.position }) }
//
// held(name) says whether one is held now, rebind(name) takes the player's next press as its binding (a controls
// screen), and pointing() is the entity under the pointer.
//
// Animation (P3.12): a Prop plays its glTF clips (`clip`), and any field of any entity can be eased to a new value over
// a time, on the game's steps, with a callback when it arrives:
//
//   tween(entity("Door"), "position", Qt.vector3d(0, 3, 0), 1.5, "outCubic", () => showScreen("won"))
//
// Sound (P3.13): play() sounds a file once, where something happened or everywhere alike; a scene's Music fades from one
// track to the next when its `source` changes; and every sound is in a volume group whose level logic sets, so an
// options screen's buttons turn the music down and leave the effects alone:
//
//   play("../assets/coin.wav", { at: target.position, group: "effects" })
//   setVolume("music", 0.4)
//
// A state machine keeps what something is doing (idle, chasing, stunned), with what happens on entering a state, on
// each step in it and on leaving it, how long it has been in it, and events that move it on:
//
//   readonly property var guard: machine({ start: "watch",
//       watch: { step: () => { if (sees()) guard.go("chase") } },
//       chase: { enter: () => target.clip = "run", on: { lost: "watch" } } })
//
// Saving (P3.16): one small store per game, kept from one play to the next, which the runtime writes for it:
//
//   onStarted: best = saved("best", 0)
//   function scored() { if (score > best) { best = score; save("best", best) } }

import QtQml
import QQ

QtObject {
    id: logic

    /// The entity this logic drives, and the scene it plays in.
    property QtObject target: null
    property QtObject scene: null

    /// Seconds of play since this logic was built: its steps over 60.
    property real elapsed: 0
    /// Steps since this logic was built.
    property int steps: 0

    /// What this run came to, for whatever keeps score (P3.11): a number, a string or a small object, set by the logic
    /// file. A replay of the run must reach the same.
    property var outcome

    /// Once, before the first step: where a game sets itself up (its timers, its first entities). A logic file imports
    /// only QQ, so this, not Component.onCompleted, is where it begins. Again after each rebuild.
    signal started()

    /// Every step of the game, 60 a second whatever the frame rate, with the step's length (1/60 s): where gameplay goes.
    signal step(real dt)

    /// Once a drawn frame while the world plays, with the seconds since the last: for what is only looked at.
    signal frame(real dt)

    /// The target's body touched another entity's (null for something that is not an entity), at `point`, pushed
    /// along `normal`. Once as they meet, not while they rest together.
    // Points and normals are vector3d values; `var` here, because a Logic imports only QtQml.
    signal contact(QtObject other, var point, var normal)

    /// Something came into the target, a Trigger, or left it.
    signal entered(QtObject other)
    signal exited(QtObject other)

    /// The target, a ScreenButton, was pressed.
    signal clicked()

    /// An action became held, or was let go (P3.9): one of the scene's InputActions, or jump or run.
    signal pressed(string action)
    signal released(string action)

    /// Whether an action is held now.
    function held(name) {
        return Input.held(name)
    }
    /// Take the player's next key, mouse button or pad button as `name`'s binding for that device.
    function rebind(name) {
        Input.listen(name)
    }
    /// An action's bindings as a player reads them: "E / Mouse Left / Pad X".
    function bindingOf(name) {
        return Input.describe(name)
    }
    /// The entity under the pointer, { entity, point, normal, distance }, or null.
    function pointing() {
        const view = scene ? SceneIO.viewOf(scene) : null
        if (!view)
            return null
        const at = view.mapFromItem(null, Input.pointer.x, Input.pointer.y)
        const hit = view.pick(at.x, at.y)
        const entity = hit.objectHit ? SceneIO.entityOf(scene, hit.objectHit) : null
        return entity && !removing(entity)
               ? { entity: entity, point: hit.scenePosition, normal: hit.sceneNormal, distance: hit.distance } : null
    }

    /// A number from 0 up to 1, from the scene's seed: the same seed gives the same numbers, in the same order.
    function random() {
        return scene ? scene.random.next() : 0
    }
    /// A whole number from `low` to `high`, both included, from the scene's seed.
    function randomInt(low, high) {
        return scene ? scene.random.between(low, high) : low
    }
    /// Play a sound file once (P3.13), kept relative to the scene file: `options` { at (a place in the world, heard from
    /// there; or none, heard everywhere alike), volume (1), group ("effects"), reach (25 metres) }. The voice, or null:
    /// a replay has no ears, so there it plays nothing.
    function play(source, options) {
        if (!scene || scene.replaying || !scene._voices)
            return null
        return scene._voices.oneShot(SceneIO.resolvedUrl(scene, source), options ?? {})
    }
    /// A volume group's level, from 0 to 1 ("master", "music", "effects", "voice", "ui", or any a game names), and set it.
    function volumeOf(group) {
        return Mixer.level(group)
    }
    function setVolume(group, level) {
        Mixer.setLevel(group, level)
    }

    /// What this game kept under `key` (P3.16), or `fallback` when it kept nothing there. A game has one small store,
    /// kept from one play to the next, the only one the lockdown allows: numbers, text, true and false, null, and lists
    /// and objects of them, 64 KiB in all, under keys of 1 to 100 characters. A replay plays from the store as its run
    /// found it and keeps nothing. The store is the player's, on their machine, so nothing rewarded rests on it.
    function saved(key, fallback) {
        const store = scene ? SceneIO.storeOf(scene) : null
        const value = store ? store.value(key) : undefined
        return value === undefined ? fallback : value
    }
    /// Keep `value` under `key` for the next play. False when it is not kept; the log says why.
    function save(key, value) {
        const store = scene ? SceneIO.storeOf(scene) : null
        return store ? store.setValue(key, value) : false
    }
    /// Forget what is kept under `key`; true if something was.
    function forget(key) {
        const store = scene ? SceneIO.storeOf(scene) : null
        return store ? store.remove(key) : false
    }

    /// Whether two entities' boxes overlap, as QQ measures them (never PhysX), so a replay measures the same.
    function overlaps(a, b) {
        return SceneIO.overlaps(a, b)
    }

    /// Show a screen by name, hide one, or show one and nothing else (a title to the game, the game to its end).
    function showScreen(name) {
        if (scene)
            scene.showScreen(name)
    }
    function hideScreen(name) {
        if (scene)
            scene.hideScreen(name)
    }
    function onlyScreen(name) {
        if (scene)
            scene.onlyScreen(name)
    }

    /// Hold the world still (physics, logic, the player, particles) while screens still answer; and let it go on.
    /// While paused, no logic's clock moves, so no timer fires: a pause screen's buttons resume it.
    function pause() {
        if (scene)
            scene.paused = true
    }
    function resume() {
        if (scene)
            scene.paused = false
    }
    readonly property bool paused: scene !== null && scene.paused === true

    /// The entity in this scene called `name`, or null.
    function entity(name) {
        // The scene's own first, those in groups among them; then any prefab's part (P3.15), the first of that name.
        return scene ? SceneIO.entities(scene).find(e => e.name === name)
                       ?? SceneIO.everyEntity(scene).find(e => e.name === name) ?? null : null
    }

    /// Make an entity of `kind` with `fields` while the world plays, at the scene's top or in a group (`into`): it, or
    /// null if it cannot be made. It is part of the game being played, never of the scene being edited. A prefab is
    /// placed so: spawn("Prefab", { source: "../prefabs/coin.qml", position: at }), its parts found by its part(name).
    function spawn(kind, fields, into) {
        if (!scene)
            return null
        const made = SceneIO.add(scene, kind, fields ?? {}, into ?? null)
        if (!made)
            console.warn("QQ logic: spawn(\"" + kind + "\"): " + SceneIO.lastError)
        return made
    }

    /// Take an entity out of the game, with its body. It goes once the handler that asked has returned, so a contact
    /// or an entry can remove what caused it. False if it is not an entity of this scene, or is already going.
    function remove(entity) {
        if (!entity || !scene || SceneIO.sceneOf(entity) !== scene || removing(entity))
            return false
        logic._removing = logic._removing.concat([entity])
        // In a step it goes as the step ends, so a replay removes it at the same moment; otherwise once control returns.
        if (scene._ticking) {
            scene._doom(entity)
            Qt.callLater(() => logic._removing = logic._removing.filter(e => e !== entity))
        } else {
            Qt.callLater(() => {
                logic._removing = logic._removing.filter(e => e !== entity)
                SceneIO.remove(entity)
            })
        }
        return true
    }

    /// Whether `entity` is on its way out: asked to be removed and not yet gone.
    function removing(entity) {
        return logic._removing.indexOf(entity) >= 0
    }

    /// Call `callback` once, `seconds` from now on the logic's clock (to the nearest step). Its id, for cancel().
    function after(seconds, callback) {
        return _addTimer(seconds, 0, callback)
    }

    /// Call `callback` every `seconds` on the logic's clock, the first time `seconds` from now. Its id, for cancel().
    function every(seconds, callback) {
        if (!(seconds > 0)) {
            console.warn("QQ logic: every() needs a time above zero")
            return 0
        }
        return _addTimer(seconds, seconds, callback)
    }

    /// Stop a timer that after() or every() started, or a tween.
    function cancel(id) {
        logic._timers = logic._timers.filter(t => t.id !== id)
        logic._tweens = logic._tweens.filter(t => t.id !== id)
    }

    /// Ease `entity`'s `field` from where it is to `to` over `seconds` of play (to the nearest step), with an easing:
    /// linear, inQuad, outQuad, inOutQuad, inCubic, outCubic, inOutCubic, outBack or outBounce. A number, a vector or a
    /// colour. `done` is called on the step it arrives. A new tween on the same field takes over from the old. Its id,
    /// for cancel(); 0 if the field cannot be eased.
    function tween(entity, field, to, seconds, easing, done) {
        const now = entity ? entity[field] : undefined
        const form = typeof now === "number" ? "number"
                   : now !== null && now !== undefined && now.z !== undefined && now.x !== undefined ? "vector"
                   : now !== null && now !== undefined && now.r !== undefined && now.a !== undefined ? "colour"
                   : ""
        if (form === "") {
            console.warn("QQ logic: tween(): " + (entity ? entity.name : "nothing") + " has no " + field + " to ease")
            return 0
        }
        // A copy of where it starts: a vector or a colour read from a field follows the field as it changes.
        const from = form === "vector" ? Qt.vector3d(now.x, now.y, now.z)
                   : form === "colour" ? Qt.rgba(now.r, now.g, now.b, now.a) : now
        if (form === "colour" && typeof to === "string")
            to = Qt.lighter(to, 1)  // "#ff6a00" as a colour
        const ease = logic._easings[easing ?? "linear"]
        if (!ease) {
            console.warn("QQ logic: tween(): no easing called " + easing)
            return 0
        }
        logic._tweens = logic._tweens.filter(t => !(t.entity === entity && t.field === field))
        const id = ++logic._timerCount
        logic._tweens = logic._tweens.concat([{ id: id, entity: entity, field: field, form: form, from: from, to: to,
                                                steps: Math.max(1, Math.round(seconds * 60)), at: 0, ease: ease,
                                                done: typeof done === "function" ? done : null }])
        return id
    }

    /// A state machine (see the top of this file): `spec` names the states, each with any of enter(), step(dt), exit()
    /// and `on` (events to states), and `start` the first, entered on the logic's first step (or by go() before then).
    /// It steps with this logic: { state, previous, time and steps (in the state), go(name), send(event), is(name) }.
    function machine(spec) {
        const m = {
            spec: spec,
            state: "",
            previous: "",
            time: 0,
            steps: 0,
            _leaving: false,
            is(name) { return m.state === name },
            go(name) {
                if (!m.spec[name] || m._leaving) {
                    if (!m.spec[name])
                        console.warn("QQ logic: a machine has no state called " + name)
                    return false
                }
                const from = m.spec[m.state]
                if (from && from.exit) {
                    m._leaving = true
                    try { from.exit() } finally { m._leaving = false }
                }
                m.previous = m.state
                m.state = name
                m.time = 0
                m.steps = 0
                if (m.spec[name].enter)
                    m.spec[name].enter()
                return true
            },
            send(event) {
                const here = m.spec[m.state]
                const next = here && here.on ? here.on[event] : undefined
                const name = typeof next === "function" ? next() : next
                return name ? m.go(name) : false
            }
        }
        // Added in place, not by assigning a new list: a machine is made in a property's binding, which would otherwise
        // read and write the same list and run again.
        logic._machines.push(m)
        return m
    }

    /// The first entity a ray meets, from `from` along `direction`, within `distance` metres (all of it if left out):
    /// { entity, point, normal, distance }, or null. The target itself is not met, so a ray can start inside it.
    function ray(from, direction, distance) {
        const view = scene ? SceneIO.viewOf(scene) : null
        if (!view || direction.length() === 0)
            return null
        const hits = view.rayPickAll(from, direction.normalized()).slice().sort((a, b) => a.distance - b.distance)
        for (const hit of hits) {
            if (distance > 0 && hit.distance > distance)
                break
            const entity = SceneIO.entityOf(scene, hit.objectHit)
            if (entity && entity !== target && !removing(entity))
                return { entity: entity, point: hit.scenePosition, normal: hit.sceneNormal, distance: hit.distance }
        }
        return null
    }

    /// Marks a Logic: a file whose root is anything else is refused.
    readonly property bool qqLogic: true

    default property list<QtObject> data

    // ── run by the Behaviour: a step from the scene's clock, a frame from the display ──

    property var _timers: []
    property int _timerCount: 0
    property var _tweens: []
    property var _machines: []
    readonly property var _easings: ({
        linear: t => t,
        inQuad: t => t * t,
        outQuad: t => t * (2 - t),
        inOutQuad: t => t < 0.5 ? 2 * t * t : 1 - 2 * (1 - t) * (1 - t),
        inCubic: t => t * t * t,
        outCubic: t => 1 - (1 - t) * (1 - t) * (1 - t),
        inOutCubic: t => t < 0.5 ? 4 * t * t * t : 1 - 4 * (1 - t) * (1 - t) * (1 - t),
        outBack: t => 1 + 2.70158 * (t - 1) * (t - 1) * (t - 1) + 1.70158 * (t - 1) * (t - 1),
        outBounce: t => t < 1 / 2.75 ? 7.5625 * t * t
                      : t < 2 / 2.75 ? 7.5625 * (t - 1.5 / 2.75) * (t - 1.5 / 2.75) + 0.75
                      : t < 2.5 / 2.75 ? 7.5625 * (t - 2.25 / 2.75) * (t - 2.25 / 2.75) + 0.9375
                      : 7.5625 * (t - 2.625 / 2.75) * (t - 2.625 / 2.75) + 0.984375
    })

    function _tweenStep() {
        if (logic._tweens.length === 0)
            return
        const arrived = []
        for (const t of logic._tweens.slice()) {
            if (!t.entity || !t.entity.parent) {
                logic._tweens = logic._tweens.filter(k => k !== t)  // gone from the game
                continue
            }
            t.at += 1
            const e = t.ease(Math.min(1, t.at / t.steps))
            const a = t.from
            const b = t.to
            t.entity[t.field] = t.form === "number" ? a + (b - a) * e
                              : t.form === "vector" ? Qt.vector3d(a.x + (b.x - a.x) * e, a.y + (b.y - a.y) * e,
                                                                  a.z + (b.z - a.z) * e)
                              : Qt.rgba(a.r + (b.r - a.r) * e, a.g + (b.g - a.g) * e, a.b + (b.b - a.b) * e,
                                        a.a + (b.a - a.a) * e)
            if (t.at >= t.steps) {
                logic._tweens = logic._tweens.filter(k => k !== t)
                if (t.done)
                    arrived.push(t.done)
            }
        }
        for (const done of arrived) {
            try {
                done()
            } catch (error) {
                console.error("QQ logic: a tween's callback failed: " + error)
            }
        }
    }

    function _machineStep(dt) {
        for (const m of logic._machines) {
            if (m.state === "") {
                if (m.spec.start)
                    m.go(m.spec.start)
                continue
            }
            m.time += dt
            m.steps += 1
            const here = m.spec[m.state]
            if (here && here.step) {
                try {
                    here.step(dt)
                } catch (error) {
                    console.error("QQ logic: a machine's step failed: " + error)
                }
            }
        }
    }
    property var _removing: []
    property bool _begun: false

    function _addTimer(seconds, repeat, callback) {
        if (typeof callback !== "function") {
            console.warn("QQ logic: a timer needs a function to call")
            return 0
        }
        // In whole steps, so a timer falls on the same step in play and in a replay.
        const id = ++logic._timerCount
        logic._timers = logic._timers.concat([{ id: id, due: logic.steps + Math.max(0, Math.round(seconds * 60)),
                                                repeat: repeat > 0 ? Math.max(1, Math.round(repeat * 60)) : 0,
                                                callback: callback }])
        return id
    }

    /// One step of the game: the clock moves, every timer that is due runs, in the order they fall due, then `step`. A
    /// repeating timer fires at most once a step and catches up over the next ones. A timer that throws is reported, and
    /// the rest still run.
    function _step(dt) {
        if (!logic._begun) {
            logic._begun = true
            started()
        }
        logic.steps += 1
        logic.elapsed = logic.steps / 60
        const now = logic.steps
        const due = logic._timers.filter(t => t.due <= now).sort((a, b) => a.due - b.due || a.id - b.id)
        for (const t of due) {
            if (!logic._timers.some(k => k.id === t.id))
                continue  // cancelled by a timer that ran before it
            if (t.repeat > 0)
                t.due += t.repeat
            else
                cancel(t.id)
            try {
                t.callback()
            } catch (error) {
                console.error("QQ logic: a timer failed: " + error)
            }
        }
        _tweenStep()
        _machineStep(dt)
        step(dt)
    }

    /// A drawn frame, once the logic has begun.
    function _frame(dt) {
        if (logic._begun)
            frame(dt)
    }

    /// A press or release, from the scene's tick.
    function _hear(kind, action) {
        if (kind === "pressed")
            pressed(action)
        else
            released(action)
    }
}
