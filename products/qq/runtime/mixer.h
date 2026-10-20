// The Mixer: how loud each group of a game's sounds is (DIRECTION P3.13, ADR-105).
//
// Every sound belongs to a group: music, effects, voice, ui, or any a game names. What is heard is the sound's own
// volume, times its group's level, times the master level, so a player's options screen turns all the music down with
// one setting and leaves the effects alone. Levels go from 0 to 1, and a group never set is at 1.
//
// It also measures a sound file's length, so a sound played once from logic is put away when it has finished: Qt's
// spatial sounds do not say when they have.
//
// It is one for the whole program, so the Studio resets it when play stops: a level a game set is the game's, and goes
// with its play, as everything else of it does.

#pragma once

#include <QDateTime>
#include <QHash>
#include <QObject>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

namespace qq {

class Mixer : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    /// Every group set so far, by name, master among them: { "master": 1, "music": 0.5, … }.
    Q_PROPERTY(QVariantMap levels READ levels NOTIFY levelsChanged)

public:
    explicit Mixer(QObject *parent = nullptr);

    QVariantMap levels() const { return m_levels; }

    /// A group's level, from 0 to 1 (1 for a group never set).
    Q_INVOKABLE double level(const QString &group) const;
    /// Set a group's level, held to 0 to 1.
    Q_INVOKABLE void setLevel(const QString &group, double level);
    /// What a sound in `group` is multiplied by: its group's level times the master level.
    Q_INVOKABLE double gain(const QString &group) const;
    /// Every group back at 1, as before any game set one.
    Q_INVOKABLE void reset();
    /// A sound file's length in seconds, read from its header (WAV), or 0 when it cannot be told.
    Q_INVOKABLE double lengthOf(const QUrl &sound);

signals:
    void levelsChanged();

private:
    QVariantMap m_levels;
    /// Lengths measured, with the size and time of the file measured: a file replaced under the same name is measured
    /// again.
    struct Length
    {
        qint64 size = -1;
        QDateTime modified;
        double seconds = 0;
    };
    QHash<QUrl, Length> m_lengths;
};

/// A WAV file's length in seconds from its header: its data's size over its bytes a second; 0 if it is not one.
double wavLength(const QByteArray &header);

}  // namespace qq
