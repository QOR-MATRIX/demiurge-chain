// The homepage's two faces (site/), carried in the executable: Cinzel for names, Barlow Semi Condensed for words.

#pragma once

#include <QFontDatabase>
#include <QString>

namespace arqade {

inline void loadFonts()
{
    for (const char *face : {"Cinzel.ttf", "BarlowSemiCondensed-Regular.ttf", "BarlowSemiCondensed-Medium.ttf",
                             "BarlowSemiCondensed-SemiBold.ttf"})
        QFontDatabase::addApplicationFont(QStringLiteral(":/qt/qml/ARQADE/fonts/") + QString::fromLatin1(face));
}

}  // namespace arqade
