#include "run.h"

#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEvent>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlEngine>
#include <QRandomGenerator>

#include <cmath>
#include <iterator>
#include <utility>

#include "input.h"
#include "sceneio.h"

namespace qq {

namespace {

std::uint32_t rotl(std::uint32_t x, int k)
{
    return (x << k) | (x >> (32 - k));
}

QByteArray canonical(const QVariantMap &map)
{
    return QJsonDocument(QJsonObject::fromVariantMap(map)).toJson(QJsonDocument::Compact);
}

}  // namespace

// ── Random ──

Random::Random(QObject *parent) : QObject(parent)
{
    reset(1);
}

void Random::reset(double seed)
{
    // splitmix32 over the seed's low 32 bits fills the four words; a state of all zeros cannot come from it.
    auto mix = static_cast<std::uint32_t>(static_cast<std::int64_t>(seed));
    for (std::uint32_t &word : m_state) {
        mix += 0x9e3779b9u;
        std::uint32_t z = mix;
        z = (z ^ (z >> 16)) * 0x85ebca6bu;
        z = (z ^ (z >> 13)) * 0xc2b2ae35u;
        word = z ^ (z >> 16);
    }
    m_drawn = 0;
}

std::uint32_t Random::nextWord()
{
    // xoshiro128** 1.1 (Blackman and Vigna).
    const std::uint32_t result = rotl(m_state[1] * 5u, 7) * 9u;
    const std::uint32_t t = m_state[1] << 9;
    m_state[2] ^= m_state[0];
    m_state[3] ^= m_state[1];
    m_state[1] ^= m_state[2];
    m_state[0] ^= m_state[3];
    m_state[2] ^= t;
    m_state[3] = rotl(m_state[3], 11);
    return result;
}

double Random::next()
{
    ++m_drawn;
    const std::uint32_t high = nextWord() >> 5;  // 27 bits
    const std::uint32_t low = nextWord() >> 6;   // 26 bits
    return (double(high) * 67108864.0 + double(low)) / 9007199254740992.0;
}

int Random::between(int low, int high)
{
    if (high < low)
        std::swap(low, high);
    const double span = double(high) - double(low) + 1.0;
    return int(double(low) + std::floor(next() * span));
}

int Random::freshSeed()
{
    return int(QRandomGenerator::system()->bounded(1u, 2147483647u));
}

// ── RunRecord ──

RunRecord::RunRecord(QObject *parent) : QObject(parent) {}

void RunRecord::setScene(const QString &scene)
{
    if (scene != m_scene) {
        m_scene = scene;
        emit changed();
    }
}

void RunRecord::setSeed(int seed)
{
    if (seed != m_seed) {
        m_seed = seed;
        emit changed();
    }
}

void RunRecord::setTicks(int ticks)
{
    if (ticks != m_ticks) {
        m_ticks = ticks;
        emit changed();
    }
}

void RunRecord::setOutcome(const QVariantMap &outcome)
{
    if (outcome != m_outcome) {
        m_outcome = outcome;
        emit changed();
    }
}

void RunRecord::capture(int tick, const QVariantMap &hands, const QStringList &clicks)
{
    QVariantMap state = hands;
    const QVariantList edges = state.take(QStringLiteral("edges")).toList();
    // Kept as JSON keeps it, so a state read back from a file compares equal to one captured live.
    state = QJsonObject::fromVariantMap(state).toVariantMap();
    if (state != m_last) {
        m_states.insert(tick, state);
        m_last = state;
    }
    if (!edges.isEmpty())
        m_edges.insert(tick, edges);
    if (!clicks.isEmpty())
        m_clicks.insert(tick, clicks);
    if (tick + 1 > m_ticks)
        setTicks(tick + 1);
}

QVariantMap RunRecord::handsAt(int tick) const
{
    QVariantMap hands;
    auto it = m_states.upperBound(tick);
    if (it != m_states.cbegin())
        hands = std::prev(it).value();
    if (hands.isEmpty()) {
        hands.insert(QStringLiteral("move"), QVariantList{0, 0});
        hands.insert(QStringLiteral("look"), QVariantList{0, 0});
        hands.insert(QStringLiteral("held"), QStringList());
    }
    hands.insert(QStringLiteral("edges"), m_edges.value(tick));
    return hands;
}

QStringList RunRecord::clicksAt(int tick) const
{
    return m_clicks.value(tick);
}

void RunRecord::clear()
{
    m_states.clear();
    m_edges.clear();
    m_clicks.clear();
    m_last.clear();
    m_outcome.clear();
    m_ticks = 0;
    emit changed();
}

QByteArray RunRecord::toJson() const
{
    QJsonArray states;
    for (auto it = m_states.cbegin(); it != m_states.cend(); ++it)
        states.append(QJsonArray{it.key(), QJsonObject::fromVariantMap(it.value())});
    QJsonArray edges;
    for (auto it = m_edges.cbegin(); it != m_edges.cend(); ++it)
        edges.append(QJsonArray{it.key(), QJsonArray::fromVariantList(it.value())});
    QJsonArray clicks;
    for (auto it = m_clicks.cbegin(); it != m_clicks.cend(); ++it)
        clicks.append(QJsonArray{it.key(), QJsonArray::fromStringList(it.value())});
    const QJsonObject run{{QStringLiteral("qqRun"), 1},
                          {QStringLiteral("scene"), m_scene},
                          {QStringLiteral("seed"), m_seed},
                          {QStringLiteral("ticks"), m_ticks},
                          {QStringLiteral("outcome"), QJsonObject::fromVariantMap(m_outcome)},
                          {QStringLiteral("hands"), states},
                          {QStringLiteral("edges"), edges},
                          {QStringLiteral("clicks"), clicks}};
    return QJsonDocument(run).toJson(QJsonDocument::Indented);
}

bool RunRecord::fromJson(const QByteArray &json, QString *why)
{
    auto fail = [why](const QString &reason) {
        if (why)
            *why = reason;
        return false;
    };
    QJsonParseError parse{};
    const QJsonObject run = QJsonDocument::fromJson(json, &parse).object();
    if (parse.error != QJsonParseError::NoError)
        return fail(QStringLiteral("The run is not JSON."));
    if (run.value(QStringLiteral("qqRun")).toInt() != 1)
        return fail(QStringLiteral("This reads QQ runs of format 1."));
    clear();
    m_scene = run.value(QStringLiteral("scene")).toString();
    m_seed = run.value(QStringLiteral("seed")).toInt();
    m_ticks = run.value(QStringLiteral("ticks")).toInt();
    m_outcome = run.value(QStringLiteral("outcome")).toObject().toVariantMap();
    for (const QJsonValue &v : run.value(QStringLiteral("hands")).toArray())
        m_states.insert(v.toArray().at(0).toInt(), v.toArray().at(1).toObject().toVariantMap());
    for (const QJsonValue &v : run.value(QStringLiteral("edges")).toArray())
        m_edges.insert(v.toArray().at(0).toInt(), v.toArray().at(1).toArray().toVariantList());
    for (const QJsonValue &v : run.value(QStringLiteral("clicks")).toArray()) {
        QStringList names;
        for (const QJsonValue &name : v.toArray().at(1).toArray())
            names << name.toString();
        m_clicks.insert(v.toArray().at(0).toInt(), names);
    }
    if (m_ticks < 0)
        return fail(QStringLiteral("The run has no length."));
    emit changed();
    return true;
}

// ── replay ──

QVariantMap replayRun(QQmlEngine *engine, const QUrl &file, RunRecord *record)
{
    QVariantMap result{{QStringLiteral("outcome"), QVariantMap()},
                       {QStringLiteral("ticks"), 0},
                       {QStringLiteral("matches"), false},
                       {QStringLiteral("problem"), QString()}};
    auto refuse = [&result](const QString &why) {
        result[QStringLiteral("problem")] = why;
        return result;
    };
    if (!engine || !record)
        return refuse(QStringLiteral("Nothing to replay."));
    auto *io = engine->singletonInstance<SceneIO *>("QQ", "SceneIO");
    auto *input = engine->singletonInstance<Input *>("QQ", "Input");
    if (!io || !input)
        return refuse(QStringLiteral("The QQ module is not loaded."));
    QObject *scene = io->load(file, nullptr);
    if (!scene)
        return refuse(io->lastError());

    // Ticked by hand from here, its hands and clicks from the record, its random numbers from the record's seed, and no
    // physics world made: what a replay reaches is what logic and QQ's own kinematics reach, never PhysX.
    scene->setProperty("replaying", true);
    scene->setProperty("autoAdvance", false);
    scene->setProperty("recorder", QVariant::fromValue(static_cast<QObject *>(record)));
    scene->setProperty("playSeed", record->seed());
    scene->setProperty("playing", true);

    // Logic is built before the first tick, as it is in play.
    QElapsedTimer waiting;
    waiting.start();
    bool ready = false;
    while (waiting.elapsed() < 20000) {
        QVariant answer;
        QMetaObject::invokeMethod(scene, "ready", Q_RETURN_ARG(QVariant, answer));
        if ((ready = answer.toBool()))
            break;
        QCoreApplication::processEvents(QEventLoop::AllEvents, 20);
    }
    // A game whose logic does not build would replay to an empty outcome, and that is no match for anything.
    QVariant broken;
    QMetaObject::invokeMethod(scene, "logicErrors", Q_RETURN_ARG(QVariant, broken));
    const QStringList errors = broken.toStringList();
    if (!ready || !errors.isEmpty()) {
        input->endReplay();
        io->discard(scene);
        return refuse(ready ? QStringLiteral("The scene's logic does not build: %1").arg(errors.join(QStringLiteral("; ")))
                            : QStringLiteral("The scene's logic was not built."));
    }

    for (int tick = 0; tick < record->ticks(); ++tick) {
        QMetaObject::invokeMethod(scene, "tick");
        if (tick % 60 == 59)
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }
    QVariant text;
    QMetaObject::invokeMethod(scene, "outcomeText", Q_RETURN_ARG(QVariant, text));
    const QVariantMap outcome = QJsonDocument::fromJson(text.toString().toUtf8()).object().toVariantMap();
    result[QStringLiteral("outcome")] = outcome;
    result[QStringLiteral("ticks")] = scene->property("ticks").toInt();
    result[QStringLiteral("matches")] = canonical(outcome) == canonical(record->outcome());
    input->endReplay();
    io->discard(scene);
    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    return result;
}

}  // namespace qq
