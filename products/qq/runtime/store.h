// A game's store (DIRECTION P3.16, ADR-105 decision 7): the one place a game keeps anything from one play to the next,
// and the one thing written for it that the lockdown allows (ADR-087 decision 2).
//
// A game reaches its store only through its logic (Logic.saved, save and forget) and never names a file: the runtime
// keeps it, one per game, as a small JSON object, in QQ's saves folder on this computer (a file named for the game) and
// in the page's local storage in a browser. A game is its package, its project folder or the bundled game's resource
// folder; confined, it is always the package the engine is confined to, so no game can name another's store. Moving a
// project's folder starts its store afresh.
//
// It holds numbers, text, true and false, null, and lists and objects of them, at most 64 KiB written out, under keys of
// 1 to 100 characters. Anything else is refused, and the game is told why (`problem`).
//
// A replay neither reads nor writes it (P3.11): a run's record keeps the store as the run found it, and the replay plays
// from that copy, in memory. A store is the player's own, on their machine, where they can read and change it, so
// nothing rewarded may rest on it (ADR-102 decision 5): a rewarded outcome is checked by replaying the run.

#pragma once

#include <QJSValue>
#include <QJsonObject>
#include <QObject>
#include <QStringList>
#include <QTimer>
#include <QUrl>
#include <QVariant>

namespace qq {

class GameStore : public QObject
{
    Q_OBJECT

    /// Which game this is ("orb-run-1f0c…"), or empty for a replay's copy.
    Q_PROPERTY(QString game READ game CONSTANT)
    /// Whether it is kept from one play to the next; a replay's copy is not.
    Q_PROPERTY(bool kept READ kept CONSTANT)
    /// Its keys, and its size as written, in bytes.
    Q_PROPERTY(QStringList keys READ keys NOTIFY changed)
    Q_PROPERTY(int size READ size NOTIFY changed)
    /// Why the last thing asked of it could not be done, or empty.
    Q_PROPERTY(QString problem READ problem NOTIFY problemChanged)

public:
    /// The most a store holds, written out, and the longest key.
    static constexpr int maxBytes = 64 * 1024;
    static constexpr int maxKeyLength = 100;

    /// The store kept for `game`, read from where it is kept. Use storeFor, which keeps one per game per process.
    explicit GameStore(const QString &game, QObject *parent = nullptr);
    /// A copy in memory only, starting from `contents` (a replay's).
    GameStore(const QVariantMap &contents, QObject *parent);
    ~GameStore() override;

    /// The one store of the game whose package is `package`, made on first use and kept until the application closes;
    /// null for a package that names no folder.
    static GameStore *storeFor(const QUrl &package);
    /// The game a package is: its folder's name and a hash of where it is, so two folders of one name differ.
    static QString gameOf(const QUrl &package);
    /// Where `game`'s store is kept on this computer (in a browser, the local storage key).
    static QString placeOf(const QString &game);

    QString game() const { return m_game; }
    bool kept() const { return !m_game.isEmpty(); }
    QStringList keys() const { return m_data.keys(); }
    int size() const;
    QString problem() const { return m_problem; }

    /// What is kept under `key`, or `fallback`.
    Q_INVOKABLE QVariant value(const QString &key, const QVariant &fallback = {}) const;
    /// Keep `value` under `key`: false, with `problem` saying why, if it cannot be kept.
    Q_INVOKABLE bool setValue(const QString &key, const QJSValue &value);
    /// Forget `key`; true if it was kept.
    Q_INVOKABLE bool remove(const QString &key);
    /// Forget everything this game kept.
    Q_INVOKABLE void clear();
    /// Everything kept, as an object.
    Q_INVOKABLE QVariantMap contents() const { return m_data.toVariantMap(); }
    /// Write what is waiting to be written now, rather than in a moment.
    Q_INVOKABLE void flush();

signals:
    void changed();
    void problemChanged();

private:
    void read();
    void refuse(const QString &why);
    void touched();

    QString m_game;
    QJsonObject m_data;
    QString m_problem;
    bool m_dirty = false;
    QTimer m_later;
};

}  // namespace qq
