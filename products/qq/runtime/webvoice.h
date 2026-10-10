// WebVoice: a sound effect, for a Sound in a browser (Sound.qml says why a browser cannot have spatial audio).
//
// QQ's own type rather than Qt Multimedia's QML SoundEffect, so that a game needs no Qt Multimedia import: that module
// also offers the camera and the microphone, which a confined game may not reach (ADR-087, confinement.h).

#pragma once

#include <QObject>
#include <QUrl>
#include <QtQml/qqmlregistration.h>

class QSoundEffect;

namespace qq {

class WebVoice : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QUrl source READ source WRITE setSource NOTIFY sourceChanged)
    /// From 0 to 1.
    Q_PROPERTY(qreal volume READ volume WRITE setVolume NOTIFY volumeChanged)
    /// How many times it plays; -2 for ever.
    Q_PROPERTY(int loops READ loops WRITE setLoops NOTIFY loopsChanged)

public:
    explicit WebVoice(QObject *parent = nullptr);

    QUrl source() const;
    void setSource(const QUrl &source);
    qreal volume() const;
    void setVolume(qreal volume);
    int loops() const;
    void setLoops(int loops);

    Q_INVOKABLE void play();
    Q_INVOKABLE void stop();

signals:
    void sourceChanged();
    void volumeChanged();
    void loopsChanged();

private:
    QSoundEffect *m_effect;
};

}  // namespace qq
