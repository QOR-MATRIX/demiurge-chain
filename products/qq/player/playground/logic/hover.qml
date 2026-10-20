// The orb above the plinth: it turns slowly and rises and falls on its own breath.
//
// The playground also keeps how many times it has been opened (P3.16), in its store: the checks read the count, natively
// and in a browser after the page is loaded again. What comes back from a store is checked, since it is the player's.

import QQ

Logic {
    property real base: 0

    // Where a logic file begins: it imports only QQ, so Component.onCompleted is not there to use.
    onStarted: {
        if (target)
            base = target.position.y
        const visits = Number(saved("visits", 0))
        save("visits", (Number.isInteger(visits) && visits > 0 ? visits : 0) + 1)
    }

    onFrame: (dt) => {
        if (!target)
            return
        target.eulerRotation.y += 24 * dt
        target.position.y = base + 0.18 * Math.sin(elapsed * 1.6)
    }
}
