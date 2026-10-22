#include "look.h"

#include <QFontDatabase>

#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif

namespace qormedia {

QStringList loadFonts()
{
    QStringList families;
    for (const char *face : {"Barlow-Regular.ttf", "Barlow-Medium.ttf", "BarlowSemiCondensed-Regular.ttf",
                             "BarlowSemiCondensed-Medium.ttf", "BarlowSemiCondensed-SemiBold.ttf",
                             "IBMPlexMono-Regular.ttf", "IBMPlexMono-Medium.ttf"}) {
        const int id =
            QFontDatabase::addApplicationFont(QStringLiteral(":/qt/qml/QOR/Media/fonts/") + QString::fromLatin1(face));
        if (id >= 0)
            families << QFontDatabase::applicationFontFamilies(id);
    }
    return families;
}

bool prefersReducedMotion()
{
    // The launcher passes on the person's own Motion setting to what it starts (ADR-108 decision 5).
    const QByteArray asked = qgetenv("QOR_REDUCED_MOTION").trimmed().toLower();
    if (asked == "1" || asked == "true")
        return true;
    if (asked == "0" || asked == "false")
        return false;
#ifdef Q_OS_WIN
    // Settings > Accessibility > Visual effects > Animation effects: off means the person wants less motion.
    BOOL animate = TRUE;
    if (SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &animate, 0))
        return !animate;
#endif
    return false;
}

}  // namespace qormedia
