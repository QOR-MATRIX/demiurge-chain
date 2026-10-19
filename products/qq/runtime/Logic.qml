// What a game's logic file is: a Logic, with handlers for what happens while the world plays. A Behaviour in the scene
// names the file and the entity it drives; the file is built when play starts and again each time it is saved.
//
//   // logic/spin.qml
//   import QQ
//
//   Logic {
//       property real speed: 90          // degrees a second
//       onFrame: (dt) => target.eulerRotation.y += speed * dt
//   }
//
// `target` is the entity the Behaviour names, `scene` the scene playing, `elapsed` the seconds since this logic was
// built. `Input` (move, look, jump, run) is there for any logic that reads the player's hands.
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
// Timers run on the logic's own clock, the one `elapsed` counts and `frame` is given, so they keep time with the game
// rather than with the wall.
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

import QtQml
import QQ

QtObject {
    id: logic

    /// The entity this logic drives, and the scene it plays in.
    property QtObject target: null
    property QtObject scene: null

    /// Seconds since this logic was built.
    property real elapsed: 0

    /// Once, before the first frame: where a game sets itself up (its timers, its first entities). A logic file imports
    /// only QQ, so this, not Component.onCompleted, is where it begins. Again after each rebuild.
    signal started()

    /// Once a frame while the world plays, with the seconds since the last frame.
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

    property Connections _actions: Connections {
        target: Input
        function onPressed(action) { logic.pressed(action) }
        function onReleased(action) { logic.released(action) }
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
        return scene ? SceneIO.entities(scene).find(e => e.name === name) ?? null : null
    }

    /// Make an entity of `kind` with `fields` while the world plays: it, or null if it cannot be made. It is part of
    /// the game being played, never of the scene being edited.
    function spawn(kind, fields) {
        if (!scene)
            return null
        const made = SceneIO.add(scene, kind, fields ?? {})
        if (!made)
            console.warn("QQ logic: spawn(\"" + kind + "\"): " + SceneIO.lastError)
        return made
    }

    /// Take an entity out of the game, with its body. It goes once the handler that asked has returned, so a contact
    /// or an entry can remove what caused it. False if it is not an entity of this scene, or is already going.
    function remove(entity) {
        if (!entity || !scene || entity.parent !== scene || removing(entity))
            return false
        logic._removing = logic._removing.concat([entity])
        Qt.callLater(() => {
            logic._removing = logic._removing.filter(e => e !== entity)
            SceneIO.remove(entity)
        })
        return true
    }

    /// Whether `entity` is on its way out: asked to be removed and not yet gone.
    function removing(entity) {
        return logic._removing.indexOf(entity) >= 0
    }

    /// Call `callback` once, `seconds` from now on the logic's clock. Its id, for cancel().
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

    /// Stop a timer that after() or every() started.
    function cancel(id) {
        logic._timers = logic._timers.filter(t => t.id !== id)
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

    // ── run by the Behaviour, once a frame ──

    property var _timers: []
    property int _timerCount: 0
    property var _removing: []
    property bool _begun: false

    function _addTimer(seconds, repeat, callback) {
        if (typeof callback !== "function") {
            console.warn("QQ logic: a timer needs a function to call")
            return 0
        }
        const id = ++logic._timerCount
        logic._timers = logic._timers.concat([{ id: id, due: logic.elapsed + Math.max(0, seconds), repeat: repeat,
                                                callback: callback }])
        return id
    }

    /// Advance the clock by `dt`, call every timer that is due, in the order they fall due, then `frame`. A repeating
    /// timer fires at most once a step and catches up over the next ones. A timer that throws is reported, and the rest
    /// still run.
    function _step(dt) {
        if (!logic._begun) {
            logic._begun = true
            started()
        }
        logic.elapsed += dt
        const now = logic.elapsed
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
        frame(dt)
    }
}
