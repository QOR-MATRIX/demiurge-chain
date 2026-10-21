// Look: QQ's style (ADR-108, DIRECTION P2.9), for QQ Studio and for what a game's screens are drawn in unless a creator
// says otherwise (P3.8). Every value is Demiurge's one look, from docs/design/tokens.json through the generated
// tokens.h: obsidian grounds, parchment inks, ember the one colour of action, gilt the brand's metal. The faces are
// Cinzel for display and Barlow Semi Condensed for the interface, carried in the runtime; Barlow for reading and IBM
// Plex Mono for numbers are carried by the Studio alone, so the Player stays small (elsewhere they are names only, and
// text falls back to the system's faces). All four are under the SIL Open Font License.
//
// Reduced motion is the person's: what the launcher passes on (QOR_REDUCED_MOTION, 1 or 0), or else, on Windows, the
// system's own "Animation effects" setting. Screens and the Studio's controls then appear and change at once, and a
// creator's logic can read it.

#pragma once

#include <QColor>
#include <QObject>
#include <QString>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

class QQmlEngine;
class QJSEngine;

namespace qq {

class Look : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    /// The faces, by role, as family names: display (titles, wordmarks), ui (headings, controls, labels), body (reading)
    /// and mono (numbers and addresses).
    Q_PROPERTY(QString display READ display CONSTANT)
    Q_PROPERTY(QString ui READ ui CONSTANT)
    Q_PROPERTY(QString body READ body CONSTANT)
    Q_PROPERTY(QString mono READ mono CONSTANT)

    /// The grounds, deepest first. `deepest` is the token `void`, a word C++ and JavaScript keep for themselves.
    Q_PROPERTY(QColor deepest READ deepest CONSTANT)
    Q_PROPERTY(QColor base READ base CONSTANT)
    Q_PROPERTY(QColor well READ well CONSTANT)
    Q_PROPERTY(QColor surface READ surface CONSTANT)
    Q_PROPERTY(QColor raised READ raised CONSTANT)
    Q_PROPERTY(QColor edge READ edge CONSTANT)
    Q_PROPERTY(QColor edgeSoft READ edgeSoft CONSTANT)
    /// The inks, brightest first, each 4.5:1 or more on every ground.
    Q_PROPERTY(QColor ink READ ink CONSTANT)
    Q_PROPERTY(QColor inkBody READ inkBody CONSTANT)
    Q_PROPERTY(QColor inkMuted READ inkMuted CONSTANT)
    Q_PROPERTY(QColor inkFaint READ inkFaint CONSTANT)
    /// Ember, the one colour of action, and the ink on an ember or gilt fill.
    Q_PROPERTY(QColor accent READ accent CONSTANT)
    Q_PROPERTY(QColor accentBright READ accentBright CONSTANT)
    Q_PROPERTY(QColor accentDim READ accentDim CONSTANT)
    Q_PROPERTY(QColor onAccent READ onAccent CONSTANT)
    /// Gilt, the brand's metal. `giltDim` is for hairlines, never for text.
    Q_PROPERTY(QColor gilt READ gilt CONSTANT)
    Q_PROPERTY(QColor giltBright READ giltBright CONSTANT)
    Q_PROPERTY(QColor giltDim READ giltDim CONSTANT)
    /// Status, never decoration.
    Q_PROPERTY(QColor ok READ ok CONSTANT)
    Q_PROPERTY(QColor warn READ warn CONSTANT)
    Q_PROPERTY(QColor bad READ bad CONSTANT)

    /// The type scale, in pixels, and tracking, in ems (letter spacing is a size times one).
    Q_PROPERTY(int sizeMicro READ sizeMicro CONSTANT)
    Q_PROPERTY(int sizeCaption READ sizeCaption CONSTANT)
    Q_PROPERTY(int sizeUi READ sizeUi CONSTANT)
    Q_PROPERTY(int sizeBody READ sizeBody CONSTANT)
    Q_PROPERTY(int sizeTitle READ sizeTitle CONSTANT)
    Q_PROPERTY(int sizeHeading READ sizeHeading CONSTANT)
    Q_PROPERTY(int sizeDisplay READ sizeDisplay CONSTANT)
    Q_PROPERTY(int sizeFigure READ sizeFigure CONSTANT)
    Q_PROPERTY(double trackingHeading READ trackingHeading CONSTANT)
    Q_PROPERTY(double trackingLabel READ trackingLabel CONSTANT)
    Q_PROPERTY(double trackingEyebrow READ trackingEyebrow CONSTANT)

    /// Corners: controls and fields, and panels.
    Q_PROPERTY(int radiusControl READ radiusControl CONSTANT)
    Q_PROPERTY(int radiusPanel READ radiusPanel CONSTANT)

    /// Motion, in milliseconds, each 0 with reduced motion; and the one easing, for Easing.BezierSpline.
    Q_PROPERTY(int quick READ quick NOTIFY reducedMotionChanged)
    Q_PROPERTY(int standard READ standard NOTIFY reducedMotionChanged)
    Q_PROPERTY(int enter READ enter NOTIFY reducedMotionChanged)
    Q_PROPERTY(int sheen READ sheen NOTIFY reducedMotionChanged)
    Q_PROPERTY(QVariantList ease READ ease CONSTANT)

    /// The focus ring: its width and halo, in pixels, and its colour.
    Q_PROPERTY(int focusWidth READ focusWidth CONSTANT)
    Q_PROPERTY(int focusHalo READ focusHalo CONSTANT)
    Q_PROPERTY(QColor focus READ focus CONSTANT)

    /// Points of a screen 720 tall: a game's interface is laid out once and drawn the same at every size.
    Q_PROPERTY(double designHeight READ designHeight CONSTANT)

    /// Whether the person asked for less motion: what the launcher passed on, or the system's setting when the runtime
    /// started. Settable, so a host (the launcher, ARQADE) can pass on the person's own choice, and a check can set it.
    Q_PROPERTY(bool reducedMotion READ reducedMotion WRITE setReducedMotion NOTIFY reducedMotionChanged)

    /// How long a game's screen takes to appear or go, in milliseconds: the standard duration, 0 with reduced motion.
    Q_PROPERTY(int fade READ fade NOTIFY reducedMotionChanged)

public:
    explicit Look(QObject *parent = nullptr);

    static Look *create(QQmlEngine *qml, QJSEngine *js);

    QString display() const { return m_display; }
    QString ui() const { return m_ui; }
    QString body() const { return m_body; }
    QString mono() const { return m_mono; }

    QColor deepest() const;
    QColor base() const;
    QColor well() const;
    QColor surface() const;
    QColor raised() const;
    QColor edge() const;
    QColor edgeSoft() const;
    QColor ink() const;
    QColor inkBody() const;
    QColor inkMuted() const;
    QColor inkFaint() const;
    QColor accent() const;
    QColor accentBright() const;
    QColor accentDim() const;
    QColor onAccent() const;
    QColor gilt() const;
    QColor giltBright() const;
    QColor giltDim() const;
    QColor ok() const;
    QColor warn() const;
    QColor bad() const;

    int sizeMicro() const;
    int sizeCaption() const;
    int sizeUi() const;
    int sizeBody() const;
    int sizeTitle() const;
    int sizeHeading() const;
    int sizeDisplay() const;
    int sizeFigure() const;
    double trackingHeading() const;
    double trackingLabel() const;
    double trackingEyebrow() const;
    int radiusControl() const;
    int radiusPanel() const;
    int quick() const;
    int standard() const;
    int enter() const;
    int sheen() const;
    QVariantList ease() const;
    int focusWidth() const;
    int focusHalo() const;
    QColor focus() const;

    double designHeight() const { return 720; }
    bool reducedMotion() const { return m_reducedMotion; }
    void setReducedMotion(bool reduced);
    int fade() const { return standard(); }

    /// What the person asked for about motion: the launcher's word (QOR_REDUCED_MOTION), else the system's where it can
    /// be asked, else false.
    static bool systemPrefersReducedMotion();

signals:
    void reducedMotionChanged();

private:
    QString m_display;
    QString m_ui;
    QString m_body;
    QString m_mono;
    bool m_reducedMotion = false;
};

}  // namespace qq
