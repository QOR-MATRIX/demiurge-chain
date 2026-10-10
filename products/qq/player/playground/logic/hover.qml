// The orb above the plinth: it turns slowly and rises and falls on its own breath.

import QQ

Logic {
    property real base: 0

    Component.onCompleted: if (target) base = target.position.y

    onFrame: (dt) => {
        if (!target)
            return
        target.eulerRotation.y += 24 * dt
        target.position.y = base + 0.18 * Math.sin(elapsed * 1.6)
    }
}
