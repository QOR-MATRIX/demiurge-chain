#include "studiotools.h"

#include <QBuffer>
#include <QColor>
#include <QDir>
#include <QFileInfo>
#include <QImage>
#include <QJSValue>
#include <QJsonDocument>
#include <QMutex>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QVector2D>
#include <QVector3D>

#include <cmath>

#include "input.h"
#include "sceneio.h"

namespace qq {

namespace {

QMutex logLock;
QtMessageHandler previousHandler = nullptr;

void keepMessage(QtMsgType type, const QMessageLogContext &context, const QString &message)
{
    if (type != QtDebugMsg) {
        const char *level = type == QtInfoMsg ? "info" : type == QtWarningMsg ? "warning" : "error";
        StudioLog::instance()->append(QString::fromLatin1(level), message);
    }
    if (previousHandler)
        previousHandler(type, context, message);
}

/// A QML `var` property's value as plain Qt types (a JavaScript object becomes a QVariantMap, an array a QVariantList).
QVariant plain(const QVariant &v)
{
    return v.canConvert<QJSValue>() && v.metaType() == QMetaType::fromType<QJSValue>() ? v.value<QJSValue>().toVariant()
                                                                                       : v;
}

QStringList fieldsOf(const QObject *o)
{
    return plain(o->property("fields")).toStringList();
}

double rounded(double v)
{
    return std::round(v * 10000.0) / 10000.0;
}

QString json(const QJsonObject &o)
{
    return QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
}

McpServer::Result ok(const QJsonObject &o)
{
    return McpServer::Result::text(json(o));
}

McpServer::Result problem(const QString &why)
{
    return McpServer::Result::text(why, true);
}

bool vectorFrom(const QJsonValue &v, int n, QList<double> &out)
{
    out.clear();
    if (v.isArray() && v.toArray().size() == n) {
        for (const QJsonValue &x : v.toArray()) {
            if (!x.isDouble())
                return false;
            out << x.toDouble();
        }
        return true;
    }
    if (v.isObject()) {
        const QJsonObject o = v.toObject();
        const char *names[] = {"x", "y", "z"};
        for (int i = 0; i < n; ++i) {
            const QJsonValue x = o.value(QLatin1String(names[i]));
            if (!x.isDouble())
                return false;
            out << x.toDouble();
        }
        return true;
    }
    return false;
}

QJsonObject schema(const QJsonObject &properties, const QStringList &required = {})
{
    QJsonObject s{{QStringLiteral("type"), QStringLiteral("object")}, {QStringLiteral("properties"), properties},
                  {QStringLiteral("additionalProperties"), false}};
    if (!required.isEmpty())
        s.insert(QStringLiteral("required"), QJsonArray::fromStringList(required));
    return s;
}

QJsonObject prop(const QString &type, const QString &description)
{
    return {{QStringLiteral("type"), type}, {QStringLiteral("description"), description}};
}

QJsonObject vec(int n, const QString &description)
{
    return {{QStringLiteral("type"), QStringLiteral("array")},
            {QStringLiteral("items"), QJsonObject{{QStringLiteral("type"), QStringLiteral("number")}}},
            {QStringLiteral("minItems"), n},
            {QStringLiteral("maxItems"), n},
            {QStringLiteral("description"), description}};
}

}  // namespace

// ── the log ───────────────────────────────────────────────────────────────────────────────────

StudioLog *StudioLog::instance()
{
    static StudioLog log;
    return &log;
}

void StudioLog::install()
{
    instance();
    previousHandler = qInstallMessageHandler(keepMessage);
}

void StudioLog::append(const QString &level, const QString &text)
{
    QMutexLocker lock(&logLock);
    // Logic that throws does so every frame: the same line again is counted, not kept again, so it cannot push out
    // everything else. It moves to the end, numbered anew, so a reader asking for what is new sees it is still happening.
    if (!m_entries.isEmpty() && m_entries.last().text == text && m_entries.last().level == level) {
        m_entries.last().repeats += 1;
        m_entries.last().seq = m_next++;
        return;
    }
    m_entries.append(Entry{m_next++, level, text, 1});
    if (m_entries.size() > 500)
        m_entries.removeFirst();
}

QJsonObject StudioLog::since(qint64 since) const
{
    QMutexLocker lock(&logLock);
    QJsonArray entries;
    for (const Entry &e : m_entries) {
        if (e.seq > since) {
            QJsonObject entry{{QStringLiteral("seq"), e.seq}, {QStringLiteral("level"), e.level},
                              {QStringLiteral("text"), e.text}};
            if (e.repeats > 1)
                entry.insert(QStringLiteral("times"), e.repeats);
            entries.append(entry);
        }
    }
    return {{QStringLiteral("entries"), entries}, {QStringLiteral("next"), m_next - 1}};
}

// ── the tools ─────────────────────────────────────────────────────────────────────────────────

StudioTools::StudioTools(QQuickWindow *studio, QQmlEngine *engine, McpServer *server, QObject *parent)
    : QObject(parent), m_studio(studio), m_engine(engine), m_server(server)
{
    m_io = engine->singletonInstance<SceneIO *>("QQ", "SceneIO");
    m_server->setServerInfo(QStringLiteral("qq-studio"), QStringLiteral("0.3"), instructions());
    addTools();

    // What the Studio says, the agent can read: its notices and play's problems join Qt's messages in the log.
    connect(studio, SIGNAL(noticeChanged()), this, SLOT(noticeChanged()));
    connect(studio, SIGNAL(playProblemChanged()), this, SLOT(playProblemChanged()));
    connect(server, &McpServer::clientConnected, this, [this](const QString &client) {
        say(tr("An agent is connected (%1). What it changes shows as unsaved; you save and commit.")
                .arg(client.isEmpty() ? tr("an MCP client") : client));
    });
}

void StudioTools::noticeChanged()
{
    const QString notice = m_studio->property("notice").toString();
    if (!notice.isEmpty())
        StudioLog::instance()->append(m_studio->property("noticeIsProblem").toBool() ? QStringLiteral("problem")
                                                                                     : QStringLiteral("notice"),
                                      notice);
}

void StudioTools::playProblemChanged()
{
    const QString why = m_studio->property("playProblem").toString();
    if (!why.isEmpty())
        StudioLog::instance()->append(QStringLiteral("problem"), why);
}

QString StudioTools::instructions()
{
    return QStringLiteral(
        "You are working in QQ Studio, the editor of QQ (the QOR Engine), on one 3D scene that the creator sees as "
        "you change it.\n"
        "Units are metres and degrees. Y is up; the ground is y = 0; a Player faces -z. A Shape of scale [1,1,1] is a "
        "1 m cube (or a sphere 1 m across); position is its centre. A Player's position is its feet.\n"
        "Work like this: scene_read and kinds to see what is there and what can be added; entity_add and entity_set "
        "to build (give every entity a clear name); behaviour_write for game logic (QML: a file whose root is Logic, "
        "with onFrame: (dt) => { ... }, `target` the entity it drives, `scene`, `elapsed`, entity(name), and Input."
        "move/look/jump/run); play, then wait, input_set and frame_capture to see it run and judge it; log_read for "
        "errors; stop, then revise. Edits are refused while playing, except logic (rebuilt at once on each write) and "
        "input. Stop always restores the scene exactly.\n"
        "You cannot save or commit: the creator reviews your work, saves it and commits it.");
}

void StudioTools::addTools()
{
    using Args = const QJsonObject &;
    using Done = const McpServer::Done &;
    auto now = [](auto f) { return [f](Args a, Done done) { done(f(a)); }; };

    m_server->addTool(QStringLiteral("scene_read"),
                      QStringLiteral("The scene as its canonical QML text, each entity's fields as JSON, and whether "
                                     "it is playing or unsaved."),
                      schema({}), now([this](Args) { return sceneRead(); }));
    m_server->addTool(QStringLiteral("kinds"),
                      QStringLiteral("Every kind of entity that can be added, with its fields: type, default, and the "
                                     "range or choices allowed."),
                      schema({}), now([this](Args) { return kinds(); }));
    m_server->addTool(
        QStringLiteral("scene_set"),
        QStringLiteral("Change the scene's own fields: its name, sky colours, skyLight, exposure, bloom, gravity."),
        schema({{QStringLiteral("fields"), prop(QStringLiteral("object"), QStringLiteral("Field name to value."))}},
               {QStringLiteral("fields")}),
        now([this](Args a) { return sceneSet(a); }));
    m_server->addTool(
        QStringLiteral("entity_add"),
        QStringLiteral("Add an entity of a kind (see kinds), with its fields. Vectors are [x, y, z], colours "
                       "\"#rrggbb\", files a path in the project (\"assets/x.glb\") or on this computer. The name given "
                       "is made unique if taken; the entity added is returned."),
        schema({{QStringLiteral("kind"), prop(QStringLiteral("string"), QStringLiteral("Shape, Prop, Lamp, Sun, Ground, "
                                                                                       "Player, Emitter, Sound or "
                                                                                       "Behaviour."))},
                {QStringLiteral("name"), prop(QStringLiteral("string"), QStringLiteral("What to call it."))},
                {QStringLiteral("fields"), prop(QStringLiteral("object"), QStringLiteral("Field name to value."))}},
               {QStringLiteral("kind")}),
        now([this](Args a) { return entityAdd(a); }));
    m_server->addTool(QStringLiteral("entity_set"),
                      QStringLiteral("Change fields of the entity called `name`. Nothing is changed if any value is "
                                     "refused."),
                      schema({{QStringLiteral("name"), prop(QStringLiteral("string"), QStringLiteral("The entity."))},
                              {QStringLiteral("fields"),
                               prop(QStringLiteral("object"), QStringLiteral("Field name to value."))}},
                             {QStringLiteral("name"), QStringLiteral("fields")}),
                      now([this](Args a) { return entitySet(a); }));
    m_server->addTool(QStringLiteral("entity_remove"), QStringLiteral("Take the entity called `name` out of the scene."),
                      schema({{QStringLiteral("name"), prop(QStringLiteral("string"), QStringLiteral("The entity."))}},
                             {QStringLiteral("name")}),
                      now([this](Args a) { return entityRemove(a); }));
    m_server->addTool(
        QStringLiteral("behaviour_write"),
        QStringLiteral("Write game logic to logic/<file>.qml in the project, and say whether it builds. The file's root "
                       "is a Logic (import QQ). With `target`, a Behaviour entity running the file on that entity is "
                       "added if there is none (\"\" for logic that looks after the whole scene). While playing, the "
                       "running logic is rebuilt from the file at once."),
        schema({{QStringLiteral("file"), prop(QStringLiteral("string"), QStringLiteral("The file's name, without "
                                                                                       "logic/ or .qml."))},
                {QStringLiteral("code"), prop(QStringLiteral("string"), QStringLiteral("The whole QML file."))},
                {QStringLiteral("target"), prop(QStringLiteral("string"), QStringLiteral("The entity it drives."))}},
               {QStringLiteral("file"), QStringLiteral("code")}),
        now([this](Args a) { return behaviourWrite(a); }));
    m_server->addTool(QStringLiteral("play"),
                      QStringLiteral("Play a copy of the scene: physics, the player, particles, sound and logic run."),
                      schema({}), now([this](Args) { return play(); }));
    m_server->addTool(QStringLiteral("stop"),
                      QStringLiteral("Stop playing; the scene is exactly as it was before play."), schema({}),
                      now([this](Args) { return stop(); }));
    m_server->addTool(
        QStringLiteral("input_set"),
        QStringLiteral("While playing, hold the player's controls as a gamepad would: move [right, forward] and look "
                       "[turn right, tilt up], each from -1 to 1; jump; run. They stay held until changed or stop."),
        schema({{QStringLiteral("move"), vec(2, QStringLiteral("[x right, y forward], -1 to 1."))},
                {QStringLiteral("look"), vec(2, QStringLiteral("[x turn right, y tilt up], -1 to 1."))},
                {QStringLiteral("jump"), prop(QStringLiteral("boolean"), QStringLiteral("Jump held."))},
                {QStringLiteral("run"), prop(QStringLiteral("boolean"), QStringLiteral("Run held."))}}),
        now([this](Args a) { return inputSet(a); }));
    m_server->addTool(
        QStringLiteral("wait"),
        QStringLiteral("Let frames pass: `ms` milliseconds, at most 10000. Returns where things stand."),
        schema({{QStringLiteral("ms"), prop(QStringLiteral("integer"), QStringLiteral("Milliseconds, 1 to 10000."))}},
               {QStringLiteral("ms")}),
        [this](Args a, Done done) {
            const int ms = std::clamp(a.value(QStringLiteral("ms")).toInt(500), 1, 10000);
            QPointer<StudioTools> self(this);
            QTimer::singleShot(ms, this, [self, done] {
                if (self)
                    done(McpServer::Result::text(self->status()));
            });
        });
    m_server->addTool(
        QStringLiteral("view_set"),
        QStringLiteral("Where the editing eye looks from (not in play, where it follows the player): yaw and pitch in "
                       "degrees around `target`, at `distance` metres."),
        schema({{QStringLiteral("yaw"), prop(QStringLiteral("number"), QStringLiteral("Degrees about the vertical."))},
                {QStringLiteral("pitch"), prop(QStringLiteral("number"), QStringLiteral("Degrees up, -5 to 85."))},
                {QStringLiteral("distance"), prop(QStringLiteral("number"), QStringLiteral("Metres from the target."))},
                {QStringLiteral("target"), vec(3, QStringLiteral("The point looked at, [x, y, z]."))}}),
        now([this](Args a) { return viewSet(a); }));
    m_server->addTool(
        QStringLiteral("frame_capture"),
        QStringLiteral("The viewport as it is now, as a PNG image, `width` pixels wide (default 1024, at most 1600)."),
        schema({{QStringLiteral("width"), prop(QStringLiteral("integer"), QStringLiteral("Pixels wide."))}}),
        now([this](Args a) { return frameCapture(a); }));
    m_server->addTool(
        QStringLiteral("log_read"),
        QStringLiteral("What the Studio has said since entry `since` (0 for everything kept): notices, logic errors, QML "
                       "warnings. Returns the entries and the number to ask from next."),
        schema({{QStringLiteral("since"), prop(QStringLiteral("integer"), QStringLiteral("The last entry already read."))}}),
        now([this](Args a) { return logRead(a); }));
}

// ── reading ───────────────────────────────────────────────────────────────────────────────────

QObject *StudioTools::scene() const
{
    return m_studio ? m_studio->property("scene").value<QObject *>() : nullptr;
}

QObject *StudioTools::playScene() const
{
    return m_studio ? m_studio->property("playScene").value<QObject *>() : nullptr;
}

bool StudioTools::playing() const
{
    return m_studio && m_studio->property("playing").toBool();
}

QUrl StudioTools::project() const
{
    return m_studio ? m_studio->property("project").toUrl() : QUrl();
}

QObject *StudioTools::entity(const QString &name) const
{
    if (QObject *s = scene()) {
        for (const QVariant &e : m_io->entities(s)) {
            if (e.value<QObject *>()->property("name").toString() == name)
                return e.value<QObject *>();
        }
    }
    return nullptr;
}

QJsonValue StudioTools::valueOf(QObject *e, const QString &field) const
{
    const QVariant v = e->property(field.toUtf8());
    const QString type = m_io->fieldType(e, field);
    if (type == QStringLiteral("vector3d")) {
        const QVector3D p = v.value<QVector3D>();
        return QJsonArray{rounded(p.x()), rounded(p.y()), rounded(p.z())};
    }
    if (type == QStringLiteral("color"))
        return v.value<QColor>().name();
    if (type == QStringLiteral("url")) {
        const QUrl url = m_io->resolvedUrl(e, v.toUrl());
        if (url.isEmpty())
            return QString();
        if (url.isLocalFile() && project().isLocalFile()) {
            const QString root = project().toLocalFile();
            const QString path = url.toLocalFile();
            if (QDir::cleanPath(path).startsWith(QDir::cleanPath(root) + QLatin1Char('/')))
                return QDir(root).relativeFilePath(path);
            return path;
        }
        return url.toString();
    }
    if (type == QStringLiteral("real"))
        return rounded(v.toDouble());
    return QJsonValue::fromVariant(plain(v));
}

QJsonObject StudioTools::fieldsOf(QObject *e) const
{
    QJsonObject out;
    for (const QString &f : qq::fieldsOf(e))
        out.insert(f, valueOf(e, f));
    return out;
}

QString StudioTools::status() const
{
    QJsonObject s{{QStringLiteral("playing"), playing()},
                  {QStringLiteral("unsaved"), m_studio && m_studio->property("dirty").toBool()},
                  {QStringLiteral("notice"), m_studio ? m_studio->property("notice").toString() : QString()}};
    if (m_studio && !m_studio->property("playProblem").toString().isEmpty())
        s.insert(QStringLiteral("problem"), m_studio->property("playProblem").toString());
    if (QObject *played = playScene()) {
        if (QObject *player = played->property("player").value<QObject *>()) {
            const QVector3D feet = player->property("feet").value<QVector3D>();
            s.insert(QStringLiteral("player"),
                     QJsonObject{{QStringLiteral("feet"), QJsonArray{rounded(feet.x()), rounded(feet.y()), rounded(feet.z())}},
                                 {QStringLiteral("heading"), rounded(player->property("heading").toDouble())}});
        }
        // Where bodies are in play: what moved, and where it went.
        QJsonObject bodies;
        for (const QVariant &v : m_io->entities(played)) {
            QObject *e = v.value<QObject *>();
            auto *body = e->property("physics").value<QObject *>();
            if (body && e->property("body").toString() == QStringLiteral("dynamic")) {
                const QVector3D p = body->property("scenePosition").value<QVector3D>();
                bodies.insert(e->property("name").toString(), QJsonArray{rounded(p.x()), rounded(p.y()), rounded(p.z())});
            }
        }
        if (!bodies.isEmpty())
            s.insert(QStringLiteral("dynamicBodies"), bodies);
    }
    return json(s);
}

McpServer::Result StudioTools::sceneRead()
{
    QObject *s = scene();
    if (!s)
        return problem(tr("There is no scene open."));
    QJsonArray entities;
    for (const QVariant &v : m_io->entities(s)) {
        QObject *e = v.value<QObject *>();
        entities.append(QJsonObject{{QStringLiteral("kind"), e->property("kind").toString()},
                                    {QStringLiteral("fields"), fieldsOf(e)}});
    }
    const QUrl file = m_studio->property("sceneFile").toUrl();
    return ok({{QStringLiteral("scene"), fieldsOf(s)},
               {QStringLiteral("entities"), entities},
               {QStringLiteral("file"), file.isLocalFile() && project().isLocalFile()
                                            ? QDir(project().toLocalFile()).relativeFilePath(file.toLocalFile())
                                            : QString()},
               {QStringLiteral("project"), project().isLocalFile() ? project().toLocalFile() : QString()},
               {QStringLiteral("playing"), playing()},
               {QStringLiteral("unsaved"), m_studio->property("dirty").toBool()},
               {QStringLiteral("qml"), m_io->write(s, file)}});
}

McpServer::Result StudioTools::kinds()
{
    if (m_kinds.isEmpty()) {
        // One of each, made in a scene of its own that nobody sees, to read what each declares and starts as.
        QObject *probe = m_io->loadText(QStringLiteral("import QQ\nScene {}\n"),
                                        QUrl(QStringLiteral("qrc:/qq-agent/kinds.qml")), nullptr);
        if (!probe)
            return problem(m_io->lastError());
        for (const QString &kind : m_io->kinds()) {
            QObject *e = m_io->add(probe, kind, {});
            if (!e)
                continue;
            const QVariantMap ranges = plain(e->property("ranges")).toMap();
            const QVariantMap choices = plain(e->property("choices")).toMap();
            QJsonArray fields;
            for (const QString &f : qq::fieldsOf(e)) {
                QJsonObject field{{QStringLiteral("name"), f}, {QStringLiteral("type"), m_io->fieldType(e, f)},
                                  {QStringLiteral("default"), valueOf(e, f)}};
                if (ranges.contains(f))
                    field.insert(QStringLiteral("range"), QJsonValue::fromVariant(plain(ranges.value(f))));
                if (choices.contains(f))
                    field.insert(QStringLiteral("choices"), QJsonValue::fromVariant(plain(choices.value(f))));
                fields.append(field);
            }
            m_kinds.insert(kind, fields);
        }
        m_io->discard(probe);
    }
    return ok(m_kinds);
}

McpServer::Result StudioTools::logRead(const QJsonObject &args)
{
    return ok(StudioLog::instance()->since(args.value(QStringLiteral("since")).toInteger(0)));
}

// ── changing ──────────────────────────────────────────────────────────────────────────────────

QString StudioTools::setField(QObject *e, const QString &field, const QJsonValue &value)
{
    const QString kind = e->property("kind").toString();
    if (!qq::fieldsOf(e).contains(field))
        return tr("%1 has no field %2; its fields are %3.").arg(kind, field, qq::fieldsOf(e).join(QStringLiteral(", ")));
    const QString type = m_io->fieldType(e, field);
    const QVariantMap ranges = plain(e->property("ranges")).toMap();
    const QVariantMap choices = plain(e->property("choices")).toMap();
    QVariant v;
    if (type == QStringLiteral("vector3d")) {
        QList<double> xyz;
        if (!vectorFrom(value, 3, xyz))
            return tr("%1 is a vector: [x, y, z].").arg(field);
        v = QVector3D(float(xyz[0]), float(xyz[1]), float(xyz[2]));
    } else if (type == QStringLiteral("color")) {
        const QString name = value.toString();
        if (!QColor::isValidColorName(name))
            return tr("%1 is a colour: \"#rrggbb\".").arg(field);
        v = QColor::fromString(name);
    } else if (type == QStringLiteral("url")) {
        if (!value.isString())
            return tr("%1 is a file: a path in the project, or on this computer.").arg(field);
        const QString path = value.toString();
        if (path.isEmpty()) {
            v = QUrl();
        } else if (path.startsWith(QStringLiteral("file:")) || path.startsWith(QStringLiteral("qrc:"))) {
            v = QUrl(path);
        } else {
            QString local = path;
            if (QDir::isRelativePath(path)) {
                if (!project().isLocalFile())
                    return tr("A path in the project needs a project: the creator has not chosen one.");
                local = QDir(project().toLocalFile()).filePath(path);
            }
            if (!QFileInfo(local).isFile())
                return tr("There is no file %1.").arg(QDir::toNativeSeparators(local));
            v = QUrl::fromLocalFile(QDir::cleanPath(local));
        }
    } else if (type == QStringLiteral("real") || type == QStringLiteral("int")) {
        if (!value.isDouble())
            return tr("%1 is a number.").arg(field);
        const double x = value.toDouble();
        if (ranges.contains(field)) {
            const QVariantList r = plain(ranges.value(field)).toList();
            if (r.size() == 2 && (x < r[0].toDouble() || x > r[1].toDouble()))
                return tr("%1 is from %2 to %3.").arg(field).arg(r[0].toDouble()).arg(r[1].toDouble());
        }
        v = type == QStringLiteral("int") ? QVariant(qRound(x)) : QVariant(x);
    } else if (type == QStringLiteral("bool")) {
        if (!value.isBool())
            return tr("%1 is true or false.").arg(field);
        v = value.toBool();
    } else if (type == QStringLiteral("string")) {
        if (!value.isString())
            return tr("%1 is text.").arg(field);
        if (choices.contains(field)) {
            const QStringList allowed = plain(choices.value(field)).toStringList();
            if (!allowed.contains(value.toString()))
                return tr("%1 is one of %2.").arg(field, allowed.join(QStringLiteral(", ")));
        }
        v = value.toString();
    } else {
        return tr("%1 cannot be set by an agent.").arg(field);
    }
    if (!e->setProperty(field.toUtf8(), v))
        return tr("%1 could not be set to that.").arg(field);
    return {};
}

QString StudioTools::setFields(QObject *e, const QJsonObject &fields)
{
    // All or nothing: every value is checked against a copy of what is there before any is kept.
    QVariantMap before;
    for (auto it = fields.begin(); it != fields.end(); ++it)
        before.insert(it.key(), e->property(it.key().toUtf8()));
    for (auto it = fields.begin(); it != fields.end(); ++it) {
        if (it.key() == QStringLiteral("name") && e->property("kind").toString() != QStringLiteral("Scene")) {
            const QString name = it.value().toString().trimmed();
            QObject *other = entity(name);
            if (name.isEmpty() || (other && other != e)) {
                for (auto b = before.cbegin(); b != before.cend(); ++b)
                    e->setProperty(b.key().toUtf8(), b.value());
                return name.isEmpty() ? tr("An entity has a name.") : tr("There is already an entity called %1.").arg(name);
            }
        }
        const QString why = setField(e, it.key(), it.value());
        if (!why.isEmpty()) {
            for (auto b = before.cbegin(); b != before.cend(); ++b)
                e->setProperty(b.key().toUtf8(), b.value());
            return why;
        }
    }
    return {};
}

void StudioTools::refresh()
{
    QMetaObject::invokeMethod(m_studio, "refresh");
}

void StudioTools::say(const QString &text, bool isProblem)
{
    if (m_studio)
        QMetaObject::invokeMethod(m_studio, "say", Q_ARG(QVariant, text), Q_ARG(QVariant, isProblem));
}

McpServer::Result StudioTools::sceneSet(const QJsonObject &args)
{
    if (playing())
        return problem(tr("The scene is playing: stop, then change it."));
    QObject *s = scene();
    if (!s)
        return problem(tr("There is no scene open."));
    const QString why = setFields(s, args.value(QStringLiteral("fields")).toObject());
    if (!why.isEmpty())
        return problem(why);
    refresh();
    say(tr("The agent changed the scene's look."));
    return ok({{QStringLiteral("scene"), fieldsOf(s)}});
}

McpServer::Result StudioTools::entityAdd(const QJsonObject &args)
{
    if (playing())
        return problem(tr("The scene is playing: stop, then change it."));
    if (!scene())
        return problem(tr("There is no scene open."));
    const QString kind = args.value(QStringLiteral("kind")).toString();
    if (!m_io->kinds().contains(kind))
        return problem(tr("There is no kind %1; the kinds are %2.").arg(kind, m_io->kinds().join(QStringLiteral(", "))));
    QJsonObject fields = args.value(QStringLiteral("fields")).toObject();
    QVariantMap start;
    const QString name = args.value(QStringLiteral("name")).toString(fields.value(QStringLiteral("name")).toString());
    if (!name.trimmed().isEmpty())
        start.insert(QStringLiteral("name"), name.trimmed());
    fields.remove(QStringLiteral("name"));
    QVariant made;
    QMetaObject::invokeMethod(m_studio, "add", Q_RETURN_ARG(QVariant, made), Q_ARG(QVariant, kind),
                              Q_ARG(QVariant, start));
    QObject *e = made.value<QObject *>();
    if (!e)
        return problem(m_studio->property("notice").toString());
    const QString why = setFields(e, fields);
    if (!why.isEmpty()) {
        m_io->remove(e);
        m_studio->setProperty("selection", QVariant::fromValue<QObject *>(nullptr));
        QTimer::singleShot(0, this, &StudioTools::refresh);
        return problem(why);
    }
    refresh();
    say(tr("The agent added %1 \"%2\".").arg(kind, e->property("name").toString()));
    return ok({{QStringLiteral("kind"), kind}, {QStringLiteral("fields"), fieldsOf(e)}});
}

McpServer::Result StudioTools::entitySet(const QJsonObject &args)
{
    if (playing())
        return problem(tr("The scene is playing: stop, then change it."));
    const QString name = args.value(QStringLiteral("name")).toString();
    QObject *e = entity(name);
    if (!e)
        return problem(tr("There is no entity called %1.").arg(name));
    const QString why = setFields(e, args.value(QStringLiteral("fields")).toObject());
    if (!why.isEmpty())
        return problem(why);
    refresh();
    say(tr("The agent changed \"%1\".").arg(e->property("name").toString()));
    return ok({{QStringLiteral("kind"), e->property("kind").toString()}, {QStringLiteral("fields"), fieldsOf(e)}});
}

McpServer::Result StudioTools::entityRemove(const QJsonObject &args)
{
    if (playing())
        return problem(tr("The scene is playing: stop, then change it."));
    const QString name = args.value(QStringLiteral("name")).toString();
    QObject *e = entity(name);
    if (!e)
        return problem(tr("There is no entity called %1.").arg(name));
    if (m_studio->property("selection").value<QObject *>() == e)
        m_studio->setProperty("selection", QVariant::fromValue<QObject *>(nullptr));
    m_io->remove(e);
    QTimer::singleShot(0, this, &StudioTools::refresh);  // the entity goes when control returns to the event loop
    say(tr("The agent removed \"%1\".").arg(name));
    return ok({{QStringLiteral("removed"), name}});
}

McpServer::Result StudioTools::behaviourWrite(const QJsonObject &args)
{
    if (!project().isLocalFile())
        return problem(tr("Logic lives in the project, and the creator has not chosen one."));
    const QString file = args.value(QStringLiteral("file")).toString();
    const QString code = args.value(QStringLiteral("code")).toString();
    if (file.trimmed().isEmpty() || code.trimmed().isEmpty())
        return problem(tr("Give the file a name and some code."));
    const QUrl url = m_io->writeLogic(project(), file, code);
    if (url.isEmpty())
        return problem(m_io->lastError());
    const QString relative = QDir(project().toLocalFile()).relativeFilePath(url.toLocalFile());

    // Does it build? Checked here, from the text, so the agent hears now rather than at play.
    QStringList errors;
    {
        QQmlComponent check(m_engine);
        check.setData(code.toUtf8(), url);
        for (const QQmlError &e : check.errors())
            errors << e.toString();
    }

    QJsonObject result{{QStringLiteral("file"), relative}, {QStringLiteral("builds"), errors.isEmpty()}};
    if (args.contains(QStringLiteral("target"))) {
        const QString target = args.value(QStringLiteral("target")).toString();
        bool attached = false;
        for (const QVariant &v : m_io->entities(scene())) {
            QObject *e = v.value<QObject *>();
            if (e->property("kind").toString() == QStringLiteral("Behaviour")
                && m_io->resolvedUrl(e, e->property("source").toUrl()) == url) {
                attached = true;
            }
        }
        if (!attached && !target.isEmpty() && !entity(target)) {
            result.insert(QStringLiteral("behaviour"), tr("not added: there is no entity called %1").arg(target));
        } else if (!attached && playing()) {
            result.insert(QStringLiteral("behaviour"), tr("not added while playing: stop, then write it again"));
        } else if (!attached) {
            QVariant made;
            const QVariantMap fields{
                {QStringLiteral("name"), target.isEmpty() ? file : target + QStringLiteral(" logic")},
                {QStringLiteral("source"), url},
                {QStringLiteral("target"), target}};
            QMetaObject::invokeMethod(m_studio, "add", Q_RETURN_ARG(QVariant, made),
                                      Q_ARG(QVariant, QStringLiteral("Behaviour")), Q_ARG(QVariant, fields));
            if (QObject *b = made.value<QObject *>())
                result.insert(QStringLiteral("behaviour"), b->property("name").toString());
        } else {
            result.insert(QStringLiteral("behaviour"), tr("already attached"));
        }
    }
    say(errors.isEmpty() ? tr("The agent wrote %1.").arg(relative)
                         : tr("The agent wrote %1, which does not build.").arg(relative),
        !errors.isEmpty());
    if (!errors.isEmpty()) {
        result.insert(QStringLiteral("errors"), QJsonArray::fromStringList(errors));
        return McpServer::Result::text(json(result), true);
    }
    return ok(result);
}

// ── playing ───────────────────────────────────────────────────────────────────────────────────

McpServer::Result StudioTools::play()
{
    if (playing())
        return McpServer::Result::text(status());
    QVariant started;
    QMetaObject::invokeMethod(m_studio, "play", Q_RETURN_ARG(QVariant, started));
    if (!started.toBool())
        return problem(m_studio->property("notice").toString());
    return McpServer::Result::text(status());
}

McpServer::Result StudioTools::stop()
{
    if (m_drivingInput) {
        if (auto *input = m_engine->singletonInstance<Input *>("QQ", "Input")) {
            input->gamepadState({}, {}, false, false, false);
            input->setReadsHardware(true);
        }
        m_drivingInput = false;
    }
    if (playing())
        QMetaObject::invokeMethod(m_studio, "stop");
    return McpServer::Result::text(status());
}

McpServer::Result StudioTools::inputSet(const QJsonObject &args)
{
    if (!playing())
        return problem(tr("Input is for play: play first."));
    auto *input = m_engine->singletonInstance<Input *>("QQ", "Input");
    if (!input)
        return problem(tr("There is no input to hold."));
    QList<double> move{0, 0};
    QList<double> look{0, 0};
    if (args.contains(QStringLiteral("move")) && !vectorFrom(args.value(QStringLiteral("move")), 2, move))
        return problem(tr("move is [x, y], each from -1 to 1."));
    if (args.contains(QStringLiteral("look")) && !vectorFrom(args.value(QStringLiteral("look")), 2, look))
        return problem(tr("look is [x, y], each from -1 to 1."));
    for (double x : move + look) {
        if (x < -1 || x > 1)
            return problem(tr("Each of move and look is from -1 to 1."));
    }
    // The agent's hands are a gamepad's: a real pad is set aside until stop, so the two do not fight.
    input->setReadsHardware(false);
    m_drivingInput = true;
    input->gamepadState(QVector2D(float(move[0]), float(move[1])), QVector2D(float(look[0]), float(look[1])),
                        args.value(QStringLiteral("jump")).toBool(), args.value(QStringLiteral("run")).toBool(), true);
    return McpServer::Result::text(status());
}

McpServer::Result StudioTools::viewSet(const QJsonObject &args)
{
    if (playing())
        return problem(tr("In play the eye follows the player; stop to place it."));
    if (args.contains(QStringLiteral("yaw")))
        m_studio->setProperty("yaw", args.value(QStringLiteral("yaw")).toDouble());
    if (args.contains(QStringLiteral("pitch")))
        m_studio->setProperty("pitch", std::clamp(args.value(QStringLiteral("pitch")).toDouble(), -5.0, 85.0));
    if (args.contains(QStringLiteral("distance")))
        m_studio->setProperty("distance", std::clamp(args.value(QStringLiteral("distance")).toDouble(), 0.5, 500.0));
    if (args.contains(QStringLiteral("target"))) {
        QList<double> t;
        if (!vectorFrom(args.value(QStringLiteral("target")), 3, t))
            return problem(tr("target is [x, y, z]."));
        m_studio->setProperty("target", QVector3D(float(t[0]), float(t[1]), float(t[2])));
    }
    return ok({{QStringLiteral("yaw"), rounded(m_studio->property("yaw").toDouble())},
               {QStringLiteral("pitch"), rounded(m_studio->property("pitch").toDouble())},
               {QStringLiteral("distance"), rounded(m_studio->property("distance").toDouble())}});
}

McpServer::Result StudioTools::frameCapture(const QJsonObject &args)
{
    auto *world = m_studio->findChild<QQuickItem *>(QStringLiteral("world"));
    if (!world)
        return problem(tr("There is no viewport to capture."));
    const QImage whole = m_studio->grabWindow();
    if (whole.isNull())
        return problem(tr("The window could not be captured."));
    const qreal dpr = whole.devicePixelRatio() > 0 ? whole.devicePixelRatio() : 1;
    const QRectF area = world->mapRectToScene(world->boundingRect());
    QImage frame = whole.copy(QRect(qRound(area.x() * dpr), qRound(area.y() * dpr), qRound(area.width() * dpr),
                                    qRound(area.height() * dpr)));
    const int width = std::clamp(args.value(QStringLiteral("width")).toInt(1024), 64, 1600);
    if (frame.width() > width)
        frame = frame.scaledToWidth(width, Qt::SmoothTransformation);
    QByteArray png;
    QBuffer buffer(&png);
    buffer.open(QIODevice::WriteOnly);
    frame.save(&buffer, "PNG");

    McpServer::Result r;
    r.content.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("image")},
                                 {QStringLiteral("data"), QString::fromLatin1(png.toBase64())},
                                 {QStringLiteral("mimeType"), QStringLiteral("image/png")}});
    r.content.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("text")},
                                 {QStringLiteral("text"), QStringLiteral("%1 x %2. %3").arg(frame.width())
                                                              .arg(frame.height())
                                                              .arg(status())}});
    return r;
}

}  // namespace qq
