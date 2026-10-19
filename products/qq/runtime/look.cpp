#include "look.h"

#include <QFontDatabase>
#include <QQmlEngine>
#include <QStringList>

#ifdef Q_OS_WIN
#include <windows.h>
#endif

namespace qq {

namespace {

/// Register a face carried in the runtime's resources (runtime/CMakeLists.txt): its family, or `fallback` if it did
/// not load, so text is still drawn.
QString loadFace(const QStringList &files, const QString &fallback)
{
    QString family;
    for (const QString &file : files) {
        const int id = QFontDatabase::addApplicationFont(file);
        const QStringList families = id >= 0 ? QFontDatabase::applicationFontFamilies(id) : QStringList();
        if (family.isEmpty() && !families.isEmpty())
            family = families.first();
    }
    return family.isEmpty() ? fallback : family;
}

}  // namespace

Look::Look(QObject *parent)
    : QObject(parent)
    , m_display(loadFace({QStringLiteral(":/qq/fonts/Cinzel.ttf")}, QStringLiteral("serif")))
    , m_ui(loadFace({QStringLiteral(":/qq/fonts/BarlowSemiCondensed-Regular.ttf"),
                     QStringLiteral(":/qq/fonts/BarlowSemiCondensed-SemiBold.ttf")},
                    QStringLiteral("sans-serif")))
    , m_reducedMotion(systemPrefersReducedMotion())
{
}

Look *Look::create(QQmlEngine *, QJSEngine *)
{
    return new Look;
}

void Look::setReducedMotion(bool reduced)
{
    if (reduced == m_reducedMotion)
        return;
    m_reducedMotion = reduced;
    emit reducedMotionChanged();
}

bool Look::systemPrefersReducedMotion()
{
#ifdef Q_OS_WIN
    // Settings > Accessibility > Visual effects > Animation effects: off means the person wants less motion.
    BOOL animate = TRUE;
    if (SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &animate, 0))
        return !animate;
#endif
    return false;
}

}  // namespace qq
