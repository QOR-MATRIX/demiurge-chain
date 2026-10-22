// Demiurge's one look in QOR Media (ADR-108): the faces it carries, and the person's Motion setting.
//
// The faces are copied here from docs/design/fonts/ by docs/design/tokens.mjs and built into the executable; Theme asks
// for each by its family name. The Motion setting is the launcher's (QOR_REDUCED_MOTION, which host::launch gives
// every product it starts) or, started by hand, the system's.

#pragma once

#include <QStringList>

namespace qormedia {

/// Registers the faces built into the executable. Returns the families registered, in the order loaded.
QStringList loadFonts();

/// Whether the person asked for less motion: QOR_REDUCED_MOTION when the launcher passed it, otherwise Windows' own
/// "Animation effects" switch, otherwise no.
bool prefersReducedMotion();

}  // namespace qormedia
