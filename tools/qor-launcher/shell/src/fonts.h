// Demiurge's faces (ADR-108), carried in the app so the phone and the PC draw the same: Cinzel for the wordmark, Barlow
// Semi Condensed for headings and labels, Barlow for reading, IBM Plex Mono for numbers. docs/design/tokens.mjs copies
// each file to fonts/ from docs/design/fonts/.

#pragma once

#include <QFontDatabase>
#include <QString>
#include <QStringList>

namespace qor {

/// Registers the faces built into the app. Returns the families registered, in the order loaded.
inline QStringList loadFonts()
{
    QStringList families;
    for (const char *face : {"Cinzel.ttf", "Barlow-Regular.ttf", "Barlow-Medium.ttf", "BarlowSemiCondensed-Regular.ttf",
                             "BarlowSemiCondensed-Medium.ttf", "BarlowSemiCondensed-SemiBold.ttf",
                             "IBMPlexMono-Regular.ttf", "IBMPlexMono-Medium.ttf"}) {
        const int id = QFontDatabase::addApplicationFont(QStringLiteral(":/qt/qml/QOR/fonts/") + QString::fromLatin1(face));
        if (id >= 0)
            families << QFontDatabase::applicationFontFamilies(id);
    }
    return families;
}

}  // namespace qor
