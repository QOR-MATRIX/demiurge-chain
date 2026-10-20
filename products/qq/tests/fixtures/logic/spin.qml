// Turns its target about the vertical, steadily: the smallest useful logic, and the one QQ's tests run and rewrite.

import QQ

Logic {
    property real speed: 90  // degrees a second
    onStep: (dt) => target.eulerRotation.y += speed * dt
}
