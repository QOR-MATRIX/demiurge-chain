// QQ Studio's window (DIRECTION P3.1): the scene's entities on the left, the world in the middle with handles on the
// selection, the selected entity's fields on the right, and the project and the scene's file across the top.
//
// The Studio holds one scene at a time, built and written by SceneIO; every edit goes to an entity's properties, and
// the scene is unsaved exactly while its canonical text differs from what was last saved. Saving writes
// scenes/<name>.qml in the project and copies any model from outside the project into its assets folder first, so a
// project is whole on its own and a commit in Projects carries everything the scene needs.
//
// The viewport: click to select, drag a handle to move along its axis, turn about the vertical or scale evenly, drag
// anywhere else to orbit, scroll to zoom, F to frame the selection, Delete to remove it, Ctrl+S to save.
//
// Editing (P3.15): Ctrl+Z undoes an edit and Ctrl+Y (or Ctrl+Shift+Z) does it again, whatever made it (History.qml);
// Ctrl+D duplicates the selection beside itself; and with snapping on, a handle moves along a grid, turns by 15 degrees
// and scales by tenths. The list is the scene's tree: Ctrl+G puts the selection in a new group, Ctrl+Shift+G takes a
// group's contents out of it, the inspector moves the selection between groups, and a group can be kept as a prefab,
// prefabs/<name>.qml, placed as often as wanted. A click in the world picks what is innermost, but a prefab's parts
// pick the prefab. The project's files are listed under the scene, each used by a click (ProjectPanel.qml); a logic
// file opens in the editor under the world (LogicEditor.qml); and F3 shows what the world costs, with every body drawn
// as its collider while it plays (StatsOverlay.qml).
//
// Play (F5) runs the scene (DIRECTION P3.2): a copy is built from the scene's canonical text and played in its place,
// with physics, the player under the keyboard, mouse and gamepad, particles, sound and logic that reloads when its file
// is saved. Stop (Esc) discards the copy, so the scene comes back exactly as it was; nothing play does can reach it.

import QtQuick
// Before the Material style: Qt Quick 3D has a type called Material too, and the import that comes later wins.
import QtQuick3D
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Dialogs
import QtQuick.Layouts
import QQ

ApplicationWindow {
    id: window

    /// A glTF model for a new scene, and the project folder, both given on the command line (or chosen here).
    required property url model
    required property url project

    width: 1600
    height: 960
    visible: true
    title: (dirty ? "● " : "") + sceneName + " — QQ Studio"
    color: Look.deepest

    // The tool register of Demiurge's one look (ADR-108): every colour, face and corner from Look. Qt's Material style
    // draws the fields, lists and menus in it; the actions are StudioButtons.
    Material.theme: Material.Dark
    Material.accent: Look.accent
    Material.primary: Look.surface
    Material.background: Look.base
    Material.foreground: Look.inkBody
    Material.roundedScale: Material.SmallScale
    Material.containerStyle: Material.Outlined
    font.family: Look.body

    property QtObject scene: null
    property QtObject selection: null
    property url sceneFile: ""
    property string sceneName: "untitled"
    property string savedText: ""
    property bool dirty: false
    property var entityList: []
    property var projectInfo: ({ name: "", repository: false, scenes: [] })
    property string notice: ""
    property bool noticeIsProblem: false

    /// The copy being played, or null while editing.
    property QtObject playScene: null
    readonly property bool playing: playScene !== null
    /// The player the camera follows in play, if the scene has one.
    readonly property QtObject follow: playScene ? playScene.player : null
    /// The first problem any logic has while playing: a file that does not build, or cannot be read.
    readonly property string playProblem: playScene
                                          ? SceneIO.entities(playScene).map(e => e.error ?? "").find(x => x !== "") ?? ""
                                          : ""

    /// The design loop (agent/designloop.h), set by main.cpp: a model that designs and builds from a description.
    property QtObject designer: null
    /// Publishing, through the launcher that started this Studio (launcherlink.h, ADR-088): null in a test window.
    property QtObject launcher: null
    /// Publish can be pressed whenever the launcher is there to ask; what else is missing is said when it is pressed,
    /// in the notice, rather than left in a tooltip on a grey button.
    readonly property bool canPublish: launcher !== null && launcher.available && !launcher.busy && !playing
                                       && scene !== null
    readonly property string publishHint: launcher === null ? ""
                                        : !launcher.available ? launcher.reason
                                        : !projectInfo.repository ? qsTr("Open a versioned project to publish it.")
                                        : qsTr("Commit the project and publish it as its Cartridge. The launcher asks "
                                               + "you first, in its own window.")
    /// Whether the Agent panel is open.
    property bool agentOpen: false

    /// Undo and redo (History.qml): the scene's canonical text after each edit.
    readonly property QtObject history: History {}
    /// What the next refresh() records as: an edit's key, so quick edits of one field are one step ("" for none).
    property string editKey: ""
    /// The statistics over the world, and the colliders drawn in play (F3).
    property bool showStats: false

    /// Snapping (P3.15): the grid a handle moves along, in units; 0 is off. A handle then turns by `snapAngle` degrees
    /// and scales by tenths.
    property real snapStep: 0
    readonly property real snapAngle: 15

    /// True once a scene is shown and every model in it has loaded or failed: what a capture waits for.
    readonly property bool settled: scene !== null
                                    && entityList.every(e => e.kind !== "Prop" || e.status !== Prop.Empty)

    // The eye on a sphere around its target: yaw and pitch in degrees, and distance.
    property real yaw: 25
    property real pitch: 16
    property real distance: 8
    property vector3d target: Qt.vector3d(0, 1, 0)
    readonly property vector3d orbitEye: Qt.vector3d(
        target.x + distance * Math.cos(pitch * Math.PI / 180) * Math.sin(yaw * Math.PI / 180),
        target.y + distance * Math.sin(pitch * Math.PI / 180),
        target.z + distance * Math.cos(pitch * Math.PI / 180) * Math.cos(yaw * Math.PI / 180))

    // ── the scene ──────────────────────────────────────────────────────────────────────────────

    function say(text, problem) {
        notice = text
        noticeIsProblem = !!problem
    }

    /// After every edit: the entity list and the unsaved state brought up to date, and the edit kept for undo.
    function refresh() {
        entityList = scene ? SceneIO.entities(scene) : []
        const key = editKey
        editKey = ""
        if (!scene) {
            dirty = false
            return
        }
        const text = SceneIO.write(scene, sceneFile)
        dirty = text !== savedText
        history.note(text, key)
    }

    /// Undo and redo: the scene put back to the text before or after, its file, saved state, view and selection kept.
    function undo() { return travel(history.undo()) }
    function redo() { return travel(history.redo()) }
    function travel(text) {
        if (playing || !scene || text === "")
            return false
        const keepSaved = savedText
        const keepDistance = distance
        const keepTarget = target
        const chosen = selection === scene ? scene : selection ? selection.name : ""
        if (!show(SceneIO.loadText(text, sceneFile, stage), sceneFile, sceneName, false))
            return false
        savedText = keepSaved
        distance = keepDistance
        target = keepTarget
        refresh()
        selection = chosen === "" ? null : chosen === scene ? scene : entityList.find(e => e.name === chosen) ?? null
        return true
    }

    /// A copy of the selection beside it, every field the same but its name, and selected.
    function duplicateSelection() {
        if (playing || !selection || selection === scene)
            return null
        if (selection.kind === "Player" || selection.kind === "Player2D") {
            say(qsTr("A scene has one player."), true)
            return null
        }
        const copy = {}
        for (const field of selection.fields) {
            const value = selection[field]
            const type = SceneIO.fieldType(selection, field)
            // Read values are live references to the original's: copied out, so the two stay apart.
            copy[field] = type === "vector3d" ? Qt.vector3d(value.x, value.y, value.z)
                        : type === "color" ? String(value)
                        : type === "url" ? (String(value) === "" ? "" : SceneIO.resolvedUrl(selection, value))
                        : value
        }
        copy.name = String(selection.name).replace(/ \d+$/, "")
        if (copy.position !== undefined)
            copy.position = copy.position.plus(Qt.vector3d(1, 0, 0))
        const made = add(selection.kind, copy)
        if (made)
            say(qsTr("Duplicated as %1.").arg(made.name))
        return made
    }

    /// A value on the snapping grid (or as it is, with snapping off).
    function snapped(v) {
        return snapStep > 0 ? Math.round(v / snapStep) * snapStep : v
    }

    function fileFor(name) {
        return String(project) === "" ? "" : String(project) + "/scenes/" + name + ".qml"
    }

    function show(next, file, name, saved) {
        if (!next) {
            say(SceneIO.lastError, true)
            return false
        }
        if (scene)
            SceneIO.discard(scene)
        scene = next
        selection = null
        sceneFile = file
        sceneName = name
        savedText = saved ? SceneIO.write(next, file) : ""
        // A 2D scene opens framed as its game starts: its view's height, the origin at its foot.
        if (next.view === "2d") {
            distance = Math.max(1.5, Math.min(80, next.viewHeight))
            target = Qt.vector3d(0, next.viewHeight / 2, 0)
        }
        refresh()
        return true
    }

    function newScene() {
        const text = 'import QQ\nScene {\n    name: "untitled"\n    Sun { elevation: 38 }\n    Ground {}\n'
                   + '    Shape { name: "Plinth"; position: Qt.vector3d(0, 0.25, 0); scale: Qt.vector3d(2, 0.5, 2); '
                   + 'colour: "#2b3140"; roughness: 0.6 }\n'
                   + '    Prop { name: "Orb"; position: Qt.vector3d(0, 1.75, 0); source: ' + JSON.stringify(String(model))
                   + ' }\n}\n'
        if (show(SceneIO.loadText(text, fileFor("untitled"), stage), fileFor("untitled"), "untitled", false)) {
            history.reset(SceneIO.write(scene, sceneFile))
            say(qsTr("A new scene. Save it to keep it in the project."))
        }
    }

    function openScene(name) {
        if (show(SceneIO.load(fileFor(name), stage), fileFor(name), name, true)) {
            history.reset(SceneIO.write(scene, sceneFile))
            say(qsTr("Opened scenes/%1.qml").arg(name))
        }
    }

    function chooseProject(folder) {
        project = folder
        projectInfo = SceneIO.project(folder)
        if (scene && String(sceneFile) === "")
            sceneFile = fileFor(sceneName)
        refresh()
    }

    function saveScene() {
        if (!scene || playing)
            return false
        if (String(project) === "") {
            projectChooser.thenSave = true
            projectChooser.open()
            return false
        }
        const name = sceneName.trim().toLowerCase().replace(/[^a-z0-9-]+/g, "-").replace(/^-+|-+$/g, "") || "untitled"
        // Files from outside the project (models, sounds, pictures, skies, prefabs, logic) come into it first, so the
        // project is whole on its own: the scene's own (its sky) and its entities', those in groups too.
        if (!adoptFiles([scene].concat(SceneIO.entities(scene))))
            return false
        scene.name = name
        const file = fileFor(name)
        const problem = SceneIO.save(scene, file)
        if (problem !== "") {
            say(problem, true)
            return false
        }
        sceneName = name
        sceneFile = file
        savedText = SceneIO.write(scene, file)
        // What saving changed (the name, files brought into the project) is part of the step it saved, not one to undo.
        history.amend(savedText)
        projectInfo = SceneIO.project(project)
        refresh()
        say(projectInfo.repository ? qsTr("Saved scenes/%1.qml. Commit it in Projects.").arg(name)
                                   : qsTr("Saved scenes/%1.qml. This folder is not versioned yet: open it in Projects.").arg(name))
        return true
    }

    function importScene(file) {
        const result = SceneIO.importQqJson(file)
        if (result.error !== "") {
            say(result.error, true)
            return
        }
        const name = String(file).split("/").pop().replace(/\.qq\.json$/, "").toLowerCase()
        if (!show(SceneIO.loadText(result.qml, fileFor(name), stage), fileFor(name), name, false))
            return
        history.reset(SceneIO.write(scene, sceneFile))
        say(qsTr("Imported %1 entities.").arg(result.imported)
            + (result.skipped.length ? " " + qsTr("Not brought in: %1.").arg(result.skipped.join("; ")) : ""))
    }

    function add(kind, properties) {
        return addInto(kind, properties, null)
    }

    /// Add an entity of `kind` in a group (`into`), or at the scene's top (null): it, selected, or null.
    function addInto(kind, properties, into) {
        if (!scene)
            return null
        let n = 1
        const base = properties.name ?? kind
        let name = base
        while (entityList.some(e => e.name === name))
            name = base + " " + (++n)
        properties.name = name
        if (kind !== "Sun" && kind !== "Ground" && kind !== "Behaviour" && kind !== "Music" && properties.position === undefined)
            properties.position = Qt.vector3d(window.target.x, Math.max(0.5, window.target.y), window.target.z)
        const made = SceneIO.add(scene, kind, properties, into)
        if (!made) {
            say(SceneIO.lastError, true)
            return null
        }
        selection = made
        refresh()
        return made
    }

    // ── the project's files (P3.15) ──

    /// What a click on one of the project's files does (ProjectPanel.qml).
    function useHint(f) {
        switch (f.kind) {
        case "scene": return qsTr("Open it")
        case "prefab": return qsTr("Place one")
        case "logic": return qsTr("Open it in the logic editor")
        case "model": return qsTr("Add it as a model")
        case "sound": return qsTr("Add it as a sound")
        case "sky": return qsTr("Make it the scene's sky")
        case "map": return qsTr("Add it as a tile map")
        case "picture": return qsTr("Put it on the selection, or add it as a sprite in a 2D scene")
        }
        return ""
    }

    /// Use one of the project's files, as its kind says (useHint).
    function useFile(f) {
        if (playing || !scene)
            return null
        const base = String(f.name).split("/").pop().replace(/\.[a-z0-9]+$/i, "")
        const here = scene.view === "2d" ? Qt.vector3d(target.x, target.y, 0) : undefined
        switch (f.kind) {
        case "scene":
            openScene(f.name)
            return scene
        case "prefab":
            return add("Prefab", { name: base, source: f.url })
        case "logic":
            return openLogic(f.name) ? f : null
        case "model":
            return add("Prop", { name: base, source: f.url })
        case "sound":
            return add("Sound", { name: base, source: f.url })
        case "map":
            return add("TileMap", { name: base, source: f.url, position: here ?? Qt.vector3d(0, 0, 0) })
        case "sky":
            scene.sky = f.url
            refresh()
            say(qsTr("%1 is the scene's sky.").arg(f.name))
            return scene
        case "picture":
            if (selection && selection !== scene && selection.fields.indexOf("image") >= 0) {
                selection.image = f.url
                refresh()
                say(qsTr("%1 is on %2.").arg(f.name).arg(selection.name))
                return selection
            }
            if (scene.view === "2d")
                return add("Sprite", { name: base, source: f.url, position: here })
            say(qsTr("Select a shape or the ground to put %1 on it.").arg(f.name), true)
            return null
        }
        return null
    }

    /// Open logic/<name>.qml in the editor under the world.
    function openLogic(name) {
        if (String(project) === "")
            return false
        return logicEditor.open(name)
    }

    /// The logic file a Behaviour runs, by name, if it is in the project's logic/; or "".
    function logicNameOf(behaviour) {
        const url = String(SceneIO.resolvedUrl(behaviour, behaviour.source))
        const folder = String(project) + "/logic/"
        return url.startsWith(folder) ? url.slice(folder.length).replace(/\.qml$/i, "") : ""
    }

    // ── the scene's tree (P3.15) ──

    /// How many groups (or prefabs) an entity is in.
    function depthOf(e) {
        let d = 0
        for (let p = e.parent; p && p !== scene; p = p.parent)
            d++
        return d
    }

    /// The groups the selection could go into: every group but itself and what is in it.
    function groupsFor(e) {
        return entityList.filter(g => g.kind === "Group" && g !== e && !SceneIO.entities(e).includes(g))
    }

    /// Put the selection in a new group, where it stands (Ctrl+G).
    function groupSelection() {
        if (playing || !selection || selection === scene)
            return null
        const chosen = selection
        if (!SceneIO.nestable(chosen.kind)) {
            say(qsTr("A %1 cannot be in a group.").arg(chosen.kind), true)
            return null
        }
        const at = chosen.position
        const group = addInto("Group", { name: "Group", position: Qt.vector3d(at.x, at.y, at.z) },
                              chosen.parent === scene ? null : chosen.parent)
        if (!group)
            return null
        SceneIO.reparent(chosen, group)
        selection = group
        refresh()
        say(qsTr("%1 is in a new group.").arg(chosen.name))
        return group
    }

    /// Take what is in the selected group out of it, where it stands, and the group away (Ctrl+Shift+G).
    function ungroupSelection() {
        if (playing || !selection || selection.kind !== "Group")
            return false
        const group = selection
        for (const e of SceneIO.entities(group).filter(e => e.parent === group))
            SceneIO.reparent(e, group.parent)
        selection = null
        SceneIO.remove(group)
        Qt.callLater(refresh)
        return true
    }

    /// Move the selection into `group`, or out to the scene's top (null), where it stands.
    function moveSelectionInto(group) {
        if (playing || !selection || selection === scene)
            return false
        if (!SceneIO.reparent(selection, group ?? scene)) {
            say(SceneIO.lastError, true)
            return false
        }
        refresh()
        return true
    }

    /// Bring every file the entities name in from outside the project, as saving does; false (said) if one cannot be.
    function adoptFiles(entities) {
        for (const e of entities) {
            for (const field of e.fields) {
                if (SceneIO.fieldType(e, field) !== "url" || String(e[field]) === "")
                    continue
                const adopted = SceneIO.adopt(SceneIO.resolvedUrl(e, e[field]), project)
                if (String(adopted) === "") {
                    say(SceneIO.lastError, true)
                    return false
                }
                e[field] = adopted
            }
        }
        return true
    }

    /// Turn the selected prefab back into a group of its parts, where it stood, to change them; Keep as prefab then
    /// writes the group back over the same file.
    function unpackPrefab() {
        if (playing || !selection || selection.kind !== "Prefab" || !selection.content)
            return null
        const prefab = selection
        const parts = prefab.content
        const into = prefab.parent
        const copy = v => Qt.vector3d(v.x, v.y, v.z)
        const turn = copy(prefab.eulerRotation)
        const size = copy(prefab.scale)
        const name = prefab.name
        parts.fromPrefab = SceneIO.resolvedUrl(prefab, prefab.source)
        if (!SceneIO.reparent(parts, into)) {
            say(SceneIO.lastError, true)
            return null
        }
        parts.eulerRotation = turn
        parts.scale = size
        prefab.content = null
        selection = null
        SceneIO.remove(prefab)
        parts.name = name
        selection = parts
        Qt.callLater(refresh)
        say(qsTr("Unpacked: change it, then Keep as prefab writes it back to its file."))
        return parts
    }

    /// Keep the selected group as a prefab, prefabs/<name>.qml, and put that prefab where the group was. A group
    /// unpacked from a prefab is written back over that file, and every prefab of it in the scene is made again.
    function makePrefab() {
        if (playing || !selection || selection.kind !== "Group")
            return null
        if (String(project) === "") {
            say(qsTr("A prefab lives in the project: choose a project first."), true)
            return null
        }
        const group = selection
        if (!adoptFiles([group].concat(SceneIO.entities(group))))
            return null
        const back = String(group.fromPrefab) !== ""
        const file = back ? SceneIO.writePrefab(group, project, String(group.fromPrefab).split("/").pop().replace(/\.qml$/i, ""), true)
                          : SceneIO.writePrefab(group, project, group.name)
        if (String(file) === "") {
            say(SceneIO.lastError, true)
            return null
        }
        const copy = v => Qt.vector3d(v.x, v.y, v.z)
        const fields = { name: group.name, source: file, position: copy(group.position),
                         eulerRotation: copy(group.eulerRotation), scale: copy(group.scale) }
        const into = group.parent === scene ? null : group.parent
        selection = null
        SceneIO.remove(group)
        entityList = SceneIO.entities(scene)
        const placed = addInto("Prefab", fields, into)
        // The others of that file are made again from it.
        for (const e of SceneIO.entities(scene))
            if (e !== placed && e.kind === "Prefab" && String(SceneIO.resolvedUrl(e, e.source)) === String(file))
                e._build()
        if (placed)
            say(qsTr("Kept as prefabs/%1; this is a prefab of it now.").arg(String(file).split("/").pop()))
        projectInfo = SceneIO.project(project)
        return placed
    }

    function play() {
        if (!scene || playing)
            return false
        world.hearing = true  // before any sound is made, so the engine is in metres from the first
        const copy = SceneIO.play(scene, sceneFile, playStage)
        if (!copy) {
            world.hearing = false
            say(SceneIO.lastError, true)
            return false
        }
        playScene = copy
        viewport.forceActiveFocus()  // out of any text field, so the keys reach the game
        say(qsTr("Playing. Esc stops, and the scene comes back as it was."))
        return true
    }

    function stop() {
        if (!playing)
            return false
        SceneIO.discard(playScene)
        playScene = null
        world.hearing = false
        Input.release()
        Mixer.reset()  // the levels the game set were the play's
        refresh()
        say(qsTr("Stopped. The scene is as it was before play."))
        return true
    }

    /// Put everything away in a safe order, before the window goes: what the application does as it quits.
    function release() {
        if (designer)
            designer.stop()
        stop()
        if (scene)
            SceneIO.discard(scene)
        scene = null
        selection = null
    }

    /// Put the scene back to `text` (the agent's run taken back), keeping what was last saved: unsaved again only if
    /// it differs from that.
    function restoreText(text) {
        if (playing)
            stop()
        const keep = savedText
        if (!show(SceneIO.loadText(text, sceneFile, stage), sceneFile, sceneName, false))
            return false
        savedText = keep
        refresh()
        say(qsTr("The agent's run is taken back."))
        return true
    }

    /// Start a new behaviour: a logic file in the project from a template, driving the selection.
    function addBehaviour() {
        if (String(project) === "") {
            say(qsTr("A behaviour's logic lives in the project: choose a project first."), true)
            return null
        }
        const driven = selection && selection !== scene ? selection.name : ""
        let n = 1
        let name = "behaviour"
        const taken = SceneIO.project(project).logic  // never over a file already there
        while (taken.indexOf(name) >= 0)
            name = "behaviour-" + (++n)
        const file = SceneIO.writeLogic(project, name, logicTemplate)
        if (String(file) === "") {
            say(SceneIO.lastError, true)
            return null
        }
        const made = add("Behaviour", { name: driven !== "" ? driven + " logic" : "Logic", source: file, target: driven })
        if (made)
            say(qsTr("Wrote logic/%1.qml. Edit it while the scene plays; each save is picked up.").arg(name))
        return made
    }

    readonly property string logicTemplate: '// Game logic: runs while the scene plays, and is rebuilt each time this file is saved.\n'
        + '// `target` is the entity this drives, `scene` the scene, `elapsed` the seconds of play since it was built;\n'
        + '// `Input` has move, look, jump and run. onStep comes 60 times a second, whatever the frame rate.\n\n'
        + 'import QQ\n\nLogic {\n    property real speed: 90  // degrees a second\n\n'
        + '    onStep: (dt) => {\n        if (target)\n            target.eulerRotation.y += speed * dt\n    }\n}\n'

    /// Save the scene, then ask the launcher to commit the project and publish it as its Cartridge (P3.4).
    function publish() {
        if (!canPublish)
            return
        if (String(project) === "") {
            say(qsTr("Choose a project to publish: Project, then a folder the launcher's Projects versions."), true)
            return
        }
        if (!projectInfo.repository) {
            say(qsTr("%1 is not versioned yet: open it in the launcher's Projects first, then publish.")
                .arg(projectInfo.name), true)
            return
        }
        if (dirty && !saveScene())
            return
        say(qsTr("Publishing %1… approve it in the launcher's window.").arg(projectInfo.name))
        launcher.publish(project, qsTr("Publish %1").arg(sceneName))
    }

    function removeSelection() {
        if (playing || !selection || selection === scene)
            return
        SceneIO.remove(selection)
        selection = null
        Qt.callLater(refresh)
    }

    function frameSelection() {
        if (selection && selection !== scene && selection.scenePosition !== undefined)
            target = selection.scenePosition
    }

    /// The entity a picked model belongs to: the innermost that declares a kind, other than the scene; but a prefab's
    /// parts are its file's, so a click on one picks the prefab (the outermost, if prefabs are in prefabs).
    function entityOf(object) {
        let found = null
        for (let o = object; o && o.kind !== "Scene"; o = o.parent) {
            if (o.kind === undefined)
                continue
            if (o.kind === "Prefab")
                found = o
            else if (!found)
                found = o
        }
        return found
    }

    Component.onCompleted: {
        if (String(project) !== "")
            chooseProject(project)
        newScene()
    }

    // Ctrl+S saves what has the keys: the logic open in the editor, or the scene.
    Shortcut { sequences: [StandardKey.Save]; onActivated: logicEditor.typing ? logicEditor.save() : window.saveScene() }
    Shortcut { sequence: "F3"; onActivated: window.showStats = !window.showStats }
    Shortcut { sequences: [StandardKey.Delete]; enabled: !window.playing; onActivated: window.removeSelection() }
    Shortcut { sequences: [StandardKey.Undo]; enabled: !window.playing; onActivated: window.undo() }
    // The platform's redo keys: Ctrl+Y and Ctrl+Shift+Z on Windows. Naming one again would make it ambiguous, and an
    // ambiguous shortcut does nothing.
    Shortcut { sequences: [StandardKey.Redo]; enabled: !window.playing; onActivated: window.redo() }
    Shortcut { sequence: "Ctrl+D"; enabled: !window.playing; onActivated: window.duplicateSelection() }
    Shortcut { sequence: "Ctrl+G"; enabled: !window.playing; onActivated: window.groupSelection() }
    Shortcut { sequence: "Ctrl+Shift+G"; enabled: !window.playing; onActivated: window.ungroupSelection() }
    Shortcut { sequence: "F"; enabled: !window.playing; onActivated: window.frameSelection() }
    Shortcut { sequence: "F5"; onActivated: window.playing ? window.stop() : window.play() }
    Shortcut { sequence: "Escape"; enabled: window.playing; onActivated: window.stop() }

    // ── the window ─────────────────────────────────────────────────────────────────────────────

    header: ToolBar {
        Material.background: Look.deepest
        height: 56
        // A gilt hairline under the bar.
        Rectangle {
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
            height: 1
            color: Qt.rgba(Look.gilt.r, Look.gilt.g, Look.gilt.b, 0.22)
        }
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 12
            spacing: 12

            // The wordmark: the one place the tool register sets Cinzel.
            Label {
                text: "QQ"
                font.family: Look.display
                font.pixelSize: 22
                font.weight: Font.DemiBold
                font.letterSpacing: 22 * Look.trackingHeading
                color: Look.giltBright
            }
            Label {
                text: window.projectInfo.name !== "" ? window.projectInfo.name : qsTr("No project")
                Layout.maximumWidth: 200
                elide: Text.ElideRight
                font.family: Look.ui
                font.weight: Font.Medium
                font.letterSpacing: Look.sizeTitle * Look.trackingHeading
                font.pixelSize: Look.sizeTitle
                color: Look.inkMuted
            }
            Label { text: "/"; color: Look.inkFaint }
            TextField {
                Layout.preferredWidth: 200
                implicitHeight: 38
                topInset: 0
                bottomInset: 0
                topPadding: 8
                bottomPadding: 8
                text: window.sceneName
                Accessible.name: qsTr("Scene name")
                onEditingFinished: window.sceneName = text
            }
            Label {
                visible: window.dirty
                text: qsTr("Unsaved")
                color: Material.accent
                font.pixelSize: 12
            }
            Item { Layout.fillWidth: true }
            // Two lines, and the whole of it on hover: a refused publish once ended at its transaction hash, before the
            // reason (15 October 2026).
            Label {
                id: noticeLabel
                objectName: "noticeLabel"
                // It gives way first when the window is narrow, so every action stays in the window.
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                Layout.maximumWidth: 560
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                text: window.playProblem !== "" ? window.playProblem : window.notice
                color: window.noticeIsProblem || window.playProblem !== "" ? Look.bad : Look.inkBody
                opacity: window.noticeIsProblem || window.playProblem !== "" ? 1 : 0.7
                font.pixelSize: 13
                HoverHandler { id: noticeHover }
                ToolTip {
                    objectName: "noticeTip"
                    visible: noticeHover.hovered && noticeLabel.truncated
                    text: noticeLabel.text
                    width: Math.min(implicitWidth, 560)
                }
            }
            StudioButton {
                objectName: "agentButton"
                visible: window.designer !== null
                text: qsTr("Agent")
                checkable: true
                checked: window.agentOpen
                onToggled: window.agentOpen = checked
            }
            StudioButton {
                objectName: "undoButton"
                text: qsTr("Undo")
                enabled: !window.playing && window.history.canUndo
                ToolTip.visible: hovered
                ToolTip.text: qsTr("Undo (Ctrl+Z)")
                onClicked: window.undo()
            }
            StudioButton {
                objectName: "redoButton"
                text: qsTr("Redo")
                enabled: !window.playing && window.history.canRedo
                ToolTip.visible: hovered
                ToolTip.text: qsTr("Redo (Ctrl+Y)")
                onClicked: window.redo()
            }
            ComboBox {
                objectName: "snapChoice"
                flat: true
                enabled: !window.playing
                Layout.preferredWidth: 128
                implicitHeight: 38
                topInset: 0
                bottomInset: 0
                font.pixelSize: Look.sizeBody
                Accessible.name: qsTr("Snapping")
                // One art pixel is a unit over the scene's pixels to a unit.
                readonly property var steps: [0, window.scene ? 1 / window.scene.pixelsPerUnit : 0.0625, 0.1, 0.25, 0.5, 1]
                model: [qsTr("Snap off"), qsTr("Snap 1 px"), qsTr("Snap 0.1"), qsTr("Snap 0.25"), qsTr("Snap 0.5"),
                        qsTr("Snap 1")]
                onActivated: (i) => window.snapStep = steps[i]
                ToolTip.visible: hovered
                ToolTip.text: qsTr("Handles move along this grid, turn by 15 degrees and scale by tenths")
            }
            StudioButton {
                objectName: "statsButton"
                text: qsTr("Stats")
                checkable: true
                checked: window.showStats
                onToggled: window.showStats = checked
                ToolTip.visible: hovered
                ToolTip.text: qsTr("What the world costs, and colliders in play (F3)")
            }
            StudioButton { text: qsTr("New"); enabled: !window.playing; onClicked: window.newScene() }
            StudioButton { text: qsTr("Import 2D…"); enabled: !window.playing; onClicked: importChooser.open() }
            // Play is the one primary action; unsaved work lights Save's ember edge rather than making a second one.
            StudioButton {
                objectName: "saveButton"
                kind: "ghost"
                text: qsTr("Save")
                enabled: !window.playing
                highlighted: window.dirty
                onClicked: window.saveScene()
            }
            // Wrapped, so a disabled button still says why it is: a disabled control receives no hover.
            Item {
                implicitWidth: publishButton.implicitWidth
                implicitHeight: publishButton.implicitHeight
                HoverHandler { id: publishHover }
                ToolTip.visible: publishHover.hovered && window.publishHint !== ""
                ToolTip.text: window.publishHint
                StudioButton {
                    id: publishButton
                    objectName: "publishButton"
                    kind: "ghost"
                    anchors.fill: parent
                    text: window.launcher && window.launcher.busy ? qsTr("Publishing…") : qsTr("Publish")
                    enabled: window.canPublish
                    onClicked: window.publish()
                }
            }
            StudioButton {
                id: playButton
                objectName: "playButton"
                kind: window.playing ? "danger" : "primary"
                text: window.playing ? qsTr("■  Stop") : qsTr("▶  Play")
                Accessible.name: window.playing ? qsTr("Stop") : qsTr("Play")
                ToolTip.visible: hovered
                ToolTip.text: window.playing ? qsTr("Stop (Esc)") : qsTr("Play (F5)")
                onClicked: window.playing ? window.stop() : window.play()
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ── left: the scene and the project ──
        Pane {
            Layout.preferredWidth: 270
            Layout.fillHeight: true
            enabled: !window.playing
            opacity: enabled ? 1 : 0.45
            Material.background: Look.base
            padding: 14

            ColumnLayout {
                anchors.fill: parent
                spacing: 8

                RowLayout {
                    Label {
                        text: qsTr("Scene")
                        font.family: Look.ui
                        font.weight: Font.DemiBold
                        font.pixelSize: Look.sizeMicro
                        font.letterSpacing: Look.sizeMicro * Look.trackingEyebrow
                        font.capitalization: Font.AllUppercase
                        color: Look.inkMuted
                        Layout.fillWidth: true
                    }
                    StudioButton {
                        text: qsTr("+ Add")
                        onClicked: addMenu.open()
                        Menu {
                            id: addMenu
                            MenuItem { text: qsTr("Cube"); onTriggered: window.add("Shape", { name: "Cube", form: "cube" }) }
                            MenuItem { text: qsTr("Sphere"); onTriggered: window.add("Shape", { name: "Sphere", form: "sphere" }) }
                            MenuItem { text: qsTr("Cylinder"); onTriggered: window.add("Shape", { name: "Cylinder", form: "cylinder" }) }
                            MenuItem { text: qsTr("Cone"); onTriggered: window.add("Shape", { name: "Cone", form: "cone" }) }
                            MenuItem {
                                text: qsTr("Lamp")
                                onTriggered: window.add("Lamp", { position: Qt.vector3d(window.target.x + 1, 2.5, window.target.z + 1) })
                            }
                            MenuItem {
                                text: qsTr("Spotlight")
                                onTriggered: window.add("Spotlight", { position: Qt.vector3d(window.target.x, 4, window.target.z) })
                            }
                            MenuItem { text: qsTr("Model…"); onTriggered: modelChooser.open() }
                            MenuItem {
                                text: qsTr("Group")
                                onTriggered: window.add("Group", { name: "Group" })
                            }
                            MenuItem { text: qsTr("Prefab…"); onTriggered: prefabChooser.open() }
                            MenuSeparator {}
                            MenuItem {
                                text: qsTr("Player")
                                enabled: !window.entityList.some(e => e.kind === "Player")
                                onTriggered: window.add("Player", { position: Qt.vector3d(window.target.x, 0, window.target.z + 3) })
                            }
                            MenuItem { text: qsTr("Particles"); onTriggered: window.add("Emitter", { name: "Particles" }) }
                            Menu {
                                title: qsTr("Interface")
                                MenuItem { text: qsTr("Text"); onTriggered: window.add("ScreenText", { name: "Text" }) }
                                MenuItem { text: qsTr("Bar"); onTriggered: window.add("ScreenBar", { name: "Bar" }) }
                                MenuItem { text: qsTr("Button"); onTriggered: window.add("ScreenButton", { name: "Button", anchor: "centre", insetX: 0, insetY: 0 }) }
                                MenuItem { text: qsTr("Image"); onTriggered: window.add("ScreenImage", { name: "Image" }) }
                                MenuItem { text: qsTr("Panel"); onTriggered: window.add("ScreenPanel", { name: "Panel" }) }
                            }
                            Menu {
                                title: qsTr("2D")
                                MenuItem { text: qsTr("Sprite"); onTriggered: window.add("Sprite", { name: "Sprite", position: Qt.vector3d(window.target.x, window.target.y, 0) }) }
                                MenuItem { text: qsTr("Tile map"); onTriggered: window.add("TileMap", { name: "Tiles", position: Qt.vector3d(window.target.x, window.target.y, 0) }) }
                                MenuItem {
                                    text: qsTr("Player")
                                    enabled: !window.entityList.some(e => e.kind === "Player" || e.kind === "Player2D")
                                    onTriggered: window.add("Player2D", { position: Qt.vector3d(window.target.x, window.target.y, 0) })
                                }
                            }
                            MenuItem { text: qsTr("Action"); onTriggered: window.add("InputAction", { name: "action", keys: "E", pad: "X" }) }
                            MenuItem {
                                text: qsTr("Trigger")
                                onTriggered: window.add("Trigger", { position: Qt.vector3d(window.target.x, 1, window.target.z),
                                                                     scale: Qt.vector3d(2, 2, 2) })
                            }
                            MenuItem { text: qsTr("Sound…"); onTriggered: soundChooser.open() }
                            MenuItem { text: qsTr("Music…"); onTriggered: musicChooser.open() }
                            MenuItem { text: qsTr("Behaviour"); onTriggered: window.addBehaviour() }
                        }
                    }
                }

                ItemDelegate {
                    Layout.fillWidth: true
                    text: qsTr("Look and sky")
                    highlighted: window.selection !== null && window.selection === window.scene
                    onClicked: window.selection = window.scene
                }

                ListView {
                    id: tree
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: window.entityList
                    delegate: ItemDelegate {
                        id: entry
                        required property var modelData
                        width: tree.width
                        highlighted: window.selection === modelData
                        onClicked: window.selection = modelData
                        contentItem: RowLayout {
                            spacing: 10
                            // Indented by the groups it is in.
                            Item { implicitWidth: window.depthOf(entry.modelData) * 14; visible: implicitWidth > 0 }
                            Rectangle {
                                width: 9
                                height: 9
                                radius: entry.modelData.kind === "Shape" && entry.modelData.form === "cube" ? 1 : 5
                                color: entry.modelData.kind === "Shape" || entry.modelData.kind === "Ground" ? entry.modelData.colour
                                     : entry.modelData.kind === "Lamp" || entry.modelData.kind === "Sun"
                                       || entry.modelData.kind === "Spotlight" ? entry.modelData.color
                                     : entry.modelData.kind === "Group" ? Look.inkMuted
                                     : entry.modelData.kind === "Prefab" ? Look.gilt
                                     : Look.accent
                            }
                            Label { text: entry.modelData.name; Layout.fillWidth: true; elide: Text.ElideRight }
                            Label { text: entry.modelData.kind; opacity: 0.4; font.pixelSize: 11 }
                        }
                    }
                }

                MenuSeparator { Layout.fillWidth: true }

                RowLayout {
                    Label {
                        text: qsTr("Project")
                        font.family: Look.ui
                        font.weight: Font.DemiBold
                        font.pixelSize: Look.sizeMicro
                        font.letterSpacing: Look.sizeMicro * Look.trackingEyebrow
                        font.capitalization: Font.AllUppercase
                        color: Look.inkMuted
                        Layout.fillWidth: true
                    }
                    StudioButton {
                        text: String(window.project) === "" ? qsTr("Choose…") : qsTr("Change…")
                        onClicked: projectChooser.open()
                    }
                }
                Label {
                    Layout.fillWidth: true
                    visible: String(window.project) !== ""
                    wrapMode: Text.Wrap
                    font.pixelSize: 12
                    opacity: 0.6
                    text: window.projectInfo.repository ? qsTr("Versioned with Qontrol")
                                                        : qsTr("Not versioned yet: open it in Projects")
                }
                ProjectPanel {
                    objectName: "projectPanel"
                    visible: String(window.project) !== ""
                    Layout.fillWidth: true
                    Layout.preferredHeight: 260
                    studio: window
                    info: window.projectInfo
                    store: window.scene ? SceneIO.storeOf(window.scene) : null
                }
            }
        }

        // ── middle: the world, and the logic editor under it when a file is open ──
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

        Item {
            id: viewport
            objectName: "viewport"
            Layout.fillWidth: true
            Layout.fillHeight: true

            StatsOverlay {
                objectName: "statsOverlay"
                visible: window.showStats
                z: 10
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 12
                world: world
                scene: window.playScene ?? window.scene
            }
            // Its draw calls are counted only while it shows; and in play every body is drawn as its collider.
            Binding { target: world.renderStats; property: "extendedDataCollectionEnabled"; value: window.showStats }
            Binding {
                target: window.playScene
                when: window.playScene !== null
                property: "debugPhysics"
                value: window.showStats
            }

            World {
                id: world
                objectName: "world"
                anchors.fill: parent
                target: !window.follow ? window.target
                      : flat ? window.playScene.keptInView(window.follow.lookAt, width / (2 * magnification),
                                                           height / (2 * magnification))
                      : window.follow.lookAt
                // In play with a player, the eye follows it; otherwise it orbits as in editing.
                eye: window.follow ? window.follow.eye : window.orbitEye
                exposure: window.scene ? window.scene.exposure : 1
                bloom: window.scene ? window.scene.bloom : true
                skyLight: window.scene ? window.scene.skyLight : 0.75
                skyTop: window.scene ? window.scene.skyTop : "#0b1430"
                skyHorizon: window.scene ? window.scene.skyHorizon : "#c7623a"
                groundHorizon: window.scene ? window.scene.groundHorizon : "#2a1a1a"
                groundBottom: window.scene ? window.scene.groundBottom : "#07080c"
                sky: window.scene && String(window.scene.sky) !== "" ? SceneIO.resolvedUrl(window.scene, window.scene.sky) : ""
                skyTurn: window.scene ? window.scene.skyTurn : 0
                // A 2D scene is edited flat (P3.10): panned by a drag, zoomed by the wheel to `distance` units tall, in
                // the look it plays in; in play it is scaled as the Player scales it.
                flat: window.scene ? window.scene.view === "2d" : false
                pixelsPerUnit: window.scene ? window.scene.pixelsPerUnit : 16
                viewHeight: window.playing && window.scene ? window.scene.viewHeight : window.distance
                pixelArt: window.scene ? window.scene.pixelArt : true
                freeZoom: !window.playing

                Node { id: stage; visible: !window.playing }
                Node { id: playStage }
            }

            // The handles, in a view of their own over the world: always on top, never part of the scene.
            View3D {
                id: overlay
                objectName: "overlay"
                anchors.fill: parent
                environment: SceneEnvironment {
                    backgroundMode: SceneEnvironment.Transparent
                    antialiasingMode: SceneEnvironment.MSAA
                }
                camera: world.flat ? overlayFlatCamera : overlayCamera
                OrthographicCamera {
                    id: overlayFlatCamera
                    position: world.activeCamera.position
                    horizontalMagnification: world.magnification
                    verticalMagnification: world.magnification
                    clipNear: 1
                    clipFar: 4000
                }
                PerspectiveCamera {
                    id: overlayCamera
                    position: world.viewCamera.position
                    rotation: world.viewCamera.rotation
                    fieldOfView: world.viewCamera.fieldOfView
                    clipNear: world.viewCamera.clipNear
                    clipFar: world.viewCamera.clipFar
                }
                Gizmo {
                    id: gizmo
                    objectName: "gizmo"
                    target: window.playing ? null : window.selection
                    flat: world.flat
                    // Flat, the handles keep the size they would have at the view's height away.
                    eye: world.flat ? position.plus(Qt.vector3d(0, 0, world.viewHeight)) : world.activeCamera.position
                }
                // After this view changes camera (flat or not), the first change to its handles was not drawn: found in
                // P3.10's checks, and drawn once anything asked for a frame later. So it asks, just after its camera
                // changes and just after its handles move, show, hide or change size.
                function repaint() { Qt.callLater(() => overlay.update()) }
                onCameraChanged: repaint()
                Connections {
                    target: gizmo
                    function onScenePositionChanged() { overlay.repaint() }
                    function onVisibleChanged() { overlay.repaint() }
                    function onSizeChanged() { overlay.repaint() }
                }
            }

            TapHandler {
                enabled: !window.playing
                onTapped: (point) => {
                    const hit = world.pick(point.position.x, point.position.y)
                    window.selection = window.entityOf(hit.objectHit)
                }
            }

            DragHandler {
                target: null
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                property int handle: -1
                property vector3d startPosition
                property vector3d startRotation
                property vector3d startScale
                property real startYaw
                property real startPitch
                property vector3d startTarget
                property point lastTranslation

                onActiveChanged: {
                    lastTranslation = Qt.point(0, 0)
                    if (active) {
                        const p = centroid.pressPosition
                        const hit = gizmo.visible ? overlay.pick(p.x, p.y) : null
                        handle = hit && hit.objectHit && hit.objectHit.axis !== undefined ? hit.objectHit.axis : -1
                        if (handle >= 0) {
                            startPosition = window.selection.position
                            startRotation = window.selection.eulerRotation
                            startScale = window.selection.scale
                        } else {
                            startYaw = window.yaw
                            startPitch = window.pitch
                            startTarget = window.target
                        }
                    } else if (handle >= 0) {
                        handle = -1
                        window.refresh()
                    }
                }
                onTranslationChanged: {
                    // In play, a drag looks around through the player's eyes.
                    if (window.follow) {
                        Input.pointerMoved(Qt.point(translation.x - lastTranslation.x, translation.y - lastTranslation.y))
                        lastTranslation = Qt.point(translation.x, translation.y)
                        return
                    }
                    if (handle < 0 && world.flat) {
                        // Flat, a drag pans: the world follows the pointer.
                        window.target = startTarget.minus(Qt.vector3d(translation.x / world.magnification,
                                                                      -translation.y / world.magnification, 0))
                        return
                    }
                    if (handle < 0) {
                        window.yaw = startYaw - translation.x * 0.3
                        window.pitch = Math.max(-5, Math.min(85, startPitch + translation.y * 0.3))
                        return
                    }
                    if (handle === 4) {
                        // Even scaling: right grows, left shrinks, by the same factor on every axis; snapping, by tenths.
                        const factor = Math.exp(translation.x * 0.01)
                        window.selection.scale = startScale.times(window.snapStep > 0 ? Math.max(0.1, Math.round(factor * 10) / 10)
                                                                                      : factor)
                        return
                    }
                    // Turning: snapping, to whole steps of snapAngle.
                    const turned = (from, by) => window.snapStep > 0 ? Math.round((from + by) / window.snapAngle) * window.snapAngle
                                                                     : from + by
                    if (handle === 3 && world.flat) {
                        window.selection.eulerRotation = Qt.vector3d(startRotation.x, startRotation.y,
                                                                     turned(startRotation.z, -translation.x * 0.5))
                        return
                    }
                    if (handle === 3) {
                        window.selection.eulerRotation = Qt.vector3d(startRotation.x, turned(startRotation.y, translation.x * 0.5),
                                                                     startRotation.z)
                        return
                    }
                    // Along one axis: the drag measured along that axis as it appears on screen.
                    const axis = [Qt.vector3d(1, 0, 0), Qt.vector3d(0, 1, 0), Qt.vector3d(0, 0, 1)][handle]
                    const origin = window.selection.scenePosition
                    const a = world.mapFrom3DScene(origin)
                    const b = world.mapFrom3DScene(origin.plus(axis))
                    const dx = b.x - a.x
                    const dy = b.y - a.y
                    const length2 = dx * dx + dy * dy
                    if (length2 < 1)
                        return
                    const along = (translation.x * dx + translation.y * dy) / length2
                    const moved = startPosition.plus(axis.times(along))
                    // Snapping: the axis dragged along lands on the grid; the others stay where they were.
                    window.selection.position = Qt.vector3d(handle === 0 ? window.snapped(moved.x) : moved.x,
                                                            handle === 1 ? window.snapped(moved.y) : moved.y,
                                                            handle === 2 ? window.snapped(moved.z) : moved.z)
                }
            }

            WheelHandler {
                onWheel: (event) => window.distance = Math.max(1.5, Math.min(80, window.distance * Math.pow(0.9, event.angleDelta.y / 120)))
            }

            Label {
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                anchors.margins: 14
                text: !window.playing && world.flat ? qsTr("Click to select · drag a handle to move · drag to pan · scroll to zoom · F frames the selection · F5 plays")
                    : !window.playing ? qsTr("Click to select · drag a handle to move · drag to orbit · scroll to zoom · F frames the selection · F5 plays")
                    : window.follow && world.flat && window.follow.style === "top-down" ? qsTr("WASD or the left stick to move · Shift to run · Esc stops")
                    : window.follow && world.flat ? qsTr("A and D or the left stick to move · Space or A to jump · Shift to run · Esc stops")
                    : window.follow ? qsTr("WASD or the left stick to move · Space or A to jump · Shift to run · drag, the arrows or the right stick to look · Esc stops")
                    : qsTr("Drag to orbit · scroll to zoom · Esc stops · add a Player to walk the world")
                opacity: 0.45
                font.pixelSize: 12
            }

            // While playing, a quiet mark that this is play and not the scene being edited.
            Rectangle {
                visible: window.playing
                anchors.top: parent.top
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.topMargin: 14
                width: playingLabel.implicitWidth + 28
                height: 30
                radius: 15
                color: Qt.rgba(0.05, 0.06, 0.08, 0.72)
                border.color: Qt.rgba(0.24, 0.86, 0.52, 0.5)
                Row {
                    anchors.centerIn: parent
                    spacing: 8
                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        anchors.verticalCenter: parent.verticalCenter
                        color: Look.ok
                        // Still, for anyone who asked for less motion.
                        SequentialAnimation on opacity {
                            running: window.playing && !Look.reducedMotion
                            loops: Animation.Infinite
                            NumberAnimation { to: 0.3; duration: 700; easing.type: Easing.InOutSine }
                            NumberAnimation { to: 1; duration: 700; easing.type: Easing.InOutSine }
                        }
                    }
                    Label {
                        id: playingLabel
                        text: qsTr("PLAYING") + (Input.gamepad ? "  ·  " + qsTr("gamepad") : "")
                        font.family: Look.ui
                        font.pixelSize: Look.sizeCaption
                        font.letterSpacing: Look.sizeCaption * Look.trackingLabel
                        font.weight: Font.DemiBold
                        color: Look.ok
                    }
                }
            }
        }

        LogicEditor {
            id: logicEditor
            objectName: "logicEditor"
            visible: name !== ""
            Layout.fillWidth: true
            Layout.preferredHeight: 280
            studio: window
        }
        }

        // ── right: the selection's fields ──
        Pane {
            Layout.preferredWidth: 320
            Layout.fillHeight: true
            enabled: !window.playing
            opacity: enabled ? 1 : 0.45
            Material.background: Look.base
            padding: 16

            ScrollView {
                anchors.fill: parent
                contentWidth: availableWidth

                ColumnLayout {
                    width: parent.width
                    spacing: 12

                    // Its name and kind on a line of their own; what can be done with it under them.
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0
                        Label {
                            text: window.selection ? (window.selection === window.scene ? qsTr("Look and sky")
                                                                                        : window.selection.name)
                                                   : qsTr("Nothing selected")
                            font.family: Look.ui
                            font.pixelSize: Look.sizeHeading + 2
                            font.weight: Font.DemiBold
                            font.letterSpacing: (Look.sizeHeading + 2) * Look.trackingHeading
                            color: Look.ink
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                        Label {
                            visible: window.selection !== null && window.selection !== window.scene
                            text: window.selection ? window.selection.kind : ""
                            font.family: Look.ui
                            font.pixelSize: Look.sizeCaption
                            font.letterSpacing: Look.sizeCaption * Look.trackingLabel
                            font.capitalization: Font.AllUppercase
                            color: Look.gilt
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        visible: window.selection !== null && window.selection !== window.scene
                        spacing: 4
                        StudioButton {
                            objectName: "groupButton"
                            visible: window.selection !== null && window.selection !== window.scene
                                     && SceneIO.nestable(window.selection.kind)
                            text: qsTr("Group")
                            ToolTip.visible: hovered
                            ToolTip.text: qsTr("Put it in a new group (Ctrl+G)")
                            onClicked: window.groupSelection()
                        }
                        StudioButton {
                            objectName: "duplicateButton"
                            visible: window.selection !== null && window.selection !== window.scene
                            text: qsTr("Duplicate")
                            ToolTip.visible: hovered
                            ToolTip.text: qsTr("Duplicate (Ctrl+D)")
                            onClicked: window.duplicateSelection()
                        }
                        StudioButton {
                            visible: window.selection !== null && window.selection !== window.scene
                            text: qsTr("Delete")
                            onClicked: window.removeSelection()
                        }
                    }

                    Label {
                        visible: !window.selection
                        Layout.fillWidth: true
                        wrapMode: Text.Wrap
                        opacity: 0.55
                        text: qsTr("Select something in the world or in the list to change it.")
                    }

                    // Which group it is in, to move it to another, or out to the scene's top.
                    RowLayout {
                        visible: window.selection !== null && window.selection !== window.scene
                                 && SceneIO.nestable(window.selection.kind)
                        Layout.fillWidth: true
                        Label { text: qsTr("In group"); opacity: 0.7; Layout.preferredWidth: 90 }
                        ComboBox {
                            objectName: "groupChoice"
                            Layout.fillWidth: true
                            readonly property var choices: window.selection ? [null].concat(window.groupsFor(window.selection)) : [null]
                            model: choices.map(g => g ? g.name : qsTr("None (the scene)"))
                            currentIndex: window.selection ? Math.max(0, choices.indexOf(window.selection.parent)) : 0
                            onActivated: (i) => window.moveSelectionInto(choices[i])
                        }
                    }
                    StudioButton {
                        objectName: "editLogicButton"
                        visible: window.selection !== null && window.selection.kind === "Behaviour"
                                 && window.logicNameOf(window.selection) !== ""
                        text: qsTr("Edit logic")
                        onClicked: window.openLogic(window.logicNameOf(window.selection))
                    }
                    StudioButton {
                        objectName: "unpackButton"
                        visible: window.selection !== null && window.selection.kind === "Prefab"
                        text: qsTr("Unpack")
                        ToolTip.visible: hovered
                        ToolTip.text: qsTr("Make it a group of its parts, to change them")
                        onClicked: window.unpackPrefab()
                    }
                    RowLayout {
                        visible: window.selection !== null && window.selection.kind === "Group"
                        StudioButton {
                            objectName: "prefabButton"
                            text: qsTr("Keep as prefab")
                            ToolTip.visible: hovered
                            ToolTip.text: qsTr("Write it to prefabs/ and place it as a prefab")
                            onClicked: window.makePrefab()
                        }
                        StudioButton {
                            objectName: "ungroupButton"
                            text: qsTr("Ungroup")
                            flat: true
                            ToolTip.visible: hovered
                            ToolTip.text: qsTr("Take what is in it out (Ctrl+Shift+G)")
                            onClicked: window.ungroupSelection()
                        }
                    }

                    Inspector {
                        id: inspector
                        objectName: "inspector"
                        Layout.fillWidth: true
                        target: window.selection
                        onEdited: {
                            window.editKey = inspector.lastEdit
                            window.refresh()
                        }
                    }
                }
            }
        }

        // ── the agent, when it is open (usable in play: that is when it looks) ──
        Pane {
            visible: window.agentOpen && window.designer !== null
            Layout.preferredWidth: 380
            Layout.fillHeight: true
            Material.background: Look.well
            padding: 16

            Loader {
                anchors.fill: parent
                active: window.designer !== null
                sourceComponent: AgentPanel { designer: window.designer }
            }
        }
    }

    Connections {
        target: window.launcher
        function onPublished(answer) {
            const r = answer.receipt
            if (answer.action === "minted")
                window.say(qsTr("Published as Cartridge %1/%2, \"%3\", at commit %4.")
                           .arg(r.collection).arg(r.item).arg(r.name).arg(r.commit.id.slice(0, 10)))
            else if (answer.action === "revised")
                window.say(qsTr("Published again: Cartridge %1/%2 is now at commit %3.")
                           .arg(r.collection).arg(r.item).arg(r.commit.id.slice(0, 10)))
            else
                window.say(qsTr("Already published: Cartridge %1/%2 is at this commit.")
                           .arg(answer.collection).arg(answer.item))
            window.projectInfo = SceneIO.project(window.project)
        }
        function onFailed(code, message) {
            window.say(code === "declined" ? qsTr("Not published: you declined it in the launcher. Nothing was signed.")
                                           : qsTr("Not published: %1").arg(message), code !== "declined")
        }
    }

    FolderDialog {
        id: projectChooser
        property bool thenSave: false
        title: qsTr("Choose the project folder")
        onAccepted: {
            window.chooseProject(selectedFolder)
            if (thenSave) {
                thenSave = false
                window.saveScene()
            }
        }
        onRejected: thenSave = false
    }

    FileDialog {
        id: modelChooser
        title: qsTr("Add a glTF model")
        nameFilters: [qsTr("glTF 2.0 (*.gltf *.glb)")]
        onAccepted: window.add("Prop", { name: String(selectedFile).split("/").pop().replace(/\.(gltf|glb)$/i, ""),
                                         source: selectedFile })
    }

    FileDialog {
        id: soundChooser
        title: qsTr("Add a sound")
        nameFilters: [qsTr("Sound (*.wav *.mp3 *.ogg *.flac)")]
        onAccepted: window.add("Sound", { name: String(selectedFile).split("/").pop().replace(/\.[a-z0-9]+$/i, ""),
                                          source: selectedFile })
    }

    FileDialog {
        id: musicChooser
        title: qsTr("Add the scene's music")
        nameFilters: [qsTr("Sound (*.wav *.mp3 *.ogg *.flac)")]
        onAccepted: window.add("Music", { name: "Music", source: selectedFile })
    }

    FileDialog {
        id: prefabChooser
        title: qsTr("Place a prefab")
        currentFolder: String(window.project) !== "" ? String(window.project) + "/prefabs" : ""
        nameFilters: [qsTr("QQ prefabs (*.qml)")]
        onAccepted: window.add("Prefab", { name: String(selectedFile).split("/").pop().replace(/\.qml$/i, ""),
                                           source: selectedFile })
    }

    FileDialog {
        id: importChooser
        title: qsTr("Import a 2D QQ scene")
        nameFilters: [qsTr("QQ 2D scene (*.qq.json)")]
        onAccepted: window.importScene(selectedFile)
    }
}
