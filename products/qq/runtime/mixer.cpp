#include "mixer.h"

#include <QFile>
#include <QFileInfo>
#include <QtEndian>

#include <algorithm>

namespace qq {

Mixer::Mixer(QObject *parent) : QObject(parent)
{
    m_levels.insert(QStringLiteral("master"), 1.0);
}

double Mixer::level(const QString &group) const
{
    return m_levels.value(group, 1.0).toDouble();
}

void Mixer::setLevel(const QString &group, double level)
{
    if (group.isEmpty())
        return;
    level = std::clamp(level, 0.0, 1.0);
    if (m_levels.contains(group) && m_levels.value(group).toDouble() == level)
        return;
    m_levels.insert(group, level);
    emit levelsChanged();
}

double Mixer::gain(const QString &group) const
{
    return level(QStringLiteral("master")) * (group == QStringLiteral("master") ? 1.0 : level(group));
}

void Mixer::reset()
{
    QVariantMap levels;
    levels.insert(QStringLiteral("master"), 1.0);
    if (levels == m_levels)
        return;
    m_levels = levels;
    emit levelsChanged();
}

double Mixer::lengthOf(const QUrl &sound)
{
    const QString path = sound.isLocalFile() ? sound.toLocalFile()
                       : sound.scheme() == QStringLiteral("qrc") ? QStringLiteral(":") + sound.path()
                                                                  : QString();
    if (path.isEmpty())
        return 0;
    const QFileInfo info(path);
    if (const auto it = m_lengths.constFind(sound);
        it != m_lengths.cend() && it->size == info.size() && it->modified == info.lastModified())
        return it->seconds;
    double seconds = 0;
    QFile f(path);
    if (f.open(QIODevice::ReadOnly))
        seconds = wavLength(f.read(4096));
    m_lengths.insert(sound, Length{info.size(), info.lastModified(), seconds});
    return seconds;
}

double wavLength(const QByteArray &header)
{
    // RIFF, WAVE, then chunks: "fmt " says the bytes a second (its ninth to twelfth bytes), "data" how many bytes.
    if (header.size() < 12 || !header.startsWith("RIFF") || header.mid(8, 4) != "WAVE")
        return 0;
    quint32 bytesPerSecond = 0;
    qsizetype at = 12;
    while (at + 8 <= header.size()) {
        const QByteArray id = header.mid(at, 4);
        const quint32 size = qFromLittleEndian<quint32>(header.constData() + at + 4);
        if (id == "fmt " && at + 8 + 12 <= header.size())
            bytesPerSecond = qFromLittleEndian<quint32>(header.constData() + at + 8 + 8);
        if (id == "data")
            return bytesPerSecond > 0 ? double(size) / bytesPerSecond : 0;
        at += 8 + size + (size & 1);  // chunks are padded to an even size
    }
    return 0;
}

}  // namespace qq
