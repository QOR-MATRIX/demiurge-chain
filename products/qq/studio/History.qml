// The Studio's undo and redo (DIRECTION P3.15): the scene's canonical text after each edit, kept in order. An edit is
// whatever ends in the window's refresh() — a field set in the inspector, a handle let go, an entity added, duplicated
// or deleted, a tool the agent used — and undo puts the scene back to the text before it. Because the text is the
// scene, nothing an edit does can escape it, and a step costs a few kilobytes.
//
// Edits of the same field in quick succession (a slider dragged, a name typed) are one step: each note carries a key,
// and a note with the key of the one before, within `mergeWithin` milliseconds, replaces it.

import QtQml

QtObject {
    id: history

    /// The texts, oldest first, and which of them the scene is now.
    property var texts: []
    property int at: -1
    /// The most steps kept: the oldest go first.
    property int limit: 200
    property int mergeWithin: 1200

    readonly property bool canUndo: at > 0
    readonly property bool canRedo: at >= 0 && at < texts.length - 1

    property string _lastKey: ""
    property double _lastTime: 0

    /// Start again from `text`: a scene opened, made or imported.
    function reset(text) {
        texts = [text]
        at = 0
        _lastKey = ""
    }

    /// The scene is now `text`, after an edit named by `key` ("" for one that is never merged).
    function note(text, key) {
        if (at < 0) {
            reset(text)
            return
        }
        if (text === texts[at])
            return
        const now = Date.now()
        const kept = texts.slice(0, at + 1)
        if (key !== "" && key === _lastKey && now - _lastTime < mergeWithin && at > 0 && at === texts.length - 1) {
            kept[at] = text
        } else {
            kept.push(text)
            if (kept.length > limit)
                kept.splice(0, kept.length - limit)
        }
        texts = kept
        at = kept.length - 1
        _lastKey = key
        _lastTime = now
    }

    /// The scene is now `text` without an edit (saving named it and brought its files in): the step it is at becomes that.
    function amend(text) {
        if (at < 0) {
            reset(text)
            return
        }
        const kept = texts.slice()
        kept[at] = text
        texts = kept
        _lastKey = ""
    }

    /// The text to go back to, or "" if there is none.
    function undo() {
        if (!canUndo)
            return ""
        at -= 1
        _lastKey = ""
        return texts[at]
    }

    /// The text to go forward to again, or "".
    function redo() {
        if (!canRedo)
            return ""
        at += 1
        _lastKey = ""
        return texts[at]
    }
}
