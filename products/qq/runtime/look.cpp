#include "look.h"

#include "tokens.h"

#include <QFile>
#include <QFontDatabase>
#include <QQmlEngine>
#include <QStringList>

#ifdef Q_OS_WIN
#include <windows.h>
#endif

namespace qq {

namespace {

/// Register the faces in `files` that are there (the runtime's resources, or the Studio's), and answer with `family`,
/// the look's own name for them. Not the name a loaded file reports: on Windows Qt names Barlow Semi Condensed "Barlow",
/// the family of every width, and with Barlow itself loaded too (the Studio) "Barlow" draws the wider face. By its own
/// name each face is found, and each weight in it (measured 21 October 2026).
QString loadFace(const QStringList &files, const char *family)
{
    for (const QString &file : files) {
        if (QFile::exists(file))
            QFontDatabase::addApplicationFont(file);
    }
    return QString::fromLatin1(family);
}

QColor rgb(QRgb value)
{
    return QColor(value);
}

}  // namespace

Look::Look(QObject *parent)
    : QObject(parent)
    , m_display(loadFace({QStringLiteral(":/qq/fonts/Cinzel.ttf")}, tokens::fontDisplay))
    , m_ui(loadFace({QStringLiteral(":/qq/fonts/BarlowSemiCondensed-Regular.ttf"),
                     QStringLiteral(":/qq/fonts/BarlowSemiCondensed-SemiBold.ttf")},
                    tokens::fontInterface))
    // Carried by the Studio only (studio/CMakeLists.txt): in the Player these are names, and text falls back.
    , m_body(loadFace({QStringLiteral(":/qq/studio-fonts/Barlow-Regular.ttf"),
                       QStringLiteral(":/qq/studio-fonts/Barlow-Medium.ttf")},
                      tokens::fontBody))
    , m_mono(loadFace({QStringLiteral(":/qq/studio-fonts/IBMPlexMono-Regular.ttf"),
                       QStringLiteral(":/qq/studio-fonts/IBMPlexMono-Medium.ttf")},
                      tokens::fontMono))
    , m_reducedMotion(systemPrefersReducedMotion())
{
}

Look *Look::create(QQmlEngine *, QJSEngine *)
{
    return new Look;
}

QColor Look::deepest() const { return rgb(tokens::colorVoid); }
QColor Look::base() const { return rgb(tokens::colorBase); }
QColor Look::well() const { return rgb(tokens::colorWell); }
QColor Look::surface() const { return rgb(tokens::colorSurface); }
QColor Look::raised() const { return rgb(tokens::colorRaised); }
QColor Look::edge() const { return rgb(tokens::colorEdge); }
QColor Look::edgeSoft() const { return rgb(tokens::colorEdgeSoft); }
QColor Look::ink() const { return rgb(tokens::colorInk); }
QColor Look::inkBody() const { return rgb(tokens::colorInkBody); }
QColor Look::inkMuted() const { return rgb(tokens::colorInkMuted); }
QColor Look::inkFaint() const { return rgb(tokens::colorInkFaint); }
QColor Look::accent() const { return rgb(tokens::colorAccent); }
QColor Look::accentBright() const { return rgb(tokens::colorAccentBright); }
QColor Look::accentDim() const { return rgb(tokens::colorAccentDim); }
QColor Look::onAccent() const { return rgb(tokens::colorOnAccent); }
QColor Look::gilt() const { return rgb(tokens::colorGilt); }
QColor Look::giltBright() const { return rgb(tokens::colorGiltBright); }
QColor Look::giltDim() const { return rgb(tokens::colorGiltDim); }
QColor Look::ok() const { return rgb(tokens::colorOk); }
QColor Look::warn() const { return rgb(tokens::colorWarn); }
QColor Look::bad() const { return rgb(tokens::colorBad); }

int Look::sizeMicro() const { return tokens::sizeMicro; }
int Look::sizeCaption() const { return tokens::sizeCaption; }
int Look::sizeUi() const { return tokens::sizeUi; }
int Look::sizeBody() const { return tokens::sizeBody; }
int Look::sizeTitle() const { return tokens::sizeTitle; }
int Look::sizeHeading() const { return tokens::sizeHeading; }
int Look::sizeDisplay() const { return tokens::sizeDisplay; }
int Look::sizeFigure() const { return tokens::sizeFigure; }
double Look::trackingHeading() const { return tokens::trackingHeading; }
double Look::trackingLabel() const { return tokens::trackingLabel; }
double Look::trackingEyebrow() const { return tokens::trackingEyebrow; }
int Look::radiusControl() const { return tokens::radiusControl; }
int Look::radiusPanel() const { return tokens::radiusPanel; }
int Look::quick() const { return m_reducedMotion ? 0 : tokens::motionQuick; }
int Look::standard() const { return m_reducedMotion ? 0 : tokens::motionStandard; }
int Look::enter() const { return m_reducedMotion ? 0 : tokens::motionEnter; }
int Look::sheen() const { return m_reducedMotion ? 0 : tokens::motionSheen; }
int Look::focusWidth() const { return tokens::focusWidth; }
int Look::focusHalo() const { return tokens::focusHalo; }
QColor Look::focus() const { return rgb(tokens::focusColor); }

QVariantList Look::ease() const
{
    // Easing.BezierSpline takes the two control points and then the end point.
    return {tokens::easeBezier[0], tokens::easeBezier[1], tokens::easeBezier[2], tokens::easeBezier[3], 1.0, 1.0};
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

}  // namespace qq
