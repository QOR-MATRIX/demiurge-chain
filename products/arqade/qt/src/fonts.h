// Demiurge's faces (ADR-108), carried in the executable: Cinzel for names and numerals, Barlow Semi Condensed for labels
// and controls, Barlow for reading. docs/design/tokens.mjs copies each file here from docs/design/fonts/.

#pragma once

#include <QFontDatabase>
#include <QString>

namespace arqade {

inline void loadFonts()
{
    for (const char *face : {"Cinzel.ttf", "BarlowSemiCondensed-Regular.ttf", "BarlowSemiCondensed-Medium.ttf",
                             "BarlowSemiCondensed-SemiBold.ttf", "Barlow-Regular.ttf", "Barlow-Medium.ttf"})
        QFontDatabase::addApplicationFont(QStringLiteral(":/qt/qml/ARQADE/fonts/") + QString::fromLatin1(face));
}

}  // namespace arqade
