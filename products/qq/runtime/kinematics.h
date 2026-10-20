// QQ's own kinematics (DIRECTION P3.11, ADR-105 decision 6): boxes moved against boxes, in doubles and nothing else, so
// the same moves give the same places on every machine and in a replay with no physics engine. A rewarded outcome may
// depend on these, never on PhysX.
//
// A mover is a box about its centre. sweep2D moves it across, then up or down, and stops it at the first solid in its
// way on each axis. Touching is not overlapping: a box resting on a floor, its foot exactly on the floor's top, is free
// to walk along it. Moves are small (a tick's worth), so nothing thinner than a tick's travel is crossed.

#pragma once

#include <QList>

namespace qq {

/// An axis-aligned box: x from left to right, y from bottom to top, z from back to front.
struct Box {
    double left = 0, right = 0, bottom = 0, top = 0, back = 0, front = 0;
};

/// Where a sweep left the mover, and what it met on each side.
struct Moved {
    double x = 0, y = 0;
    bool left = false, right = false, down = false, up = false;
};

/// Move a box `halfWidth` by `halfHeight` about (x, y), at depth z, by (dx, dy) among `solids`.
Moved sweep2D(const QList<Box> &solids, double x, double y, double z, double halfWidth, double halfHeight, double dx,
              double dy);

/// Whether two boxes overlap (touching is not overlapping).
bool overlaps(const Box &a, const Box &b);

}  // namespace qq
