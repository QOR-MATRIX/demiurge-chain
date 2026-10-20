// WebVoice: a sound for a web page, which Qt's spatial audio cannot sound in (Sound.qml says why): a Sound, a one-shot
// from logic (Voices.qml) or a Music track, in a browser.
//
// In a browser it is the page's own Web Audio (P3.13): the file decoded once, then played through a gain and a stereo
// panner, so a sound off to one side is heard on that side; a browser with no stereo panner plays it unpanned. A page
// may make no sound until the person has touched it or pressed a key, so the first touch or key starts the page's
// audio. Elsewhere (natively, where it is only built and checked) it is Qt Multimedia's QSoundEffect, unpanned.
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
    /// From -1 (all left) to 1 (all right); 0 in the middle.
    Q_PROPERTY(qreal pan READ pan WRITE setPan NOTIFY panChanged)
    /// How many times it plays; -2 for ever.
    Q_PROPERTY(int loops READ loops WRITE setLoops NOTIFY loopsChanged)

public:
    explicit WebVoice(QObject *parent = nullptr);
    ~WebVoice() override;

    QUrl source() const { return m_source; }
    void setSource(const QUrl &source);
    qreal volume() const { return m_volume; }
    void setVolume(qreal volume);
    qreal pan() const { return m_pan; }
    void setPan(qreal pan);
    int loops() const { return m_loops; }
    void setLoops(int loops);

    Q_INVOKABLE void play();
    Q_INVOKABLE void stop();

signals:
    void sourceChanged();
    void volumeChanged();
    void panChanged();
    void loopsChanged();

private:
    void apply();

    QUrl m_source;
    qreal m_volume = 1;
    qreal m_pan = 0;
    int m_loops = 1;
#ifdef Q_OS_WASM
    int m_id = 0;  // the page's voice
#else
    QSoundEffect *m_effect = nullptr;
#endif
};

}  // namespace qq
