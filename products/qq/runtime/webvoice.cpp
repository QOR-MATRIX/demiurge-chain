#include "webvoice.h"

#include <QSoundEffect>

#include <algorithm>

namespace qq {

WebVoice::WebVoice(QObject *parent) : QObject(parent), m_effect(new QSoundEffect(this)) {}

QUrl WebVoice::source() const
{
    return m_effect->source();
}

void WebVoice::setSource(const QUrl &source)
{
    if (source == m_effect->source())
        return;
    m_effect->setSource(source);
    emit sourceChanged();
}

qreal WebVoice::volume() const
{
    return m_effect->volume();
}

void WebVoice::setVolume(qreal volume)
{
    volume = std::clamp(volume, 0.0, 1.0);
    if (qFuzzyCompare(float(volume), m_effect->volume()))
        return;
    m_effect->setVolume(float(volume));
    emit volumeChanged();
}

int WebVoice::loops() const
{
    return m_effect->loopCount();
}

void WebVoice::setLoops(int loops)
{
    if (loops == m_effect->loopCount())
        return;
    m_effect->setLoopCount(loops);
    emit loopsChanged();
}

void WebVoice::play()
{
    m_effect->play();
}

void WebVoice::stop()
{
    m_effect->stop();
}

}  // namespace qq
