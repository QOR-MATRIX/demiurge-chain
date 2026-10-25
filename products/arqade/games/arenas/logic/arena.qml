// Flux Four and Rift Reversi (DIRECTION P7.24, ADR-102), ARQADE's two arenas rebuilt in QQ, played against another
// person on ARQADE's server. This logic plays both: the scene it is in names the arena (arena.mjs's SCENES).
//
// What the game asks and what it makes of the answers is arena.mjs, the one place it lives: this logic hands it every
// event as QQ's tick delivers it (a screen button's click, what the host told, an action's press, a step), sends what
// it asks to the host (askHost), and draws the board the server sent. Nothing drawn here decides anything, and nothing
// here decides a move: the server does, as it does for the website.
//
// The board stands in a group turned toward where the eye looks from. Flux Four's is a frame of seven columns of wells,
// six deep, its pulses dropping into them and coming to rest with a bounce; Rift Reversi's is six by six tiles, its
// discs set down and turned over, each turned disc lifting as it flips. The player's move waits over the board in their
// colour (the host's gold, the guest's cyan): a pulse held over the column with a ring where it would land, or a
// see-through disc on the cell, with each legal cell marked. The last move holds a spark. Four in a row pulse as the match ends. With less motion asked for (Look),
// pulses arrive and discs turn at once, and nothing bobs or pulses.
//
// A pulse landing, a disc set down and the discs turning are heard from where they happen, through QQ's spatial audio;
// the player's move coming round, a challenger arriving, the match's end and the music are heard everywhere.
//
// The pointer chooses too: over the board it moves the cursor, and its press makes the move. Where the pointer is
// belongs to the window, not the run, so a replay makes nothing of the press; the move it asked for is in the server's
// answer, which the run keeps.

import QQ
import "arena.mjs" as Arena

Logic {
    id: arena

    /// The conversation's state (arena.mjs), which only it changes.
    property var game: null
    /// The arena: "flux" or "reversi".
    property string which: ""

    /// Nobody's, the host's and the guest's colours.
    readonly property var colours: ["#9ba2ad", "#f0a35c", "#85c3cf"]
    property real pitch: 0.92
    /// The board's cells, by number, which the pointer picks; the discs on them (null where none).
    property var cells: []
    property var discs: []
    /// Rift Reversi's marks on the legal moves.
    property var marks: []
    /// The move about to be made: Flux Four's pulse held over its column and the ring round where it would land; Rift
    /// Reversi's disc, see-through, on the cell.
    property var ghost: null
    property var ring: null
    /// The last move's spark.
    property var last: null
    property var sparks: null
    property real burst: 0
    /// Flux Four's winning line, pulsing as the match ends.
    property var line: []
    property bool wasMyTurn: false
    /// How solid Rift Reversi's see-through disc is: more on a legal cell than on any other.
    property real held: 0.55
    /// Where the pointer was at the last frame: { x, y }.
    property var pointer: ({ x: -1, y: -1 })
    property real clock: 0

    readonly property var buttons: ["Create", "Join", "Return", "Close lobby", "Leave waiting", "Concede", "Leave match",
                                    "Concede match", "Keep playing", "Rematch", "Leave result", "Find another"]

    /// Cell `i`'s place on the board, its middle on the board's face.
    function at(i) {
        const board = Arena.BOARDS[which]
        const c = i % board.columns, r = Math.floor(i / board.columns)
        return Qt.vector3d((c - (board.columns - 1) / 2) * pitch, ((board.rows - 1) / 2 - r) * pitch, 0)
    }

    /// Something of the board, made here: only looked at, so it has no body.
    function piece(fields) {
        fields.body = "none"
        return spawn("Shape", fields, entity("Board"))
    }

    function send(message) {
        if (message)
            askHost(message)
    }

    function text(name, words) {
        const shown = entity(name)
        if (shown && shown.text !== words)
            shown.text = words
    }

    /// The screens and their words for where the conversation is; and what the run came to.
    function show() {
        outcome = Arena.outcome(game)
        const s = Arena.say(game)
        const where = Arena.screen(game)
        onlyScreen(where)
        if (where === "lobby") {
            if (s.canCreate)
                showScreen("lobby-create")
            if (s.canJoin)
                showScreen("lobby-join")
            if (s.canReturn)
                showScreen("lobby-return")
        }
        if (where === "match" && game.confirming)
            showScreen("concede")
        if (s.notice !== "")
            showScreen("notice")
        text("Status", s.status)
        text("Join", s.join)
        text("Waiting line", s.waiting)
        text("Host", s.host)
        text("Guest", s.guest)
        text("Turn", s.turn)
        text("Clock", s.clock)
        text("Result", s.result)
        text("Result line", s.line)
        text("Notice", s.notice)
        const clockText = entity("Clock")
        if (clockText)
            clockText.colour = Arena.myTurn(game) && Arena.secondsLeft(game) <= 10 ? "#ff8a4c" : "#d6d1c6"
        clock = Arena.secondsLeft(game)
    }

    /// The disc or pulse on cell `i`, of `seat`'s colour, made if there is none.
    function disc(i, seat) {
        let d = discs[i]
        if (!d) {
            const p = at(i)
            d = piece({ name: "Disc " + i, form: "cylinder", position: Qt.vector3d(p.x, p.y, which === "flux" ? 0.08 : 0.05),
                        eulerRotation: Qt.vector3d(90, 0, 0),
                        scale: which === "flux" ? Qt.vector3d(0.62, 0.16, 0.62) : Qt.vector3d(0.7, 0.1, 0.7),
                        colour: colours[seat], metalness: 0.45, roughness: 0.22, emissive: colours[seat],
                        emissivePower: 0.12 })
            const now = discs.slice()
            now[i] = d
            discs = now
        }
        return d
    }

    function tint(d, seat) {
        d.colour = colours[seat]
        d.emissive = colours[seat]
    }

    /// Where `d` is now, as a copy: a vector read from a field follows the field as it changes.
    function placeOf(d) {
        return Qt.vector3d(d.position.x, d.position.y, d.position.z)
    }

    /// Rift Reversi's disc turned over to `seat`'s colour, lifting as it turns.
    function flip(d, seat, delay) {
        if (Look.reducedMotion) {
            tint(d, seat)
            return
        }
        after(delay, () => {
            const rest = placeOf(d)
            tween(d, "position", Qt.vector3d(rest.x, rest.y, 0.3), 0.14, "outQuad",
                  () => tween(d, "position", Qt.vector3d(rest.x, rest.y, 0.05), 0.16, "inQuad"))
            tween(d, "eulerRotation", Qt.vector3d(180, 0, 0), 0.14, "inQuad", () => {
                tint(d, seat)
                tween(d, "eulerRotation", Qt.vector3d(270, 0, 0), 0.16, "outQuad",
                      () => d.eulerRotation = Qt.vector3d(90, 0, 0))
            })
        })
    }

    /// The spark on the last move, and a burst of it.
    function spark(i, seat) {
        const p = at(i)
        last.position = Qt.vector3d(p.x, p.y, which === "flux" ? 0.17 : 0.11)
        last.visible = true
        sparks.position = Qt.vector3d(p.x, p.y, 0.2)
        sparks.colour = colours[seat]
        burst = 1
    }

    /// The board as the server sent it. What changed says what moves: a pulse dropped, a disc set down and turned.
    function sync(changed) {
        const m = game.match
        const cellsNow = m ? m.cells : []
        for (let i = 0; i < cells.length; i++) {
            const seat = cellsNow[i] ?? 0
            if (seat === 0) {
                if (discs[i]) {
                    remove(discs[i])
                    const now = discs.slice()
                    now[i] = null
                    discs = now
                }
                continue
            }
            const fresh = !discs[i]
            const d = disc(i, seat)
            const animate = changed.moved && i === m.placed && fresh
            if (animate && which === "flux") {
                const rest = placeOf(d)
                d.position = Qt.vector3d(rest.x, at(0).y + pitch * 1.2, rest.z)
                const fall = Math.floor(i / 7) + 1.2
                if (Look.reducedMotion) {
                    d.position = rest
                    play("../assets/drop.wav", { volume: 0.8, at: d.scenePosition })
                } else {
                    tween(d, "position", rest, 0.16 + 0.05 * fall, "outBounce",
                          () => play("../assets/drop.wav", { volume: 0.8, at: d.scenePosition }))
                }
            } else if (animate) {
                play("../assets/place.wav", { volume: 0.8, at: d.scenePosition })
                if (!Look.reducedMotion) {
                    const full = Qt.vector3d(d.scale.x, d.scale.y, d.scale.z)
                    d.scale = Qt.vector3d(0.05, 0.1, 0.05)
                    tween(d, "scale", full, 0.25, "outBack")
                }
            } else if (changed.moved && m.flipped.indexOf(i) >= 0 && !fresh) {
                const p = at(m.placed), q = at(i)
                const far = Math.max(Math.abs(q.x - p.x), Math.abs(q.y - p.y)) / pitch
                flip(d, seat, 0.08 + 0.06 * far)
            } else if (!changed.moved || m.flipped.indexOf(i) < 0) {
                tint(d, seat)
            }
        }
        if (changed.moved && m.flipped.length > 0) {
            const turned = discs[m.flipped[0]]
            if (turned)
                after(0.1, () => play("../assets/flip.wav", { volume: 0.7, at: turned.scenePosition }))
        }
        if (m && m.placed >= 0 && m.cells[m.placed] !== 0)
            spark(m.placed, m.cells[m.placed])
        else
            last.visible = false
        line = m && m.status === "done" && which === "flux" && m.winner > 0 ? Arena.fourLine(m.cells, m.winner) : []
        if (changed.began && m.moves === 0)
            play("../assets/joined.wav", { volume: 0.7 })
        if (changed.ended)
            play(m.seat > 0 && m.winner === m.seat ? "../assets/won.wav" : "../assets/over.wav", { volume: 0.8 })
        const mine = Arena.myTurn(game)
        if (mine && !wasMyTurn && !changed.ended)
            play("../assets/turn.wav", { volume: 0.5 })
        wasMyTurn = mine
        aimed()
    }

    /// The move about to be made, drawn in the player's colour while it is theirs.
    function aimed() {
        const m = game.match
        const mine = Arena.myTurn(game) && game.waiting === ""
        const colour = colours[m ? m.seat : 0]
        if (which === "flux") {
            const landing = m ? Arena.landing(m.cells, game.cursor) : -1
            const top = at(game.cursor)
            ghost.visible = mine
            ghost.position = Qt.vector3d(top.x, top.y + pitch * 1.2, 0.08)
            ghost.colour = colour
            ghost.emissive = colour
            ring.visible = mine && landing >= 0
            if (landing >= 0) {
                const p = at(landing)
                ring.position = Qt.vector3d(p.x, p.y, -0.06)
            }
            ring.colour = colour
            ring.emissive = colour
        } else {
            const legal = m ? m.legal : []
            for (let i = 0; i < marks.length; i++) {
                marks[i].visible = mine && legal.indexOf(i) >= 0 && i !== game.cursor
                marks[i].emissive = colour
            }
            const p = at(game.cursor)
            ghost.visible = mine
            ghost.position = Qt.vector3d(p.x, p.y, 0.05)
            held = legal.indexOf(game.cursor) >= 0 ? 0.55 : 0.18
            ghost.opacity = held
            ghost.colour = colour
            ghost.emissive = colour
        }
    }

    /// The board cell under the pointer, or -1: where the pointer meets the board, whatever piece it meets there.
    function pointed() {
        const p = pointing()
        const board = entity("Board")
        if (!p || !board)
            return -1
        const local = board.mapPositionFromScene(p.point)
        const size = Arena.BOARDS[which]
        const c = Math.round(local.x / pitch + (size.columns - 1) / 2)
        const r = Math.round((size.rows - 1) / 2 - local.y / pitch)
        return c >= 0 && c < size.columns && r >= 0 && r < size.rows ? r * size.columns + c : -1
    }

    function buildFlux() {
        const w = 7 * pitch, h = 6 * pitch
        piece({ name: "Frame", form: "cube", position: Qt.vector3d(0, 0, -0.24), scale: Qt.vector3d(w + 0.55, h + 0.55, 0.3),
                colour: "#1b2b3b", metalness: 0.3, roughness: 0.62, emissive: "#233445", emissivePower: 0.1 })
        piece({ name: "Plinth", form: "cube", position: Qt.vector3d(0, -h / 2 - 0.42, -0.12),
                scale: Qt.vector3d(w + 1.1, 0.22, 0.8), colour: "#b08a4a", metalness: 0.85, roughness: 0.28,
                emissive: "#c9a45c", emissivePower: 0.06 })
        for (const side of [-1, 1])
            piece({ name: side < 0 ? "Left rim" : "Right rim", form: "cube",
                    position: Qt.vector3d(side * (w / 2 + 0.33), 0, -0.1), scale: Qt.vector3d(0.08, h + 0.6, 0.36),
                    colour: "#b08a4a", metalness: 0.85, roughness: 0.28, emissive: "#c9a45c", emissivePower: 0.12 })
        const made = []
        for (let i = 0; i < 42; i++) {
            const p = at(i)
            made.push(piece({ name: "Cell " + i, form: "cylinder", position: Qt.vector3d(p.x, p.y, -0.05),
                              eulerRotation: Qt.vector3d(90, 0, 0), scale: Qt.vector3d(0.72, 0.08, 0.72),
                              colour: "#09121b", metalness: 0.2, roughness: 0.85, emissive: "#1c2733",
                              emissivePower: 0.12 }))
        }
        cells = made
        ghost = piece({ name: "Held pulse", form: "cylinder", position: Qt.vector3d(0, 0, 0.08),
                        eulerRotation: Qt.vector3d(90, 0, 0), scale: Qt.vector3d(0.62, 0.16, 0.62),
                        colour: colours[1], metalness: 0.45, roughness: 0.22, emissive: colours[1], emissivePower: 0.3 })
        ghost.opacity = 0.7
        ghost.visible = false
        ring = piece({ name: "Landing", form: "cylinder", position: Qt.vector3d(0, 0, -0.06),
                       eulerRotation: Qt.vector3d(90, 0, 0), scale: Qt.vector3d(0.82, 0.02, 0.82),
                       colour: colours[1], metalness: 0, roughness: 0.5, emissive: colours[1], emissivePower: 0.6 })
        ring.visible = false
    }

    function buildReversi() {
        const w = 6 * pitch
        piece({ name: "Frame", form: "cube", position: Qt.vector3d(0, 0, -0.2), scale: Qt.vector3d(w + 0.5, w + 0.5, 0.26),
                colour: "#173238", metalness: 0.3, roughness: 0.62, emissive: "#213c40", emissivePower: 0.1 })
        for (const [name, x, y, sx, sy] of [["Top rim", 0, w / 2 + 0.29, w + 0.66, 0.08],
                                            ["Bottom rim", 0, -w / 2 - 0.29, w + 0.66, 0.08],
                                            ["Left rim", -w / 2 - 0.29, 0, 0.08, w + 0.5],
                                            ["Right rim", w / 2 + 0.29, 0, 0.08, w + 0.5]])
            piece({ name: name, form: "cube", position: Qt.vector3d(x, y, -0.08), scale: Qt.vector3d(sx, sy, 0.32),
                    colour: "#6f9aa3", metalness: 0.7, roughness: 0.45, emissive: "#85c3cf", emissivePower: 0.08 })
        const made = [], marked = []
        for (let i = 0; i < 36; i++) {
            const p = at(i)
            made.push(piece({ name: "Cell " + i, form: "cube", position: Qt.vector3d(p.x, p.y, -0.05),
                              scale: Qt.vector3d(0.86 * pitch, 0.86 * pitch, 0.06), colour: "#183a40", metalness: 0.35,
                              roughness: 0.4, emissive: "#0f2a2f", emissivePower: 0.15 }))
            const mark = piece({ name: "Mark " + i, form: "sphere", position: Qt.vector3d(p.x, p.y, 0),
                                 scale: Qt.vector3d(0.13, 0.13, 0.13), colour: "#0c0e14", metalness: 0, roughness: 0.6,
                                 emissive: colours[1], emissivePower: 0.55 })
            mark.visible = false
            marked.push(mark)
        }
        cells = made
        marks = marked
        ghost = piece({ name: "Held disc", form: "cylinder", position: Qt.vector3d(0, 0, 0.05),
                        eulerRotation: Qt.vector3d(90, 0, 0), scale: Qt.vector3d(0.7, 0.1, 0.7), colour: colours[1],
                        metalness: 0.45, roughness: 0.22, emissive: colours[1], emissivePower: 0.3 })
        ghost.opacity = 0.55
        ghost.visible = false
    }

    onStarted: {
        which = Arena.SCENES[scene.name] ?? "flux"
        game = Arena.create(which)
        pitch = which === "flux" ? 0.92 : 0.95
        if (which === "flux")
            buildFlux()
        else
            buildReversi()
        discs = cells.map(() => null)
        last = piece({ name: "Last move", form: "cylinder", position: Qt.vector3d(0, 0, 0.17),
                       eulerRotation: Qt.vector3d(90, 0, 0), scale: Qt.vector3d(0.2, 0.02, 0.2), colour: "#f3ede1",
                       metalness: 0, roughness: 0.4, emissive: "#f3ede1", emissivePower: 0.5 })
        last.visible = false
        sparks = spawn("Emitter", { name: "Sparks", position: Qt.vector3d(0, 0, 0.2), colour: colours[1], rate: 1,
                                    life: 0.9, size: 0.05, speed: 0.8, spread: 1.4 }, entity("Board"))
        for (const name of buttons) {
            const b = entity(name)
            if (b)
                b.clicked.connect(() => {
                    send(Arena.click(game, name))
                    // Left, or a new match asked for: the board is cleared until one is told.
                    if (!game.match)
                        sync({})
                    aimed()
                    show()
                })
        }
        send(Arena.begin(game))
        show()
    }

    onTold: (message) => {
        sync(Arena.heard(game, message))
        show()
    }

    // The arrows, WASD or the d-pad move the cursor; Space, Enter or the pad's A makes the move; Q and E, or the pad's
    // shoulders, jump between Rift Reversi's moves; 1 to 7 drop a pulse straight into Flux Four's column; the pointer
    // over the board aims and its press moves.
    onPressed: (action) => {
        if (action === "pick") {
            const cell = pointed()
            if (cell >= 0) {
                Arena.point(game, cell)
                send(Arena.move(game))
            }
        } else {
            send(Arena.press(game, action))
        }
        aimed()
        show()
    }

    onStep: {
        const notice = game.notice
        Arena.step(game)
        if (notice !== game.notice || Arena.secondsLeft(game) !== clock)
            show()
    }

    // Only looked at: the pointer aiming, the held pulse bobbing, the held disc breathing, the last move's spark, the
    // winning line pulsing.
    onFrame: (dt) => {
        const now = Input.pointer
        if (now.x !== pointer.x || now.y !== pointer.y) {
            pointer = { x: now.x, y: now.y }
            if (Arena.myTurn(game)) {
                const cell = pointed()
                const before = game.cursor
                if (cell >= 0)
                    Arena.point(game, cell)
                if (game.cursor !== before)
                    aimed()
            }
        }
        const t = elapsed
        if (which === "flux" && ghost.visible && !Look.reducedMotion) {
            const top = at(game.cursor)
            ghost.position = Qt.vector3d(top.x, top.y + pitch * 1.2 + 0.06 * Math.sin(t * 3), 0.08)
        }
        // Rift Reversi's disc breathes, so it reads as one about to be placed.
        if (which === "reversi" && ghost.visible)
            ghost.opacity = Look.reducedMotion ? held : held * (0.75 + 0.25 * Math.sin(t * 3.5))
        burst = Math.max(0, burst - dt * 1.6)
        sparks.rate = 1 + 60 * burst
        for (let k = 0; k < line.length; k++) {
            const d = discs[line[k]]
            if (d)
                d.emissivePower = Look.reducedMotion ? 0.45 : 0.2 + 0.25 * Math.sin(t * 4 - k * 0.6)
        }
    }
}
