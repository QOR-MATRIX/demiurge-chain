// Which GPU QQ draws with. On a computer with two (a laptop's integrated graphics and its discrete card) Qt takes the
// first the system lists, which is the slower one; a game wants the faster. Called first thing in main(), before the
// application exists.

#pragma once

#include <QString>

namespace qq {

/// Draw with the high-performance GPU, if there is a choice. Leaves alone a choice already made in the environment
/// (QT_D3D_ADAPTER_INDEX). The adapter chosen, for a log line, or empty if nothing was changed.
QString preferHighPerformanceGpu();

}  // namespace qq
