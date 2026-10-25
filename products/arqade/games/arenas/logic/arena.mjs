// Flux Four and Rift Reversi (DIRECTION P7.24, ADR-102), ARQADE's two arenas rebuilt in QQ: two people play one board
// on ARQADE's server, which keeps every match and decides every move (lib/arena-store.ts and lib/arena-engine.ts), as
// it does for the website's arcade. A QQ game reaches no server of its own, so the arena asks its host, ARQADE's app,
// for what only the server knows and does, and draws what the host tells it (ADR-117).
//
// This module is the game's side of that conversation, the one place it lives: what the game asks for each thing a
// player does, what it makes of each answer, and what its screens say. It decides no move. A move it asks for is one
// the server listed as legal, and the server checks it again; a board it draws is the one the server sent. Plain
// JavaScript with no clock of its own and no chance, so a run QQ kept (the host's answers are in it, ADR-117) replays
// here in Node to the same state, and to the same outcome, which is the server's verdict on the match.
//
// What the game asks (each an object with `ask`):
//
//   lobby                            the arena's open matches, and whether the player is signed in, now and as they change
//   create                           a new match, its lobby open for a challenger
//   join    { id }                   take the other seat of someone's open match
//   open    { id }                   the player's own match again, after leaving it
//   move    { id, cell, revision }   a move: Flux Four's column, Rift Reversi's cell, at the board the player last saw
//   resign  { id, revision }         concede the match
//   cancel  { id, revision }         close one's own lobby before anyone joins
//   leave                            stop following the match, and hear the lobby again
//
// What the host tells (one of):
//
//   { lobby: [{ id, game, status, alias, seat }], signedIn, why }   the live arcade's matches; why, if not signed in
//   { match }                         the followed match as the server keeps it (its snapshot)
//   { refused }                       why something asked for was not done, in words for the player

/** The arenas, by the scene that plays each. */
export const SCENES = { "flux-four": "flux", "rift-reversi": "reversi" }

/** Each arena's board: columns and rows, as the server numbers its cells (row by row from the top). */
export const BOARDS = {
    flux: { name: "FLUX FOUR", columns: 7, rows: 6 },
    reversi: { name: "RIFT REVERSI", columns: 6, rows: 6 },
}

/** How long a notice stays, in steps of 1/60 s. */
export const NOTICE_STEPS = 300

/** What the game asks, and while it is unanswered asks nothing more. */
const ASKS = ["create", "join", "open", "move", "resign", "cancel"]
/** What brings a match to follow. */
const FINDING = ["create", "join", "open"]

function whole(value, fallback = 0) {
    return Number.isInteger(value) ? value : fallback
}

function words(value, longest = 200) {
    return typeof value === "string" ? value.slice(0, longest) : ""
}

function player(value) {
    return value && typeof value === "object" ? words(value.name, 60) : ""
}

/** A new arena of `game`, "flux" or "reversi", in its lobby. */
export function create(game) {
    if (!BOARDS[game])
        throw new Error("There is no arena called " + game + ".")
    return {
        game,
        steps: 0,
        signedIn: false,
        why: "",
        // Someone else's open matches, newest first, and the player's own match still waiting or under way.
        open: [],
        mine: null,
        // The match followed, as the server last told it, and the step it was told at.
        match: null,
        heardAt: 0,
        // What was asked and is not yet answered.
        waiting: "",
        notice: "",
        noticeAt: 0,
        // The move the player is about to make: Flux Four's column, Rift Reversi's cell.
        cursor: game === "flux" ? 3 : 14,
        // Concede asked once; asked again, it is done.
        confirming: false,
        // The step a screen button was last clicked at.
        clickedAt: -1,
    }
}

/** The arena's board as a snapshot gives it, or null if it is not one. */
export function readMatch(game, m) {
    const board = BOARDS[game]
    const s = m && typeof m === "object" && m.state && typeof m.state === "object" ? m.state : null
    const size = board.columns * board.rows
    if (!s || m.game !== game || typeof m.id !== "string" || !/^[0-9a-f-]{36}$/.test(m.id) || !Array.isArray(s.cells)
            || s.cells.length !== size)
        return null
    const cell = (v) => Number.isInteger(v) && v >= 0 && v < size
    return {
        id: m.id,
        status: ["waiting", "active", "done", "cancelled"].includes(m.status) ? m.status : "done",
        revision: whole(m.revision),
        moves: whole(m.moves),
        seat: m.seat === 1 || m.seat === 2 ? m.seat : 0,
        cells: s.cells.map(v => v === 1 || v === 2 ? v : 0),
        turn: s.turn === 2 ? 2 : 1,
        winner: s.winner === 0 || s.winner === 1 || s.winner === 2 ? s.winner : null,
        placed: cell(s.placed) ? s.placed : -1,
        flipped: Array.isArray(s.flipped) ? s.flipped.filter(cell) : [],
        legal: Array.isArray(m.legal) ? m.legal.filter(Number.isInteger) : [],
        host: player(m.host),
        guest: player(m.guest),
        // Milliseconds left for the move, as the server counted them when it answered.
        left: Math.max(0, whole(m.deadline) - whole(m.serverNow)),
    }
}

/** Whether it is the player's move now. */
export function myTurn(state) {
    const m = state.match
    return m !== null && m.status === "active" && m.seat > 0 && m.turn === m.seat
}

function settleCursor(state) {
    const m = state.match
    if (!myTurn(state) || m.legal.length === 0 || m.legal.includes(state.cursor))
        return
    // The legal move nearest where the player was looking.
    const board = BOARDS[state.game]
    const at = (i) => state.game === "flux" ? [i, 0] : [i % board.columns, Math.floor(i / board.columns)]
    const [cx, cy] = at(state.cursor)
    let best = m.legal[0]
    let far = Infinity
    for (const i of m.legal) {
        const [x, y] = at(i)
        const d = (x - cx) * (x - cx) + (y - cy) * (y - cy)
        if (d < far) {
            far = d
            best = i
        }
    }
    state.cursor = best
}

/**
 * What the host told. What changed, for the drawing: { took (a match was taken in), moved (a disc newly placed),
 * began (the match became active), ended (it finished) }.
 */
export function heard(state, message) {
    const changed = { took: false, moved: false, began: false, ended: false }
    if (!message || typeof message !== "object")
        return changed
    if (Array.isArray(message.lobby)) {
        state.signedIn = message.signedIn === true
        state.why = words(message.why)
        const rows = message.lobby.filter(r => r && typeof r === "object" && r.game === state.game
                                               && typeof r.id === "string")
        state.open = rows.filter(r => r.status === "waiting" && r.seat === 0)
                         .slice(0, 30).map(r => ({ id: r.id, host: words(r.alias, 60) }))
        const own = rows.find(r => (r.seat === 1 || r.seat === 2) && (r.status === "waiting" || r.status === "active"))
        state.mine = own ? { id: own.id, status: own.status } : null
    }
    if (message.match !== undefined) {
        const m = readMatch(state.game, message.match)
        const before = state.match
        const finding = FINDING.includes(state.waiting)
        // Only the match followed, and never an older board than the one drawn.
        const ours = m !== null && (before ? m.id === before.id || finding : finding)
        if (ours && !(before && m.id === before.id && m.revision < before.revision)) {
            const same = before !== null && before.id === m.id
            state.match = m
            state.heardAt = state.steps
            if (!same || m.revision > before.revision)
                state.waiting = ""
            if (m.status !== "active")
                state.confirming = false
            changed.took = true
            changed.moved = same && m.revision > before.revision && m.placed >= 0 && before.cells[m.placed] === 0
                            && m.cells[m.placed] !== 0
            changed.began = m.status === "active" && (!same || before.status !== "active")
            changed.ended = m.status === "done" && (!same || before.status !== "done")
            settleCursor(state)
        }
    }
    if (typeof message.refused === "string") {
        state.waiting = ""
        state.notice = words(message.refused)
        state.noticeAt = state.steps
    }
    return changed
}

/** One step of 1/60 s: the clock runs and a notice fades. */
export function step(state) {
    state.steps += 1
    if (state.notice !== "" && state.steps - state.noticeAt >= NOTICE_STEPS)
        state.notice = ""
}

/** Whole seconds left for the move, counted down from the server's answer. */
export function secondsLeft(state) {
    const m = state.match
    if (!m || m.status !== "active")
        return 0
    return Math.max(0, Math.ceil((m.left - (state.steps - state.heardAt) * 1000 / 60) / 1000))
}

function ask(state, what, fields) {
    state.waiting = what
    return Object.assign({ ask: what }, fields)
}

/** What the game asks as it starts. */
export function begin(state) {
    return { ask: "lobby" }
}

/** A new match of this arena: from the lobby, or a rematch from a finished one. */
export function newMatch(state) {
    if (state.waiting !== "" || !state.signedIn || (state.match && state.match.status !== "done"
                                                    && state.match.status !== "cancelled"))
        return null
    state.match = null
    return ask(state, "create")
}

/** The newest open match of someone else's: its other seat taken. */
export function join(state) {
    if (state.waiting !== "" || !state.signedIn || state.match || state.open.length === 0)
        return null
    return ask(state, "join", { id: state.open[0].id })
}

/** The player's own match, waiting or under way, followed again. */
export function returnTo(state) {
    if (state.waiting !== "" || state.match || !state.mine)
        return null
    return ask(state, "open", { id: state.mine.id })
}

/** The move under the cursor, if it is the player's move and a legal one. */
export function move(state) {
    const m = state.match
    if (state.waiting !== "" || !myTurn(state) || !m.legal.includes(state.cursor) || state.confirming)
        return null
    return ask(state, "move", { id: m.id, cell: state.cursor, revision: m.revision })
}

/** Concede: asked once, it asks to be sure; asked again (sure), it is done. */
export function concede(state, sure) {
    const m = state.match
    if (state.waiting !== "" || !m || m.status !== "active" || m.seat === 0)
        return null
    if (!sure) {
        state.confirming = true
        return null
    }
    state.confirming = false
    return ask(state, "resign", { id: m.id, revision: m.revision })
}

/** Not conceding after all. */
export function keepPlaying(state) {
    state.confirming = false
}

/** One's own lobby closed before anyone joins. */
export function closeLobby(state) {
    const m = state.match
    if (state.waiting !== "" || !m || m.status !== "waiting" || m.seat !== 1)
        return null
    return ask(state, "cancel", { id: m.id, revision: m.revision })
}

/** Back to the lobby. A match under way goes on without the player watching: its clock still runs. */
export function leave(state) {
    state.match = null
    state.waiting = ""
    state.confirming = false
    return { ask: "leave" }
}

/** The cursor moved by the arrows, the d-pad or the stick: Flux Four's to the next column there is room in, Rift
 *  Reversi's one cell, kept on the board. */
export function aim(state, dx, dy) {
    const board = BOARDS[state.game]
    if (state.game === "flux") {
        if (dx === 0)
            return
        const m = state.match
        for (let c = state.cursor + dx; c >= 0 && c < board.columns; c += dx) {
            if (!m || !myTurn(state) || m.legal.includes(c)) {
                state.cursor = c
                return
            }
        }
        return
    }
    const x = Math.max(0, Math.min(board.columns - 1, state.cursor % board.columns + dx))
    const y = Math.max(0, Math.min(board.rows - 1, Math.floor(state.cursor / board.columns) + dy))
    state.cursor = y * board.columns + x
}

/** Rift Reversi's cursor to the next legal move, or the one before (`by` -1), in the board's order. */
export function cycle(state, by) {
    const m = state.match
    if (!myTurn(state) || m.legal.length === 0)
        return
    const legal = m.legal.slice().sort((a, b) => a - b)
    const at = legal.indexOf(state.cursor)
    state.cursor = legal[((at < 0 ? (by > 0 ? -1 : 0) : at) + by + legal.length) % legal.length]
}

/** The cursor on a cell the pointer is over: Flux Four takes its column. */
export function point(state, cell) {
    const board = BOARDS[state.game]
    if (!Number.isInteger(cell) || cell < 0 || cell >= board.columns * board.rows)
        return
    state.cursor = state.game === "flux" ? cell % board.columns : cell
}

/** Flux Four's cell a pulse dropped in `column` comes to rest in, or -1 if the column is full. */
export function landing(cells, column) {
    for (let r = BOARDS.flux.rows - 1; r >= 0; r--)
        if (cells[r * BOARDS.flux.columns + column] === 0)
            return r * BOARDS.flux.columns + column
    return -1
}

/** Flux Four's winning line: the four cells (or more) in a row of `seat`'s, or none. */
export function fourLine(cells, seat) {
    const { columns, rows } = BOARDS.flux
    for (const [dx, dy] of [[1, 0], [0, 1], [1, 1], [1, -1]]) {
        for (let y = 0; y < rows; y++) {
            for (let x = 0; x < columns; x++) {
                const line = []
                for (let k = 0; k < 4; k++) {
                    const cx = x + dx * k, cy = y + dy * k
                    if (cx < 0 || cx >= columns || cy < 0 || cy >= rows || cells[cy * columns + cx] !== seat)
                        break
                    line.push(cy * columns + cx)
                }
                if (line.length === 4)
                    return line
            }
        }
    }
    return []
}

/** Each seat's discs on the board: [the host's, the guest's]. */
export function counts(cells) {
    let a = 0, b = 0
    for (const v of cells) {
        if (v === 1)
            a += 1
        else if (v === 2)
            b += 1
    }
    return [a, b]
}

/** Which screen shows: lobby, waiting, match, result or closed. */
export function screen(state) {
    const m = state.match
    if (!m)
        return "lobby"
    return m.status === "waiting" ? "waiting" : m.status === "active" ? "match" : m.status === "done" ? "result" : "closed"
}

/** The words the screens show now. */
export function say(state) {
    const m = state.match
    const board = BOARDS[state.game]
    const out = {
        name: board.name,
        status: "",
        join: state.open.length > 0 ? "Join " + (state.open[0].host || "a challenger") : "",
        canJoin: state.signedIn && !m && state.waiting === "" && state.open.length > 0,
        canReturn: !m && state.waiting === "" && state.mine !== null,
        canCreate: state.signedIn && !m && state.waiting === "",
        waiting: "",
        host: "",
        guest: "",
        turn: "",
        clock: "",
        result: "",
        line: "",
        notice: state.notice,
    }
    if (!m) {
        out.status = state.waiting === "create" ? "Opening your lobby…"
                   : state.waiting === "join" ? "Joining…"
                   : state.waiting === "open" ? "Returning to your match…"
                   : !state.signedIn ? (state.why || "Sign in to the launcher with your QOR ID to play the arenas.")
                   : state.open.length === 1 ? "1 open lobby"
                   : state.open.length + " open lobbies"
        return out
    }
    const tally = state.game === "reversi" ? counts(m.cells) : null
    out.host = (m.host || "Player 1") + (m.seat === 1 ? "  (you)" : "") + (tally ? "   " + tally[0] : "")
    out.guest = (m.guest || "Waiting for a challenger") + (m.seat === 2 ? "  (you)" : "") + (tally && m.guest ? "   " + tally[1] : "")
    if (m.status === "waiting") {
        out.waiting = m.seat === 1 ? "Your lobby is open. Another player can join it from ARQADE or the website."
                                   : "A challenger is waiting on the other side."
        return out
    }
    if (m.status === "active") {
        out.turn = m.seat === 0 ? "SPECTATING" : myTurn(state) ? "YOUR MOVE" : "THEIR MOVE"
        out.clock = secondsLeft(state) + " s"
        if (state.waiting === "move")
            out.turn = "SENDING YOUR MOVE…"
        return out
    }
    if (m.status === "done") {
        const won = m.seat > 0 && m.winner === m.seat
        out.result = m.winner === 0 ? "DRAW" : won ? "VICTORY" : "MATCH COMPLETE"
        out.line = m.winner === 0 ? "Balance restored." : won ? "Beautifully played."
                 : m.seat === 0 ? (m.winner === 1 ? m.host : m.guest) + " won." : "Another world awaits."
        if (tally)
            out.line += "   " + tally[0] + " to " + tally[1]
    }
    return out
}

/** What the run came to: the match it played and the server's verdict, for the player's seat. */
export function outcome(state) {
    const m = state.match
    if (!m)
        return { match: "", seat: 0, result: "", moves: 0 }
    const result = m.status === "cancelled" ? "cancelled"
                 : m.status !== "done" ? ""
                 : m.winner === 0 ? "drawn"
                 : m.seat === 0 ? "watched"
                 : m.winner === m.seat ? "won" : "lost"
    return { match: m.id, seat: m.seat, result, moves: m.moves }
}

/**
 * A QQ run of the arena `game` played again: { qqRun: 1, ticks, edges, clicks, told }, as QQ writes it. Each tick
 * takes its clicks, then what the host told, then its presses, then steps, as QQ's scene does; a press of the pointer
 * on the board is not in the run (where the pointer was is the window's), and the move it asked for is in the
 * server's answer. Its outcome, which is the one the game reached if the run is the game's.
 */
export function replay(run, game) {
    if (!run || run.qqRun !== 1 || !Number.isInteger(run.ticks) || run.ticks < 0)
        throw new Error("This replays QQ runs of format 1.")
    const edges = new Map((run.edges || []).map(([tick, list]) => [tick, list]))
    const clicks = new Map((run.clicks || []).map(([tick, list]) => [tick, list]))
    const told = new Map((run.told || []).map(([tick, list]) => [tick, list]))
    const state = create(game)
    for (let tick = 0; tick < run.ticks; tick++) {
        for (const button of clicks.get(tick) || [])
            click(state, button)
        for (const message of told.get(tick) || [])
            heard(state, message)
        for (const [what, action] of edges.get(tick) || [])
            if (what === "pressed")
                press(state, action)
        step(state)
    }
    return { state, outcome: outcome(state) }
}

/** A screen button, by its name in the scene: what it asks, or null. */
export function click(state, button) {
    state.clickedAt = state.steps
    switch (button) {
    case "Create": return newMatch(state)
    case "Join": return join(state)
    case "Return": return returnTo(state)
    case "Close lobby": return closeLobby(state)
    case "Concede": return concede(state, false)
    case "Concede match": return concede(state, true)
    case "Keep playing": keepPlaying(state); return null
    case "Rematch": return newMatch(state)
    case "Leave waiting":
    case "Leave match":
    case "Leave result":
    case "Find another": return leave(state)
    default: return null
    }
}

/** An action pressed, by its name in the scene: what it asks, or null. */
export function press(state, action) {
    switch (action) {
    case "left": aim(state, -1, 0); return null
    case "right": aim(state, 1, 0); return null
    case "up": aim(state, 0, -1); return null
    case "down": aim(state, 0, 1); return null
    case "next": cycle(state, 1); return null
    case "previous": cycle(state, -1); return null
    // Space or Enter that pressed a screen button with the keyboard's focus is the button's: it comes at the same tick.
    case "choose": return state.clickedAt === state.steps ? null : move(state)
    // The pointer's press: the game's logic reads which cell it is on, which a run does not keep.
    case "pick": return null
    default: {
        // Flux Four's columns, 1 to 7: the cursor there and the pulse dropped.
        const column = /^column([1-7])$/.exec(action)
        if (column && state.game === "flux") {
            state.cursor = Number(column[1]) - 1
            return move(state)
        }
        return null
    }
    }
}
