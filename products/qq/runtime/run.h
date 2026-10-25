// A game's run, kept so it can be played again (DIRECTION P3.11, ADR-105 decision 6).
//
// A scene plays in ticks of 1/60 s (Scene.qml). Each tick takes the player's hands (Input::takeStep: move, look, the
// actions held, and the presses and releases since the last tick), the screen buttons clicked and what the game's host
// told it (ADR-117), and only then lets logic and QQ's own kinematics step. So a run is wholly its seed and what each
// tick took: a RunRecord keeps those, and a replay gives them back tick by tick, with no window and no physics engine,
// to reach the same outcome. What a replay cannot reach, PhysX's results among it, is never part of an outcome that is
// rewarded (ADR-102 decision 5).
//
// Random numbers come from Random, seeded by the scene's `seed` or by whoever starts the run: xoshiro128**, in whole
// numbers, so the same seed gives the same numbers on every machine.

#pragma once

#include <QMap>
#include <QObject>
#include <QStringList>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

#include <array>
#include <cstdint>

class QQmlEngine;

namespace qq {

/// Seeded random numbers, the same on every machine: xoshiro128**, its state filled from the seed by splitmix32.
class Random : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// How many numbers have been drawn since the last reset.
    Q_PROPERTY(int drawn READ drawn)

public:
    explicit Random(QObject *parent = nullptr);

    /// Start again from `seed`.
    Q_INVOKABLE void reset(double seed);
    /// A number from 0 up to, not including, 1, with 53 bits of chance.
    Q_INVOKABLE double next();
    /// A whole number from `low` to `high`, both included.
    Q_INVOKABLE int between(int low, int high);

    int drawn() const { return m_drawn; }

    /// A seed for a run that names none: never 0, which means "choose one".
    static int freshSeed();

private:
    std::uint32_t nextWord();

    std::array<std::uint32_t, 4> m_state{};
    int m_drawn = 0;
};

/// What a run took, tick by tick, and what it came to.
class RunRecord : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// The scene's name, the seed it played with, how many ticks it lasted, and its outcome (each Logic's `outcome`, by
    /// its Behaviour's name).
    Q_PROPERTY(QString scene READ scene WRITE setScene NOTIFY changed)
    Q_PROPERTY(int seed READ seed WRITE setSeed NOTIFY changed)
    Q_PROPERTY(int ticks READ ticks WRITE setTicks NOTIFY changed)
    Q_PROPERTY(QVariantMap outcome READ outcome WRITE setOutcome NOTIFY changed)
    /// The game's store as the run found it (P3.16): a replay plays from this copy, never from the store itself.
    Q_PROPERTY(QVariantMap store READ store WRITE setStore NOTIFY changed)

public:
    explicit RunRecord(QObject *parent = nullptr);

    QString scene() const { return m_scene; }
    void setScene(const QString &scene);
    int seed() const { return m_seed; }
    void setSeed(int seed);
    int ticks() const { return m_ticks; }
    void setTicks(int ticks);
    QVariantMap outcome() const { return m_outcome; }
    void setOutcome(const QVariantMap &outcome);
    QVariantMap store() const { return m_store; }
    void setStore(const QVariantMap &store);

    /// What tick `tick` took: the hands (move, look, held, edges) and the buttons clicked, by name. Only what changed is
    /// kept: the hands' state when it differs from the tick before, the edges and clicks when there are any.
    Q_INVOKABLE void capture(int tick, const QVariantMap &hands, const QStringList &clicks);
    /// The hands as tick `tick` took them: the state in force then, and that tick's edges.
    Q_INVOKABLE QVariantMap handsAt(int tick) const;
    /// The buttons clicked at tick `tick`.
    Q_INVOKABLE QStringList clicksAt(int tick) const;
    /// What the game's host told it, heard at tick `tick` (ADR-117), and what a replay hears there.
    Q_INVOKABLE void keepTold(int tick, const QVariantList &told);
    Q_INVOKABLE QVariantList toldAt(int tick) const;
    /// Forget everything.
    Q_INVOKABLE void clear();

    /// As a file keeps it (JSON), and read back; false, with `why`, if it is not a run.
    QByteArray toJson() const;
    bool fromJson(const QByteArray &json, QString *why = nullptr);

signals:
    void changed();

private:
    QString m_scene;
    int m_seed = 0;
    int m_ticks = 0;
    QVariantMap m_outcome;
    QVariantMap m_store;
    QMap<int, QVariantMap> m_states;
    QMap<int, QVariantList> m_edges;
    QMap<int, QStringList> m_clicks;
    QMap<int, QVariantList> m_told;
    QVariantMap m_last;
};

/// Play `record` again on the scene in `file`, with no window and no physics engine, as fast as it goes: { outcome,
/// ticks, matches (the outcome is the one recorded), problem (why it could not, or empty) }.
QVariantMap replayRun(QQmlEngine *engine, const QUrl &file, RunRecord *record);

}  // namespace qq
