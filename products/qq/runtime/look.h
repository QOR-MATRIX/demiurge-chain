// Look: QQ's interface style (DIRECTION P3.8, ADR-105), what a game's screens are drawn in unless a creator says
// otherwise. The faces are the homepage's and ARQADE's (Cinzel for display, Barlow Semi Condensed for everything
// else, both under the SIL Open Font License, carried in the runtime); the colours are the launcher's inks and accent
// with ARQADE's gilt, so a QQ game reads as part of the ecosystem it plays in.
//
// Reduced motion is the person's: on Windows, the system's own "Show animations" setting; anywhere it can be asked of,
// it is. A game's screens appear at once rather than fading when it is on, and a creator's logic can read it.

#pragma once

#include <QColor>
#include <QObject>
#include <QString>
#include <QtQml/qqmlregistration.h>

class QQmlEngine;
class QJSEngine;

namespace qq {

class Look : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    /// The display face, for titles, and the interface face, for everything else: family names, once loaded.
    Q_PROPERTY(QString display READ display CONSTANT)
    Q_PROPERTY(QString ui READ ui CONSTANT)

    Q_PROPERTY(QColor ink READ ink CONSTANT)
    Q_PROPERTY(QColor inkMuted READ inkMuted CONSTANT)
    Q_PROPERTY(QColor accent READ accent CONSTANT)
    Q_PROPERTY(QColor accentBright READ accentBright CONSTANT)
    Q_PROPERTY(QColor surface READ surface CONSTANT)
    Q_PROPERTY(QColor edge READ edge CONSTANT)
    Q_PROPERTY(QColor gilt READ gilt CONSTANT)

    /// Points of a screen 720 tall: an interface is laid out once and drawn the same at every size.
    Q_PROPERTY(double designHeight READ designHeight CONSTANT)

    /// Whether the person asked for less motion. Read from the system when the runtime starts; settable, so a host (the
    /// launcher, ARQADE) can pass on the person's own choice, and a check can set it.
    Q_PROPERTY(bool reducedMotion READ reducedMotion WRITE setReducedMotion NOTIFY reducedMotionChanged)

    /// How long a screen takes to appear or go, in milliseconds: 0 with reduced motion.
    Q_PROPERTY(int fade READ fade NOTIFY reducedMotionChanged)

public:
    explicit Look(QObject *parent = nullptr);

    static Look *create(QQmlEngine *qml, QJSEngine *js);

    QString display() const { return m_display; }
    QString ui() const { return m_ui; }
    QColor ink() const { return QColor(0xf3, 0xed, 0xe1); }
    QColor inkMuted() const { return QColor(0x9b, 0xa2, 0xad); }
    QColor accent() const { return QColor(0xff, 0x6a, 0x00); }
    QColor accentBright() const { return QColor(0xff, 0x91, 0x42); }
    QColor surface() const { return QColor(0x13, 0x18, 0x21); }
    QColor edge() const { return QColor(0x33, 0x3a, 0x47); }
    QColor gilt() const { return QColor(0xc9, 0xa4, 0x5c); }
    double designHeight() const { return 720; }
    bool reducedMotion() const { return m_reducedMotion; }
    void setReducedMotion(bool reduced);
    int fade() const { return m_reducedMotion ? 0 : 180; }

    /// What the system says about motion, where it can be asked: false elsewhere.
    static bool systemPrefersReducedMotion();

signals:
    void reducedMotionChanged();

private:
    QString m_display;
    QString m_ui;
    bool m_reducedMotion = false;
};

}  // namespace qq
