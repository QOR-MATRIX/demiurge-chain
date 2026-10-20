#include "kinematics.h"

#include <algorithm>

namespace qq {

namespace {

// Closer than this is touching, not overlapping: far below a pixel at any scale a game uses.
constexpr double touching = 1e-9;

bool inDepth(const Box &solid, double z)
{
    return z > solid.back && z < solid.front;
}

}  // namespace

Moved sweep2D(const QList<Box> &solids, double x, double y, double z, double halfWidth, double halfHeight, double dx,
              double dy)
{
    Moved m;
    m.x = x;
    m.y = y;
    auto blocks = [&](const Box &s) {
        return inDepth(s, z) && m.x + halfWidth > s.left + touching && m.x - halfWidth < s.right - touching
               && m.y + halfHeight > s.bottom + touching && m.y - halfHeight < s.top - touching;
    };
    if (dx != 0) {
        m.x += dx;
        for (const Box &s : solids) {
            if (!blocks(s))
                continue;
            if (dx > 0) {
                m.x = std::min(m.x, s.left - halfWidth);
                m.right = true;
            } else {
                m.x = std::max(m.x, s.right + halfWidth);
                m.left = true;
            }
        }
    }
    if (dy != 0) {
        m.y += dy;
        for (const Box &s : solids) {
            if (!blocks(s))
                continue;
            if (dy > 0) {
                m.y = std::min(m.y, s.bottom - halfHeight);
                m.up = true;
            } else {
                m.y = std::max(m.y, s.top + halfHeight);
                m.down = true;
            }
        }
    }
    return m;
}

bool overlaps(const Box &a, const Box &b)
{
    return a.right > b.left + touching && a.left < b.right - touching && a.top > b.bottom + touching
           && a.bottom < b.top - touching && a.front > b.back + touching && a.back < b.front - touching;
}

}  // namespace qq
