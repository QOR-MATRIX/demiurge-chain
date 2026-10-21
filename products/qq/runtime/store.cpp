#include "store.h"

#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHash>
#include <QJSValueIterator>
#include <QJsonArray>
#include <QJsonDocument>
#include <QPointer>
#include <QSaveFile>
#include <QStandardPaths>

#include <cmath>

#ifdef Q_OS_WASM
#include <emscripten.h>

// The page's local storage, one item per game. Each call answers at once; a browser that keeps nothing (storage turned
// off, or full) says so rather than throwing into the game.
EM_JS_DEPS(qqStore, "$UTF8ToString,$stringToUTF8,$lengthBytesUTF8");

/// The item's length in bytes as UTF-8; -1 when there is none, -2 when the page has no storage.
EM_JS(int, qqStoreLength, (const char *key), {
    try {
        const text = globalThis.localStorage.getItem(UTF8ToString(key));
        return text === null ? -1 : lengthBytesUTF8(text);
    } catch (e) {
        return -2;
    }
});

EM_JS(void, qqStoreCopy, (const char *key, char *into, int size), {
    try {
        stringToUTF8(globalThis.localStorage.getItem(UTF8ToString(key)) || "", into, size);
    } catch (e) {
    }
});

/// Keep `text` under `key`, or remove the item when it is empty: 1 done, 0 refused.
EM_JS(int, qqStoreWrite, (const char *key, const char *text), {
    try {
        const name = UTF8ToString(key);
        const value = UTF8ToString(text);
        if (value.length === 0)
            globalThis.localStorage.removeItem(name);
        else
            globalThis.localStorage.setItem(name, value);
        return 1;
    } catch (e) {
        return 0;
    }
});
#endif

namespace qq {

namespace {

constexpr int maxDepth = 32;

/// `value` as JSON, or false with `why` when it is not something a store keeps.
bool toJson(const QJSValue &value, QJsonValue *out, int depth, QString *why)
{
    if (depth > maxDepth) {
        *why = QStringLiteral("it is nested more than %1 deep").arg(maxDepth);
        return false;
    }
    if (value.isUndefined()) {
        *why = QStringLiteral("it is undefined (forget(key) removes a key)");
        return false;
    }
    if (value.isNull()) {
        *out = QJsonValue(QJsonValue::Null);
        return true;
    }
    if (value.isBool()) {
        *out = value.toBool();
        return true;
    }
    if (value.isNumber()) {
        const double number = value.toNumber();
        if (!std::isfinite(number)) {
            *why = QStringLiteral("it is not a finite number");
            return false;
        }
        *out = number;
        return true;
    }
    if (value.isString()) {
        *out = value.toString();
        return true;
    }
    if (value.isCallable()) {
        *why = QStringLiteral("it is a function");
        return false;
    }
    if (value.isQObject()) {
        *why = QStringLiteral("it is an entity, or another of the engine's objects");
        return false;
    }
    if (value.isArray()) {
        QJsonArray list;
        const quint32 length = value.property(QStringLiteral("length")).toUInt();
        for (quint32 i = 0; i < length; ++i) {
            QJsonValue item;
            if (!toJson(value.property(i), &item, depth + 1, why))
                return false;
            list.append(item);
        }
        *out = list;
        return true;
    }
    // A plain object, and nothing that only looks like one: a vector, a colour, a date or a pattern is a value of its
    // own, and would come back as something else.
    if (value.isObject() && value.toVariant().metaType() == QMetaType::fromType<QVariantMap>()) {
        QJsonObject object;
        QJSValueIterator it(value);
        while (it.hasNext()) {
            it.next();
            QJsonValue item;
            if (!toJson(it.value(), &item, depth + 1, why))
                return false;
            object.insert(it.name(), item);
        }
        *out = object;
        return true;
    }
    *why = QStringLiteral("a store keeps numbers, text, true and false, null, and lists and objects of them (keep a "
                          "vector as its numbers, a colour as its text)");
    return false;
}

/// A store's text as an object, or false if it is not one a store could have written.
bool parse(const QByteArray &text, QJsonObject *out)
{
    if (text.size() > GameStore::maxBytes)
        return false;
    QJsonParseError error{};
    const QJsonDocument document = QJsonDocument::fromJson(text, &error);
    if (error.error != QJsonParseError::NoError || !document.isObject())
        return false;
    *out = document.object();
    return true;
}

QHash<QString, QPointer<GameStore>> &stores()
{
    static QHash<QString, QPointer<GameStore>> kept;
    return kept;
}

}  // namespace

GameStore::GameStore(const QString &game, QObject *parent) : QObject(parent), m_game(game)
{
    // Written a moment after the last change rather than on every one: logic may save every step.
    m_later.setSingleShot(true);
    m_later.setInterval(250);
    connect(&m_later, &QTimer::timeout, this, &GameStore::flush);
    if (QCoreApplication::instance())
        connect(QCoreApplication::instance(), &QCoreApplication::aboutToQuit, this, &GameStore::flush);
    read();
}

GameStore::GameStore(const QVariantMap &contents, QObject *parent)
    : QObject(parent), m_data(QJsonObject::fromVariantMap(contents))
{
}

GameStore::~GameStore()
{
    flush();
}

GameStore *GameStore::storeFor(const QUrl &package)
{
    const QString game = gameOf(package);
    if (game.isEmpty())
        return nullptr;
    QPointer<GameStore> &store = stores()[game];
    // One per game in the whole process, so two engines (the Studio's and a check's) never write over each other.
    if (!store)
        store = new GameStore(game, QCoreApplication::instance());
    return store;
}

QString GameStore::gameOf(const QUrl &package)
{
    QString where;
    if (package.scheme() == QStringLiteral("qrc"))
        where = QStringLiteral("qrc:") + QDir::cleanPath(package.path());
    else if (package.isLocalFile() && !QDir::isRelativePath(package.toLocalFile()))
        where = QDir::cleanPath(package.toLocalFile());
    if (where.isEmpty())
        return {};
#ifdef Q_OS_WIN
    where = where.toLower();  // one folder, however its drive or its name is cased
#endif
    QString name;
    for (const QChar c : QFileInfo(where).fileName().toLower()) {
        if (c.unicode() < 128 && c.isLetterOrNumber())
            name += c;
        else if (!name.isEmpty() && !name.endsWith(QLatin1Char('-')))
            name += QLatin1Char('-');
    }
    while (name.endsWith(QLatin1Char('-')))
        name.chop(1);
    name.truncate(40);
    if (name.isEmpty())
        name = QStringLiteral("game");
    const QByteArray hash = QCryptographicHash::hash(where.toUtf8(), QCryptographicHash::Sha256).toHex().left(16);
    return name + QLatin1Char('-') + QString::fromLatin1(hash);
}

QString GameStore::placeOf(const QString &game)
{
#ifdef Q_OS_WASM
    return QStringLiteral("qq-store/") + game;
#else
    return QDir(QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation))
        .filePath(QStringLiteral("Demiurge/QQ/saves/") + game + QStringLiteral(".json"));
#endif
}

int GameStore::size() const
{
    return m_data.isEmpty() ? 0 : int(QJsonDocument(m_data).toJson(QJsonDocument::Compact).size());
}

QVariant GameStore::value(const QString &key, const QVariant &fallback) const
{
    const auto it = m_data.constFind(key);
    return it == m_data.constEnd() ? fallback : it->toVariant();
}

bool GameStore::setValue(const QString &key, const QJSValue &value)
{
    if (key.isEmpty() || key.size() > maxKeyLength) {
        refuse(QStringLiteral("A key is 1 to %1 characters.").arg(maxKeyLength));
        return false;
    }
    QJsonValue json;
    QString why;
    if (!toJson(value, &json, 0, &why)) {
        refuse(QStringLiteral("\"%1\" was not kept: %2.").arg(key, why));
        return false;
    }
    const auto it = m_data.constFind(key);
    const bool same = it != m_data.constEnd() && *it == json;
    QJsonObject next = m_data;
    next.insert(key, json);
    const qsizetype bytes = same ? 0 : QJsonDocument(next).toJson(QJsonDocument::Compact).size();
    if (bytes > maxBytes) {
        refuse(QStringLiteral("\"%1\" was not kept: the store would hold %2 bytes, and holds at most %3.")
                   .arg(key)
                   .arg(bytes)
                   .arg(maxBytes));
        return false;
    }
    if (!m_problem.isEmpty()) {
        m_problem.clear();
        emit problemChanged();
    }
    if (!same) {
        m_data = next;
        touched();
    }
    return true;
}

bool GameStore::remove(const QString &key)
{
    if (!m_data.contains(key))
        return false;
    m_data.remove(key);
    touched();
    return true;
}

void GameStore::clear()
{
    if (m_data.isEmpty())
        return;
    m_data = QJsonObject();
    touched();
}

void GameStore::flush()
{
    m_later.stop();
    if (!m_dirty || !kept())
        return;
    m_dirty = false;
    // An empty store is no store: nothing is left behind once a game forgets everything.
    const QByteArray text = m_data.isEmpty() ? QByteArray() : QJsonDocument(m_data).toJson(QJsonDocument::Compact);
#ifdef Q_OS_WASM
    if (!qqStoreWrite(placeOf(m_game).toUtf8().constData(), text.constData()))
        refuse(QStringLiteral("The browser would not keep this game's saved data."));
#else
    const QString path = placeOf(m_game);
    if (text.isEmpty()) {
        QFile::remove(path);
        return;
    }
    QDir().mkpath(QFileInfo(path).absolutePath());
    QSaveFile out(path);
    if (!out.open(QIODevice::WriteOnly) || out.write(text) != text.size() || !out.commit())
        refuse(QStringLiteral("This game's saved data could not be written: %1").arg(out.errorString()));
#endif
}

void GameStore::read()
{
    QByteArray text;
#ifdef Q_OS_WASM
    const QByteArray key = placeOf(m_game).toUtf8();
    const int length = qqStoreLength(key.constData());
    if (length == -2) {
        refuse(QStringLiteral("This browser keeps no saved data for the page, so nothing is kept between plays."));
        return;
    }
    if (length < 0)
        return;
    text = QByteArray(length + 1, '\0');
    qqStoreCopy(key.constData(), text.data(), length + 1);
    text.chop(1);
    if (parse(text, &m_data))
        return;
    // Kept aside rather than written over, for whoever looks.
    qqStoreWrite((key + ".unreadable").constData(), text.constData());
    qqStoreWrite(key.constData(), "");
#else
    QFile file(placeOf(m_game));
    if (!file.exists())
        return;
    if (!file.open(QIODevice::ReadOnly)) {
        refuse(QStringLiteral("This game's saved data could not be opened: %1").arg(file.errorString()));
        return;
    }
    text = file.read(maxBytes + 1);
    file.close();
    if (parse(text, &m_data))
        return;
    // Kept aside rather than written over, for whoever looks.
    const QString aside = file.fileName() + QStringLiteral(".unreadable");
    QFile::remove(aside);
    QFile::rename(file.fileName(), aside);
#endif
    m_data = QJsonObject();
    refuse(QStringLiteral("This game's saved data could not be read, so it starts without it; it is kept aside, as "
                          "unreadable."));
}

void GameStore::refuse(const QString &why)
{
    // The reason, never the value: what a game keeps is its player's.
    qWarning("QQ store: %s", qPrintable(why));
    if (why != m_problem) {
        m_problem = why;
        emit problemChanged();
    }
}

void GameStore::touched()
{
    m_dirty = true;
    emit changed();
    if (!kept())
        return;
#ifdef Q_OS_WASM
    flush();  // the page can close at any moment, and local storage answers at once
#else
    m_later.start();
#endif
}

}  // namespace qq
