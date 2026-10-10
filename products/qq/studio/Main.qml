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
    color: "#07080c"

    Material.theme: Material.Dark
    Material.accent: "#ff6a00"
    Material.primary: "#11131a"

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

    function refresh() {
        entityList = scene ? SceneIO.entities(scene) : []
        dirty = scene ? SceneIO.write(scene, sceneFile) !== savedText : false
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
        refresh()
        return true
    }

    function newScene() {
        const text = 'import QQ\nScene {\n    name: "untitled"\n    Sun { elevation: 38 }\n    Ground {}\n'
                   + '    Shape { name: "Plinth"; position: Qt.vector3d(0, 0.25, 0); scale: Qt.vector3d(2, 0.5, 2); '
                   + 'colour: "#2b3140"; roughness: 0.6 }\n'
                   + '    Prop { name: "Orb"; position: Qt.vector3d(0, 1.75, 0); source: ' + JSON.stringify(String(model))
                   + ' }\n}\n'
        if (show(SceneIO.loadText(text, fileFor("untitled"), stage), fileFor("untitled"), "untitled", false))
            say(qsTr("A new scene. Save it to keep it in the project."))
    }

    function openScene(name) {
        if (show(SceneIO.load(fileFor(name), stage), fileFor(name), name, true))
            say(qsTr("Opened scenes/%1.qml").arg(name))
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
        // Files from outside the project (models, sounds, logic) come into it first, so the project is whole on its own.
        for (const e of SceneIO.entities(scene)) {
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
        say(qsTr("Imported %1 entities.").arg(result.imported)
            + (result.skipped.length ? " " + qsTr("Not yet in 3D: %1.").arg(result.skipped.join("; ")) : ""))
    }

    function add(kind, properties) {
        if (!scene)
            return null
        let n = 1
        const base = properties.name ?? kind
        let name = base
        while (entityList.some(e => e.name === name))
            name = base + " " + (++n)
        properties.name = name
        if (kind !== "Sun" && kind !== "Ground" && kind !== "Behaviour" && properties.position === undefined)
            properties.position = Qt.vector3d(window.target.x, Math.max(0.5, window.target.y), window.target.z)
        const made = SceneIO.add(scene, kind, properties)
        if (!made) {
            say(SceneIO.lastError, true)
            return null
        }
        selection = made
        refresh()
        return made
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
        refresh()
        say(qsTr("Stopped. The scene is as it was before play."))
        return true
    }

    /// Put everything away in a safe order, before the window goes: what the application does as it quits.
    function release() {
        stop()
        if (scene)
            SceneIO.discard(scene)
        scene = null
        selection = null
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
        + '// `target` is the entity this drives, `scene` the scene, `elapsed` the seconds since it was built; `Input` has\n'
        + '// move, look, jump and run.\n\nimport QQ\n\nLogic {\n    property real speed: 90  // degrees a second\n\n'
        + '    onFrame: (dt) => {\n        if (target)\n            target.eulerRotation.y += speed * dt\n    }\n}\n'

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

    /// The entity a picked model belongs to: the nearest ancestor that declares a kind, other than the scene.
    function entityOf(object) {
        let o = object
        while (o && !(o.kind !== undefined && o.kind !== "Scene"))
            o = o.parent
        return o ?? null
    }

    Component.onCompleted: {
        if (String(project) !== "")
            chooseProject(project)
        newScene()
    }

    Shortcut { sequences: [StandardKey.Save]; onActivated: window.saveScene() }
    Shortcut { sequences: [StandardKey.Delete]; enabled: !window.playing; onActivated: window.removeSelection() }
    Shortcut { sequence: "F"; enabled: !window.playing; onActivated: window.frameSelection() }
    Shortcut { sequence: "F5"; onActivated: window.playing ? window.stop() : window.play() }
    Shortcut { sequence: "Escape"; enabled: window.playing; onActivated: window.stop() }

    // ── the window ─────────────────────────────────────────────────────────────────────────────

    header: ToolBar {
        Material.background: "#0d0f15"
        height: 56
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 12
            spacing: 12

            Label {
                text: "QQ"
                font.pixelSize: 22
                font.weight: Font.Black
                color: Material.accent
            }
            Label {
                text: window.projectInfo.name !== "" ? window.projectInfo.name : qsTr("No project")
                opacity: 0.7
            }
            Label { text: "/"; opacity: 0.35 }
            TextField {
                Layout.preferredWidth: 200
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
            Label {
                Layout.maximumWidth: 560
                elide: Text.ElideRight
                text: window.playProblem !== "" ? window.playProblem : window.notice
                color: window.noticeIsProblem || window.playProblem !== "" ? "#d57889" : Material.foreground
                opacity: window.noticeIsProblem || window.playProblem !== "" ? 1 : 0.7
                font.pixelSize: 13
            }
            ToolButton { text: qsTr("New"); enabled: !window.playing; onClicked: window.newScene() }
            ToolButton { text: qsTr("Import 2D…"); enabled: !window.playing; onClicked: importChooser.open() }
            Button {
                text: qsTr("Save")
                enabled: !window.playing
                highlighted: window.dirty
                onClicked: window.saveScene()
            }
            Button {
                id: playButton
                objectName: "playButton"
                text: window.playing ? qsTr("■  Stop") : qsTr("▶  Play")
                highlighted: !window.playing
                Material.accent: window.playing ? "#d57889" : "#3ddc84"
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
            Material.background: "#0d0f15"
            padding: 14

            ColumnLayout {
                anchors.fill: parent
                spacing: 8

                RowLayout {
                    Label {
                        text: qsTr("Scene")
                        font.pixelSize: 11
                        font.capitalization: Font.AllUppercase
                        opacity: 0.6
                        Layout.fillWidth: true
                    }
                    ToolButton {
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
                            MenuItem { text: qsTr("Model…"); onTriggered: modelChooser.open() }
                            MenuSeparator {}
                            MenuItem {
                                text: qsTr("Player")
                                enabled: !window.entityList.some(e => e.kind === "Player")
                                onTriggered: window.add("Player", { position: Qt.vector3d(window.target.x, 0, window.target.z + 3) })
                            }
                            MenuItem { text: qsTr("Particles"); onTriggered: window.add("Emitter", { name: "Particles" }) }
                            MenuItem { text: qsTr("Sound…"); onTriggered: soundChooser.open() }
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
                            Rectangle {
                                width: 9
                                height: 9
                                radius: entry.modelData.kind === "Shape" && entry.modelData.form === "cube" ? 1 : 5
                                color: entry.modelData.kind === "Shape" || entry.modelData.kind === "Ground" ? entry.modelData.colour
                                     : entry.modelData.kind === "Lamp" || entry.modelData.kind === "Sun" ? entry.modelData.color
                                     : "#ff6a00"
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
                        font.pixelSize: 11
                        font.capitalization: Font.AllUppercase
                        opacity: 0.6
                        Layout.fillWidth: true
                    }
                    ToolButton {
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
                Repeater {
                    model: window.projectInfo.scenes
                    delegate: ItemDelegate {
                        required property string modelData
                        Layout.fillWidth: true
                        text: modelData
                        font.pixelSize: 13
                        highlighted: modelData === window.sceneName && !window.dirty
                        onClicked: window.openScene(modelData)
                    }
                }
            }
        }

        // ── middle: the world ──
        Item {
            id: viewport
            objectName: "viewport"
            Layout.fillWidth: true
            Layout.fillHeight: true

            World {
                id: world
                objectName: "world"
                anchors.fill: parent
                target: window.follow ? window.follow.lookAt : window.target
                // In play with a player, the eye follows it; otherwise it orbits as in editing.
                eye: window.follow ? window.follow.eye : window.orbitEye
                exposure: window.scene ? window.scene.exposure : 1
                bloom: window.scene ? window.scene.bloom : true
                skyLight: window.scene ? window.scene.skyLight : 0.75
                skyTop: window.scene ? window.scene.skyTop : "#0b1430"
                skyHorizon: window.scene ? window.scene.skyHorizon : "#c7623a"
                groundHorizon: window.scene ? window.scene.groundHorizon : "#2a1a1a"
                groundBottom: window.scene ? window.scene.groundBottom : "#07080c"

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
                camera: overlayCamera
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
                    eye: world.viewCamera.position
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
                    if (handle < 0) {
                        window.yaw = startYaw - translation.x * 0.3
                        window.pitch = Math.max(-5, Math.min(85, startPitch + translation.y * 0.3))
                        return
                    }
                    if (handle === 4) {
                        // Even scaling: right grows, left shrinks, by the same factor on every axis.
                        window.selection.scale = startScale.times(Math.exp(translation.x * 0.01))
                        return
                    }
                    if (handle === 3) {
                        window.selection.eulerRotation = Qt.vector3d(startRotation.x, startRotation.y + translation.x * 0.5,
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
                    window.selection.position = startPosition.plus(axis.times(along))
                }
            }

            WheelHandler {
                onWheel: (event) => window.distance = Math.max(1.5, Math.min(80, window.distance * Math.pow(0.9, event.angleDelta.y / 120)))
            }

            Label {
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                anchors.margins: 14
                text: !window.playing ? qsTr("Click to select · drag a handle to move · drag to orbit · scroll to zoom · F frames the selection · F5 plays")
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
                        color: "#3ddc84"
                        SequentialAnimation on opacity {
                            running: window.playing
                            loops: Animation.Infinite
                            NumberAnimation { to: 0.3; duration: 700; easing.type: Easing.InOutSine }
                            NumberAnimation { to: 1; duration: 700; easing.type: Easing.InOutSine }
                        }
                    }
                    Label {
                        id: playingLabel
                        text: qsTr("PLAYING") + (Input.gamepad ? "  ·  " + qsTr("gamepad") : "")
                        font.pixelSize: 11
                        font.letterSpacing: 1.5
                        font.weight: Font.DemiBold
                        color: "#3ddc84"
                    }
                }
            }
        }

        // ── right: the selection's fields ──
        Pane {
            Layout.preferredWidth: 320
            Layout.fillHeight: true
            enabled: !window.playing
            opacity: enabled ? 1 : 0.45
            Material.background: "#0d0f15"
            padding: 16

            ScrollView {
                anchors.fill: parent
                contentWidth: availableWidth

                ColumnLayout {
                    width: parent.width
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0
                            Label {
                                text: window.selection ? (window.selection === window.scene ? qsTr("Look and sky")
                                                                                            : window.selection.name)
                                                       : qsTr("Nothing selected")
                                font.pixelSize: 18
                                font.weight: Font.DemiBold
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                            Label {
                                visible: window.selection !== null && window.selection !== window.scene
                                text: window.selection ? window.selection.kind : ""
                                opacity: 0.5
                                font.pixelSize: 12
                            }
                        }
                        ToolButton {
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

                    Inspector {
                        Layout.fillWidth: true
                        target: window.selection
                        onEdited: window.refresh()
                    }
                }
            }
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
        id: importChooser
        title: qsTr("Import a 2D QQ scene")
        nameFilters: [qsTr("QQ 2D scene (*.qq.json)")]
        onAccepted: window.importScene(selectedFile)
    }
}
