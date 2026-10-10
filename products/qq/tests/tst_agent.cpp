// Can an agent build and play a game in QQ Studio, through MCP and through the Studio's own design loop? (DIRECTION P3.3)
//
// Four levels. The protocol alone: MCP's JSON-RPC as a client sends it, errors included. The tools in the real Studio
// window: each one, driven through the protocol exactly as a client's lines arrive, with what changed checked in the
// window and on disk. And the real programs: qq-mcp started as an MCP client starts it, relaying to a real qq-studio
// over a pipe of the test's own. And the design loop, against a stand-in for Anthropic's API on this computer: what
// it sends, what it runs, what it leaves for the creator, and how it meets a busy provider, a refused key and a refusal.

#include <QColor>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalSocket>
#include <QProcess>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QRandomGenerator>
#include <QSet>
#include <QSignalSpy>
#include <QTcpServer>
#include <QTcpSocket>
#include <QHostAddress>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>

#include "designloop.h"
#include "gpu.h"
#include "keychain.h"
#include "mcpserver.h"
#include "sceneio.h"
#include "studiotools.h"

static const QString chosenGpu = qq::preferHighPerformanceGpu();

Q_IMPORT_QML_PLUGIN(QQPlugin)
Q_IMPORT_QML_PLUGIN(QQ_StudioPlugin)

namespace {

QString fixture(const QString &name)
{
    return QDir(QStringLiteral(QQ_FIXTURES)).filePath(name);
}

QJsonObject parse(const QByteArray &line)
{
    return QJsonDocument::fromJson(line).object();
}

QByteArray request(int id, const QString &method, const QJsonObject &params = {})
{
    QJsonObject m{{QStringLiteral("jsonrpc"), QStringLiteral("2.0")}, {QStringLiteral("id"), id},
                  {QStringLiteral("method"), method}};
    if (!params.isEmpty())
        m.insert(QStringLiteral("params"), params);
    return QJsonDocument(m).toJson(QJsonDocument::Compact);
}

/// A client of an McpServer in this process: each request goes in as a line, and the answer is the line that comes back.
struct Client {
    qq::McpServer *server;
    int next = 1;

    QJsonObject ask(const QString &method, const QJsonObject &params = {}, int timeoutMs = 15000)
    {
        QList<QByteArray> answers;
        server->handle(request(next++, method, params), [&answers](const QByteArray &a) { answers << a; });
        (void)QTest::qWaitFor([&answers] { return !answers.isEmpty(); }, timeoutMs);
        return answers.isEmpty() ? QJsonObject() : parse(answers.first());
    }

    /// A tool's result: { isError, text (the text content), json (that text, parsed, when it is JSON), image }.
    QJsonObject tool(const QString &name, const QJsonObject &arguments = {}, int timeoutMs = 15000)
    {
        const QJsonObject answer = ask(QStringLiteral("tools/call"),
                                       {{QStringLiteral("name"), name}, {QStringLiteral("arguments"), arguments}},
                                       timeoutMs);
        const QJsonObject result = answer.value(QStringLiteral("result")).toObject();
        QJsonObject out{{QStringLiteral("isError"), result.value(QStringLiteral("isError")).toBool(true)}};
        for (const QJsonValue &c : result.value(QStringLiteral("content")).toArray()) {
            const QJsonObject item = c.toObject();
            if (item.value(QStringLiteral("type")) == QStringLiteral("text")) {
                const QString text = item.value(QStringLiteral("text")).toString();
                out.insert(QStringLiteral("text"), text);
                const QJsonDocument doc = QJsonDocument::fromJson(text.toUtf8());
                if (doc.isObject())
                    out.insert(QStringLiteral("json"), doc.object());
            } else if (item.value(QStringLiteral("type")) == QStringLiteral("image")) {
                out.insert(QStringLiteral("image"), item);
            }
        }
        if (answer.contains(QStringLiteral("error")))
            out.insert(QStringLiteral("text"), answer.value(QStringLiteral("error")).toObject().value(QStringLiteral("message")));
        return out;
    }
};

/// One open Studio over a temporary project, with its tools on a server of the test's own.
struct Studio {
    QTemporaryDir project;
    QQmlApplicationEngine engine;
    QQuickWindow *window = nullptr;
    qq::McpServer server;
    std::unique_ptr<qq::StudioTools> tools;
    Client client{&server};

    bool open()
    {
        QDir(project.path()).mkdir(QStringLiteral(".git"));
        engine.setInitialProperties({{QStringLiteral("model"), QUrl::fromLocalFile(fixture(QStringLiteral("orb.gltf")))},
                                     {QStringLiteral("project"), QUrl::fromLocalFile(project.path())}});
        engine.loadFromModule("QQ.Studio", "Main");
        if (engine.rootObjects().isEmpty())
            return false;
        window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        if (!window || !QTest::qWaitForWindowExposed(window))
            return false;
        tools = std::make_unique<qq::StudioTools>(window, &engine, &server);
        return QTest::qWaitFor([this] { return window->property("settled").toBool(); }, 15000);
    }

    ~Studio()
    {
        tools.reset();
        if (window)
            QMetaObject::invokeMethod(window, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    QObject *entity(const QString &name) const
    {
        for (const QVariant &e : window->property("entityList").toList()) {
            if (e.value<QObject *>()->property("name").toString() == name)
                return e.value<QObject *>();
        }
        return nullptr;
    }

    QString sceneText()
    {
        auto *io = engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        return io->write(window->property("scene").value<QObject *>(), window->property("sceneFile").toUrl());
    }
};

/// A program's standard output, a line at a time.
QJsonObject readLine(QProcess &p, int timeoutMs = 20000)
{
    QElapsedTimer t;
    t.start();
    while (!p.canReadLine() && t.elapsed() < timeoutMs)
        p.waitForReadyRead(200);
    return p.canReadLine() ? parse(p.readLine().trimmed()) : QJsonObject();
}


/// A stand-in for Anthropic's Messages API on this computer: each request is kept (its headers and body) and answered
/// with the next scripted answer, so the design loop runs whole without a key or a bill.
class FakeAnthropic : public QObject
{
public:
    struct Answer {
        int status = 200;
        QJsonObject body;
        QByteArray retryAfter;
    };
    struct Request {
        QHash<QByteArray, QByteArray> headers;
        QJsonObject body;
        QByteArray raw;
    };

    QTcpServer server;
    QList<Answer> script;
    QList<Request> requests;

    FakeAnthropic()
    {
        server.listen(QHostAddress::LocalHost);
        connect(&server, &QTcpServer::newConnection, this, [this] {
            while (QTcpSocket *s = server.nextPendingConnection()) {
                auto *buffer = new QByteArray;
                connect(s, &QTcpSocket::disconnected, s, [s, buffer] {
                    delete buffer;
                    s->deleteLater();
                });
                connect(s, &QTcpSocket::readyRead, this, [this, s, buffer] {
                    buffer->append(s->readAll());
                    const int end = buffer->indexOf("\r\n\r\n");
                    if (end < 0)
                        return;
                    Request r;
                    int length = 0;
                    for (const QByteArray &line : buffer->left(end).split('\n').mid(1)) {
                        const int colon = line.indexOf(':');
                        if (colon < 0)
                            continue;
                        const QByteArray name = line.left(colon).trimmed().toLower();
                        r.headers.insert(name, line.mid(colon + 1).trimmed());
                        if (name == "content-length")
                            length = line.mid(colon + 1).trimmed().toInt();
                    }
                    if (buffer->size() < end + 4 + length)
                        return;
                    r.raw = buffer->mid(end + 4, length);
                    r.body = QJsonDocument::fromJson(r.raw).object();
                    buffer->clear();
                    requests << r;
                    const Answer a = script.isEmpty() ? Answer{500, {{QStringLiteral("type"), QStringLiteral("error")}}, {}}
                                                      : script.takeFirst();
                    const QByteArray body = QJsonDocument(a.body).toJson(QJsonDocument::Compact);
                    QByteArray head = "HTTP/1.1 " + QByteArray::number(a.status) + " X\r\nContent-Type: application/json\r\n"
                                      "Connection: close\r\nContent-Length: " + QByteArray::number(body.size()) + "\r\n";
                    if (!a.retryAfter.isEmpty())
                        head += "retry-after: " + a.retryAfter + "\r\n";
                    s->write(head + "\r\n" + body);
                    s->flush();
                    s->disconnectFromHost();
                });
            }
        });
    }

    QString url() const { return QStringLiteral("http://127.0.0.1:%1/v1/messages").arg(server.serverPort()); }

    /// An answer from the model: its content blocks and why it stopped.
    static Answer said(const QJsonArray &content, const QString &stop)
    {
        // Every answer counts what it used, as the API's do: 100 in, 50 written to the cache, 30 read from it, 20 out.
        const QJsonObject usage{{QStringLiteral("input_tokens"), 100}, {QStringLiteral("cache_creation_input_tokens"), 50},
                                {QStringLiteral("cache_read_input_tokens"), 30}, {QStringLiteral("output_tokens"), 20}};
        return {200, {{QStringLiteral("id"), QStringLiteral("msg_test")}, {QStringLiteral("type"), QStringLiteral("message")},
                      {QStringLiteral("role"), QStringLiteral("assistant")}, {QStringLiteral("model"), QStringLiteral("claude-opus-5-5")},
                      {QStringLiteral("content"), content}, {QStringLiteral("stop_reason"), stop},
                      {QStringLiteral("usage"), usage}},
                {}};
    }
    static QJsonObject text(const QString &t) { return {{QStringLiteral("type"), QStringLiteral("text")}, {QStringLiteral("text"), t}}; }
    static QJsonObject use(const QString &id, const QString &name, const QJsonObject &input = {})
    {
        return {{QStringLiteral("type"), QStringLiteral("tool_use")}, {QStringLiteral("id"), id},
                {QStringLiteral("name"), name}, {QStringLiteral("input"), input}};
    }
    static Answer error(int status, const QString &type, const QByteArray &retryAfter = {})
    {
        return {status,
                {{QStringLiteral("type"), QStringLiteral("error")},
                 {QStringLiteral("error"), QJsonObject{{QStringLiteral("type"), type}, {QStringLiteral("message"), type}}}},
                retryAfter};
    }
};

/// A key and a server of the test's own, for the length of a case: the creator's keychain entry is never touched.
struct DesignerSetup {
    FakeAnthropic api;
    QByteArray target = "QQ Studio test key " + QByteArray::number(QRandomGenerator::global()->generate());
    QString key = QStringLiteral("sk-ant-test-%1-SECRET").arg(QRandomGenerator::global()->generate());

    DesignerSetup()
    {
        qputenv("QQ_KEYCHAIN_TARGET", target);
        qputenv("QQ_ANTHROPIC_URL", api.url().toUtf8());
    }
    ~DesignerSetup()
    {
        qq::keychain::remove();
        qunsetenv("QQ_KEYCHAIN_TARGET");
        qunsetenv("QQ_ANTHROPIC_URL");
    }
};

}  // namespace

class TstAgent : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase()
    {
        QQuickStyle::setStyle(QStringLiteral("Material"));
        qq::StudioLog::install();
    }

    // ── the protocol ──

    void initializeAgreesAVersionAndSaysHowToWork()
    {
        qq::McpServer server;
        server.setServerInfo(QStringLiteral("qq-studio"), QStringLiteral("0.3"), qq::StudioTools::instructions());
        Client c{&server};
        QJsonObject r = c.ask(QStringLiteral("initialize"),
                              {{QStringLiteral("protocolVersion"), QStringLiteral("2025-03-26")},
                               {QStringLiteral("clientInfo"), QJsonObject{{QStringLiteral("name"), QStringLiteral("test")}}},
                               {QStringLiteral("capabilities"), QJsonObject()}})
                            .value(QStringLiteral("result"))
                            .toObject();
        QCOMPARE(r.value(QStringLiteral("protocolVersion")).toString(), QStringLiteral("2025-03-26"));
        QCOMPARE(r.value(QStringLiteral("serverInfo")).toObject().value(QStringLiteral("name")).toString(),
                 QStringLiteral("qq-studio"));
        QVERIFY(r.value(QStringLiteral("capabilities")).toObject().contains(QStringLiteral("tools")));
        QVERIFY(r.value(QStringLiteral("instructions")).toString().contains(QStringLiteral("metres")));

        // A version it does not speak: its newest, and the client decides.
        r = c.ask(QStringLiteral("initialize"), {{QStringLiteral("protocolVersion"), QStringLiteral("1999-01-01")}})
                .value(QStringLiteral("result"))
                .toObject();
        QCOMPARE(r.value(QStringLiteral("protocolVersion")).toString(), qq::McpServer::protocolVersions().first());
        QVERIFY(c.ask(QStringLiteral("ping")).contains(QStringLiteral("result")));
    }

    void whatIsNotAnAnswerableRequestIsSaidSo()
    {
        qq::McpServer server;
        server.addTool(QStringLiteral("echo"), QStringLiteral("Says it back."), QJsonObject{{QStringLiteral("type"), QStringLiteral("object")}},
                       [](const QJsonObject &a, const qq::McpServer::Done &done) {
                           done(qq::McpServer::Result::text(a.value(QStringLiteral("say")).toString(),
                                                            a.contains(QStringLiteral("fail"))));
                       });
        QList<QByteArray> answers;
        auto send = [&](const QByteArray &line) {
            answers.clear();
            server.handle(line, [&answers](const QByteArray &a) { answers << a; });
            return answers.isEmpty() ? QJsonObject() : parse(answers.first());
        };
        auto code = [](const QJsonObject &o) { return o.value(QStringLiteral("error")).toObject().value(QStringLiteral("code")).toInt(); };

        QCOMPARE(code(send("{not json")), -32700);
        QCOMPARE(code(send(R"([{"jsonrpc":"2.0","id":1,"method":"ping"}])")), -32600);
        QCOMPARE(code(send(request(2, QStringLiteral("resources/list")))), -32601);
        QCOMPARE(code(send(request(3, QStringLiteral("tools/call"), {{QStringLiteral("name"), QStringLiteral("nope")}}))), -32602);
        // A notification is not answered.
        send(R"({"jsonrpc":"2.0","method":"notifications/initialized"})");
        QVERIFY(answers.isEmpty());

        // The list, and a call: a tool's failure is a result the model reads, with its id.
        const QJsonObject list = send(request(4, QStringLiteral("tools/list")));
        QCOMPARE(list.value(QStringLiteral("result")).toObject().value(QStringLiteral("tools")).toArray().first().toObject()
                     .value(QStringLiteral("name")).toString(),
                 QStringLiteral("echo"));
        const QJsonObject ok = send(request(5, QStringLiteral("tools/call"),
                                            {{QStringLiteral("name"), QStringLiteral("echo")},
                                             {QStringLiteral("arguments"), QJsonObject{{QStringLiteral("say"), QStringLiteral("hi")}}}}));
        QCOMPARE(ok.value(QStringLiteral("id")).toInt(), 5);
        QCOMPARE(ok.value(QStringLiteral("result")).toObject().value(QStringLiteral("isError")).toBool(), false);
        const QJsonObject failed = send(request(6, QStringLiteral("tools/call"),
                                                {{QStringLiteral("name"), QStringLiteral("echo")},
                                                 {QStringLiteral("arguments"), QJsonObject{{QStringLiteral("fail"), true}}}}));
        QCOMPARE(failed.value(QStringLiteral("result")).toObject().value(QStringLiteral("isError")).toBool(), true);
    }

    // ── the tools, in the real Studio ──

    void theToolsAreListedWithSchemasClientsAccept()
    {
        Studio s;
        QVERIFY(s.open());
        const QJsonArray tools = s.client.ask(QStringLiteral("tools/list")).value(QStringLiteral("result")).toObject()
                                     .value(QStringLiteral("tools")).toArray();
        QSet<QString> names;
        const QRegularExpression valid(QStringLiteral("^[a-zA-Z0-9_-]{1,64}$"));
        for (const QJsonValue &t : tools) {
            const QJsonObject tool = t.toObject();
            names << tool.value(QStringLiteral("name")).toString();
            QVERIFY(valid.match(tool.value(QStringLiteral("name")).toString()).hasMatch());
            QVERIFY(!tool.value(QStringLiteral("description")).toString().isEmpty());
            QCOMPARE(tool.value(QStringLiteral("inputSchema")).toObject().value(QStringLiteral("type")).toString(),
                     QStringLiteral("object"));
        }
        for (const char *n : {"scene_read", "kinds", "scene_set", "entity_add", "entity_set", "entity_remove",
                              "behaviour_write", "play", "stop", "input_set", "wait", "view_set", "frame_capture",
                              "log_read"})
            QVERIFY2(names.contains(QLatin1String(n)), n);
    }

    void anAgentReadsTheSceneAndWhatCanBeAdded()
    {
        Studio s;
        QVERIFY(s.open());
        const QJsonObject read = s.client.tool(QStringLiteral("scene_read"));
        QVERIFY(!read.value(QStringLiteral("isError")).toBool());
        const QJsonObject scene = read.value(QStringLiteral("json")).toObject();
        QCOMPARE(scene.value(QStringLiteral("qml")).toString(), s.sceneText());
        QStringList names;
        for (const QJsonValue &e : scene.value(QStringLiteral("entities")).toArray())
            names << e.toObject().value(QStringLiteral("fields")).toObject().value(QStringLiteral("name")).toString();
        QVERIFY(names.contains(QStringLiteral("Plinth")) && names.contains(QStringLiteral("Orb")));
        // A vector is [x, y, z]; a colour #rrggbb.
        for (const QJsonValue &e : scene.value(QStringLiteral("entities")).toArray()) {
            const QJsonObject f = e.toObject().value(QStringLiteral("fields")).toObject();
            if (f.value(QStringLiteral("name")) == QStringLiteral("Plinth")) {
                QCOMPARE(f.value(QStringLiteral("position")).toArray(), (QJsonArray{0, 0.25, 0}));
                QCOMPARE(f.value(QStringLiteral("colour")).toString(), QStringLiteral("#2b3140"));
            }
        }

        const QJsonObject kinds = s.client.tool(QStringLiteral("kinds")).value(QStringLiteral("json")).toObject();
        QVERIFY(kinds.contains(QStringLiteral("Player")) && kinds.contains(QStringLiteral("Behaviour")));
        bool sawForm = false;
        bool sawMass = false;
        for (const QJsonValue &f : kinds.value(QStringLiteral("Shape")).toArray()) {
            const QJsonObject field = f.toObject();
            if (field.value(QStringLiteral("name")) == QStringLiteral("form")) {
                sawForm = field.value(QStringLiteral("choices")).toArray().contains(QStringLiteral("sphere"));
                QCOMPARE(field.value(QStringLiteral("default")).toString(), QStringLiteral("cube"));
            }
            if (field.value(QStringLiteral("name")) == QStringLiteral("mass"))
                sawMass = field.value(QStringLiteral("range")).toArray() == (QJsonArray{0.1, 100});
        }
        QVERIFY(sawForm && sawMass);
    }

    void anAgentAddsChangesAndRemovesWhatTheCreatorThenSees()
    {
        Studio s;
        QVERIFY(s.open());
        const QString before = s.sceneText();

        // Added, with fields, as the creator would see it: in the list, unsaved, the file not touched.
        QJsonObject r = s.client.tool(QStringLiteral("entity_add"),
                                      {{QStringLiteral("kind"), QStringLiteral("Shape")},
                                       {QStringLiteral("name"), QStringLiteral("Crate")},
                                       {QStringLiteral("fields"), QJsonObject{{QStringLiteral("position"), QJsonArray{2, 3, 0}},
                                                                              {QStringLiteral("body"), QStringLiteral("dynamic")},
                                                                              {QStringLiteral("colour"), QStringLiteral("#ff8800")}}}});
        QVERIFY2(!r.value(QStringLiteral("isError")).toBool(), qPrintable(r.value(QStringLiteral("text")).toString()));
        QObject *crate = s.entity(QStringLiteral("Crate"));
        QVERIFY(crate);
        QCOMPARE(crate->property("position").value<QVector3D>(), QVector3D(2, 3, 0));
        QCOMPARE(crate->property("body").toString(), QStringLiteral("dynamic"));
        QVERIFY(s.window->property("dirty").toBool());
        QVERIFY(s.window->property("notice").toString().contains(QStringLiteral("Crate")));
        QVERIFY(!QFile::exists(QDir(s.project.path()).filePath(QStringLiteral("scenes/untitled.qml"))));

        // A value outside what the field allows is refused, and nothing of that call is kept: the fields are taken in
        // name order, so the good colour is set before the bad roughness is met, and must be undone.
        r = s.client.tool(QStringLiteral("entity_set"),
                          {{QStringLiteral("name"), QStringLiteral("Crate")},
                           {QStringLiteral("fields"), QJsonObject{{QStringLiteral("colour"), QStringLiteral("#123456")},
                                                                  {QStringLiteral("roughness"), 5}}}});
        QVERIFY(r.value(QStringLiteral("isError")).toBool());
        QVERIFY(r.value(QStringLiteral("text")).toString().contains(QStringLiteral("roughness")));
        QCOMPARE(crate->property("colour").value<QColor>().name(), QStringLiteral("#ff8800"));
        QCOMPARE(crate->property("roughness").toDouble(), 0.4);
        r = s.client.tool(QStringLiteral("entity_set"), {{QStringLiteral("name"), QStringLiteral("Crate")},
                                                         {QStringLiteral("fields"), QJsonObject{{QStringLiteral("form"), QStringLiteral("blob")}}}});
        QVERIFY(r.value(QStringLiteral("isError")).toBool());
        r = s.client.tool(QStringLiteral("entity_add"), {{QStringLiteral("kind"), QStringLiteral("Dragon")}});
        QVERIFY(r.value(QStringLiteral("isError")).toBool());

        // Changed: one value, one line.
        r = s.client.tool(QStringLiteral("entity_set"), {{QStringLiteral("name"), QStringLiteral("Crate")},
                                                         {QStringLiteral("fields"), QJsonObject{{QStringLiteral("roughness"), 0.9}}}});
        QVERIFY(!r.value(QStringLiteral("isError")).toBool());
        QVERIFY(s.sceneText().contains(QStringLiteral("roughness: 0.9")));

        // A name taken is made unique; removing takes it out.
        r = s.client.tool(QStringLiteral("entity_add"), {{QStringLiteral("kind"), QStringLiteral("Shape")},
                                                         {QStringLiteral("name"), QStringLiteral("Crate")}});
        QCOMPARE(r.value(QStringLiteral("json")).toObject().value(QStringLiteral("fields")).toObject()
                     .value(QStringLiteral("name")).toString(),
                 QStringLiteral("Crate 2"));
        QVERIFY(!s.client.tool(QStringLiteral("entity_remove"), {{QStringLiteral("name"), QStringLiteral("Crate 2")}})
                     .value(QStringLiteral("isError")).toBool());
        QTRY_VERIFY(!s.entity(QStringLiteral("Crate 2")));

        // The scene's own look.
        QVERIFY(!s.client.tool(QStringLiteral("scene_set"), {{QStringLiteral("fields"), QJsonObject{{QStringLiteral("gravity"), 4}}}})
                     .value(QStringLiteral("isError")).toBool());
        QCOMPARE(s.window->property("scene").value<QObject *>()->property("gravity").toDouble(), 4.0);

        // And taken out again, the scene is what it was, but for the gravity.
        QVERIFY(!s.client.tool(QStringLiteral("entity_remove"), {{QStringLiteral("name"), QStringLiteral("Crate")}})
                     .value(QStringLiteral("isError")).toBool());
        QTRY_VERIFY(!s.entity(QStringLiteral("Crate")));
        s.client.tool(QStringLiteral("scene_set"), {{QStringLiteral("fields"), QJsonObject{{QStringLiteral("gravity"), 9.81}}}});
        QCOMPARE(s.sceneText(), before);
    }

    void anAgentWritesLogicAndIsToldWhetherItBuilds()
    {
        Studio s;
        QVERIFY(s.open());
        const QString code = QStringLiteral(
            "import QQ\nLogic {\n    onFrame: (dt) => target.eulerRotation.y += 90 * dt\n}\n");
        QJsonObject r = s.client.tool(QStringLiteral("behaviour_write"),
                                      {{QStringLiteral("file"), QStringLiteral("spin")},
                                       {QStringLiteral("code"), code},
                                       {QStringLiteral("target"), QStringLiteral("Plinth")}});
        QVERIFY2(!r.value(QStringLiteral("isError")).toBool(), qPrintable(r.value(QStringLiteral("text")).toString()));
        QCOMPARE(r.value(QStringLiteral("json")).toObject().value(QStringLiteral("file")).toString(), QStringLiteral("logic/spin.qml"));
        {
            // Closed again at once: a file held open here could not be replaced by the next write.
            QFile written(QDir(s.project.path()).filePath(QStringLiteral("logic/spin.qml")));
            QVERIFY(written.open(QIODevice::ReadOnly));
            QCOMPARE(QString::fromUtf8(written.readAll()), code);
        }
        QObject *behaviour = s.entity(QStringLiteral("Plinth logic"));
        QVERIFY(behaviour);
        QCOMPARE(behaviour->property("target").toString(), QStringLiteral("Plinth"));

        // Written again: the same Behaviour, not a second one.
        r = s.client.tool(QStringLiteral("behaviour_write"), {{QStringLiteral("file"), QStringLiteral("spin")},
                                                             {QStringLiteral("code"), code},
                                                             {QStringLiteral("target"), QStringLiteral("Plinth")}});
        QVERIFY2(r.value(QStringLiteral("json")).toObject().value(QStringLiteral("behaviour")).toString() == QStringLiteral("already attached"),
                 qPrintable(r.value(QStringLiteral("text")).toString()));

        // Code that does not build: written (the creator can see it), and the agent is told why.
        r = s.client.tool(QStringLiteral("behaviour_write"), {{QStringLiteral("file"), QStringLiteral("broken")},
                                                             {QStringLiteral("code"), QStringLiteral("import QQ\nLogic {\n    onFrame: (dt) =>\n")}});
        QVERIFY(r.value(QStringLiteral("isError")).toBool());
        QVERIFY(!r.value(QStringLiteral("json")).toObject().value(QStringLiteral("errors")).toArray().isEmpty());
        QVERIFY(QFile::exists(QDir(s.project.path()).filePath(QStringLiteral("logic/broken.qml"))));
    }

    void anAgentPlaysLooksAndStopsAndTheSceneComesBack()
    {
        Studio s;
        QVERIFY(s.open());
        s.client.tool(QStringLiteral("entity_add"),
                      {{QStringLiteral("kind"), QStringLiteral("Shape")}, {QStringLiteral("name"), QStringLiteral("Crate")},
                       {QStringLiteral("fields"), QJsonObject{{QStringLiteral("position"), QJsonArray{3, 4, 0}},
                                                              {QStringLiteral("body"), QStringLiteral("dynamic")}}}});
        s.client.tool(QStringLiteral("entity_add"),
                      {{QStringLiteral("kind"), QStringLiteral("Player")}, {QStringLiteral("name"), QStringLiteral("Hero")},
                       {QStringLiteral("fields"), QJsonObject{{QStringLiteral("position"), QJsonArray{0, 0, 4}}}}});
        const QString before = s.sceneText();

        // Play; edits wait for stop.
        QJsonObject r = s.client.tool(QStringLiteral("play"));
        QVERIFY2(!r.value(QStringLiteral("isError")).toBool(), qPrintable(r.value(QStringLiteral("text")).toString()));
        QVERIFY(s.window->property("playing").toBool());
        QVERIFY(s.client.tool(QStringLiteral("entity_set"), {{QStringLiteral("name"), QStringLiteral("Crate")},
                                                             {QStringLiteral("fields"), QJsonObject{{QStringLiteral("mass"), 2}}}})
                    .value(QStringLiteral("isError")).toBool());

        // Wait: the crate has fallen to the ground, as the agent is told.
        r = s.client.tool(QStringLiteral("wait"), {{QStringLiteral("ms"), 2500}});
        const QJsonArray crate = r.value(QStringLiteral("json")).toObject().value(QStringLiteral("dynamicBodies")).toObject()
                                     .value(QStringLiteral("Crate")).toArray();
        QVERIFY2(crate.size() == 3 && qAbs(crate[1].toDouble() - 0.5) < 0.1, qPrintable(r.value(QStringLiteral("text")).toString()));

        // The agent's hands: forward, for a second, moves the player toward -z.
        const QJsonArray startFeet = r.value(QStringLiteral("json")).toObject().value(QStringLiteral("player")).toObject()
                                         .value(QStringLiteral("feet")).toArray();
        QVERIFY(!s.client.tool(QStringLiteral("input_set"), {{QStringLiteral("move"), QJsonArray{0, 1}}})
                     .value(QStringLiteral("isError")).toBool());
        r = s.client.tool(QStringLiteral("wait"), {{QStringLiteral("ms"), 1000}});
        s.client.tool(QStringLiteral("input_set"), {{QStringLiteral("move"), QJsonArray{0, 0}}});
        const QJsonArray feet = r.value(QStringLiteral("json")).toObject().value(QStringLiteral("player")).toObject()
                                    .value(QStringLiteral("feet")).toArray();
        QVERIFY2(startFeet[2].toDouble() - feet[2].toDouble() > 2, qPrintable(r.value(QStringLiteral("text")).toString()));

        // A frame: an image of the viewport, which is the world, not an empty window.
        r = s.client.tool(QStringLiteral("frame_capture"), {{QStringLiteral("width"), 800}});
        const QJsonObject image = r.value(QStringLiteral("image")).toObject();
        QCOMPARE(image.value(QStringLiteral("mimeType")).toString(), QStringLiteral("image/png"));
        const QImage frame = QImage::fromData(QByteArray::fromBase64(image.value(QStringLiteral("data")).toString().toLatin1()), "PNG");
        QCOMPARE(frame.width(), 800);
        QSet<QRgb> colours;
        for (int y = 0; y < frame.height(); y += 4) {
            for (int x = 0; x < frame.width(); x += 4)
                colours.insert(frame.pixel(x, y) & 0xF0F0F0);
        }
        QVERIFY2(colours.size() > 100, qPrintable(QString::number(colours.size())));

        // Stop: the scene exactly as it was, and the agent's hands let go.
        r = s.client.tool(QStringLiteral("stop"));
        QVERIFY(!s.window->property("playing").toBool());
        QCOMPARE(s.sceneText(), before);
        QVERIFY(s.client.tool(QStringLiteral("input_set"), {{QStringLiteral("move"), QJsonArray{0, 1}}})
                    .value(QStringLiteral("isError")).toBool());
    }

    void logicThatThrowsInPlayIsInTheLog()
    {
        Studio s;
        QVERIFY(s.open());
        const qint64 start = s.client.tool(QStringLiteral("log_read"), {{QStringLiteral("since"), 0}})
                                 .value(QStringLiteral("json")).toObject().value(QStringLiteral("next")).toInteger();
        s.client.tool(QStringLiteral("behaviour_write"),
                      {{QStringLiteral("file"), QStringLiteral("oops")},
                       {QStringLiteral("code"), QStringLiteral("import QQ\nLogic {\n    onFrame: (dt) => target.nothing.here = 1\n}\n")},
                       {QStringLiteral("target"), QStringLiteral("Plinth")}});
        s.client.tool(QStringLiteral("play"));
        s.client.tool(QStringLiteral("wait"), {{QStringLiteral("ms"), 600}});
        s.client.tool(QStringLiteral("stop"));
        const QJsonObject log = s.client.tool(QStringLiteral("log_read"), {{QStringLiteral("since"), start}})
                                    .value(QStringLiteral("json")).toObject();
        QStringList texts;
        for (const QJsonValue &e : log.value(QStringLiteral("entries")).toArray())
            texts << e.toObject().value(QStringLiteral("text")).toString();
        const QString all = texts.join(QLatin1Char('\n'));
        QVERIFY2(all.contains(QStringLiteral("oops.qml")) && all.contains(QStringLiteral("TypeError")), qPrintable(all));
        // Thrown every frame, said once, and counted.
        int typeErrors = 0;
        int times = 0;
        for (const QJsonValue &e : log.value(QStringLiteral("entries")).toArray()) {
            if (e.toObject().value(QStringLiteral("text")).toString().contains(QStringLiteral("TypeError"))) {
                ++typeErrors;
                times = e.toObject().value(QStringLiteral("times")).toInt(1);
            }
        }
        QCOMPARE(typeErrors, 1);
        QVERIFY2(times > 5, qPrintable(QString::number(times)));
        QVERIFY2(all.contains(QStringLiteral("Playing")), qPrintable(all));  // the Studio's notices are there too
        QVERIFY(log.value(QStringLiteral("next")).toInteger() > start);
    }

    // ── the design loop, against a stand-in for the provider ──

    void aDescriptionIsBuiltPlayedLookedAtAndLeftForTheCreator()
    {
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        qq::DesignLoop designer(&s.server, s.window, io);
        QCOMPARE(designer.setKey(d.key), QString());
        QVERIFY(designer.hasKey());
        const qint64 logStart = qq::StudioLog::instance()->since(0).value(QStringLiteral("next")).toInteger();
        const QString before = s.sceneText();

        using F = FakeAnthropic;
        d.api.script = {
            F::said({F::text(QStringLiteral("Brief: a crate drops onto the plinth.")),
                     F::use(QStringLiteral("t1"), QStringLiteral("entity_add"),
                            {{QStringLiteral("kind"), QStringLiteral("Shape")}, {QStringLiteral("name"), QStringLiteral("Crate")},
                             {QStringLiteral("fields"), QJsonObject{{QStringLiteral("position"), QJsonArray{0, 4, 0}},
                                                                    {QStringLiteral("body"), QStringLiteral("dynamic")}}}})},
                    QStringLiteral("tool_use")),
            F::said({F::use(QStringLiteral("t2"), QStringLiteral("play")),
                     F::use(QStringLiteral("t3"), QStringLiteral("wait"), {{QStringLiteral("ms"), 1200}}),
                     F::use(QStringLiteral("t4"), QStringLiteral("frame_capture"), {{QStringLiteral("width"), 640}})},
                    QStringLiteral("tool_use")),
            F::said({F::use(QStringLiteral("t5"), QStringLiteral("stop"))}, QStringLiteral("tool_use")),
            F::said({F::text(QStringLiteral("Built a crate that falls onto the plinth."))}, QStringLiteral("end_turn")),
        };
        QSignalSpy finished(&designer, &qq::DesignLoop::finished);
        designer.start(QStringLiteral("A crate falls onto a plinth."));
        QVERIFY(designer.running());
        QVERIFY(finished.wait(30000));
        QCOMPARE(finished.first().first().toString(), QStringLiteral("done"));
        QCOMPARE(d.api.requests.size(), 4);

        // What was asked, every time: the key in its header only, the API version, the fallback opt-in, the model and
        // its settings, the Studio's fourteen tools.
        for (const FakeAnthropic::Request &r : d.api.requests) {
            QCOMPARE(r.headers.value("x-api-key"), d.key.toUtf8());
            QCOMPARE(r.headers.value("anthropic-version"), QByteArray("2023-06-01"));
            QVERIFY(r.headers.value("anthropic-beta").contains("server-side-fallback-2026-07-01"));
            QVERIFY(!r.raw.contains(d.key.toUtf8()));
            QCOMPARE(r.body.value(QStringLiteral("model")).toString(), QStringLiteral("claude-opus-5-5"));
            QCOMPARE(r.body.value(QStringLiteral("fallbacks")).toString(), QStringLiteral("default"));
            QCOMPARE(r.body.value(QStringLiteral("output_config")).toObject().value(QStringLiteral("effort")).toString(), QStringLiteral("high"));
            QCOMPARE(r.body.value(QStringLiteral("thinking")).toObject().value(QStringLiteral("type")).toString(), QStringLiteral("adaptive"));
            QCOMPARE(r.body.value(QStringLiteral("tools")).toArray().size(), 14);
            QVERIFY(r.body.value(QStringLiteral("system")).toString().contains(QStringLiteral("design brief")));
        }

        // Append-only: each conversation begins with the one before.
        for (int i = 1; i < d.api.requests.size(); ++i) {
            const QJsonArray earlier = d.api.requests[i - 1].body.value(QStringLiteral("messages")).toArray();
            const QJsonArray later = d.api.requests[i].body.value(QStringLiteral("messages")).toArray();
            QVERIFY(later.size() > earlier.size());
            for (int m = 0; m < earlier.size(); ++m)
                QCOMPARE(later[m], earlier[m]);
        }

        // The second turn's three results went back together, in order, the frame as an image.
        const QJsonArray last = d.api.requests[2].body.value(QStringLiteral("messages")).toArray();
        const QJsonArray results = last.last().toObject().value(QStringLiteral("content")).toArray();
        QCOMPARE(results.size(), 3);
        QCOMPARE(results[0].toObject().value(QStringLiteral("tool_use_id")).toString(), QStringLiteral("t2"));
        QCOMPARE(results[2].toObject().value(QStringLiteral("tool_use_id")).toString(), QStringLiteral("t4"));
        const QJsonObject image = results[2].toObject().value(QStringLiteral("content")).toArray().first().toObject();
        QCOMPARE(image.value(QStringLiteral("type")).toString(), QStringLiteral("image"));
        const QJsonObject source = image.value(QStringLiteral("source")).toObject();
        QCOMPARE(source.value(QStringLiteral("media_type")).toString(), QStringLiteral("image/png"));
        QCOMPARE(QImage::fromData(QByteArray::fromBase64(source.value(QStringLiteral("data")).toString().toLatin1()), "PNG").width(), 640);

        // What the creator is left with: the crate, unsaved, play stopped, every step shown, the key nowhere in the log.
        QVERIFY(s.entity(QStringLiteral("Crate")));
        QVERIFY(s.window->property("dirty").toBool());
        QVERIFY(!s.window->property("playing").toBool());
        bool sawFrame = false;
        for (const QVariant &step : designer.steps())
            sawFrame |= step.toMap().value(QStringLiteral("image")).toString().startsWith(QStringLiteral("data:image/png;base64,"));
        QVERIFY(sawFrame);
        QVERIFY(designer.steps().first().toMap().value(QStringLiteral("text")).toString().startsWith(QStringLiteral("Brief")));
        for (const QJsonValue &e : qq::StudioLog::instance()->since(logStart).value(QStringLiteral("entries")).toArray())
            QVERIFY(!e.toObject().value(QStringLiteral("text")).toString().contains(d.key));

        // What it used on the creator's key, as the provider counted it: four answers' worth.
        const QVariantMap used = designer.usage();
        QCOMPARE(used.value(QStringLiteral("answers")).toInt(), 4);
        QCOMPARE(used.value(QStringLiteral("input")).toLongLong(), 400);
        QCOMPARE(used.value(QStringLiteral("cacheWrite")).toLongLong(), 200);
        QCOMPARE(used.value(QStringLiteral("cacheRead")).toLongLong(), 120);
        QCOMPARE(used.value(QStringLiteral("output")).toLongLong(), 80);

        // And all of it can be taken back.
        QVERIFY(designer.canUndo());
        QVERIFY(designer.undo());
        QTRY_VERIFY(!s.entity(QStringLiteral("Crate")));
        QCOMPARE(s.sceneText(), before);
    }

    void theLoopWaitsOutABusyProviderAndStopsWhenItShould()
    {
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        qq::DesignLoop designer(&s.server, s.window, s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO"));
        QSignalSpy finished(&designer, &qq::DesignLoop::finished);
        using F = FakeAnthropic;

        // No key: nothing is sent.
        designer.start(QStringLiteral("anything"));
        QVERIFY(!designer.running());
        QVERIFY(d.api.requests.isEmpty());
        QVERIFY(designer.status().contains(QStringLiteral("key")));
        designer.setKey(d.key);

        // Rate limited, then overloaded: waited out, as retry-after says.
        d.api.script = {F::error(429, QStringLiteral("rate_limit_error"), "0"), F::error(529, QStringLiteral("overloaded_error"), "0"),
                        F::said({F::text(QStringLiteral("ok"))}, QStringLiteral("end_turn"))};
        designer.start(QStringLiteral("anything"));
        QVERIFY(finished.wait(20000));
        QCOMPARE(finished.takeFirst().first().toString(), QStringLiteral("done"));
        QCOMPARE(d.api.requests.size(), 3);

        // A refused key is said, and not tried again.
        d.api.requests.clear();
        d.api.script = {F::error(401, QStringLiteral("authentication_error"))};
        designer.start(QStringLiteral("anything"));
        QVERIFY(finished.wait(20000));
        QVERIFY(finished.takeFirst().first().toString().contains(QStringLiteral("refused the key")));
        QCOMPARE(d.api.requests.size(), 1);

        // A refusal ends the run.
        d.api.script = {F::said({}, QStringLiteral("refusal"))};
        designer.start(QStringLiteral("anything"));
        QVERIFY(finished.wait(20000));
        QVERIFY(finished.takeFirst().first().toString().contains(QStringLiteral("declined")));

        // An answer cut off at its limit: its tool call is not run.
        d.api.script = {F::said({F::use(QStringLiteral("t1"), QStringLiteral("entity_add"),
                                        {{QStringLiteral("kind"), QStringLiteral("Lamp")}, {QStringLiteral("name"), QStringLiteral("Cut")}})},
                                QStringLiteral("max_tokens"))};
        designer.start(QStringLiteral("anything"));
        QVERIFY(finished.wait(20000));
        QVERIFY(finished.takeFirst().first().toString().contains(QStringLiteral("cut off")));
        QVERIFY(!s.entity(QStringLiteral("Cut")));

        // Stopped by the creator: the request in flight is dropped, and nothing after it runs.
        d.api.requests.clear();
        d.api.script = {F::error(529, QStringLiteral("overloaded_error"), "3"),
                        F::said({F::use(QStringLiteral("t1"), QStringLiteral("entity_add"),
                                        {{QStringLiteral("kind"), QStringLiteral("Lamp")}, {QStringLiteral("name"), QStringLiteral("Late")}})},
                                QStringLiteral("tool_use"))};
        designer.start(QStringLiteral("anything"));
        QTRY_COMPARE(d.api.requests.size(), 1);
        designer.stop();
        QCOMPARE(finished.takeFirst().first().toString(), QStringLiteral("stopped"));
        QTest::qWait(3500);
        QCOMPARE(d.api.requests.size(), 1);
        QVERIFY(!s.entity(QStringLiteral("Late")));

        // Forgotten: no key, no run.
        designer.forgetKey();
        QVERIFY(!designer.hasKey());
    }

    void theRealProviderIsReachedOverTls()
    {
        // Only when asked (QQ_LIVE_ENDPOINT=1): one request to Anthropic's real endpoint with a key that is not one, to
        // see that TLS and the address work from here. It is refused before anything is run or billed.
        if (qEnvironmentVariable("QQ_LIVE_ENDPOINT") != QStringLiteral("1"))
            QSKIP("Set QQ_LIVE_ENDPOINT=1 to reach the real provider.");
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        qunsetenv("QQ_ANTHROPIC_URL");
        qq::DesignLoop designer(&s.server, s.window, s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO"));
        designer.setKey(QStringLiteral("sk-ant-not-a-key"));
        QSignalSpy finished(&designer, &qq::DesignLoop::finished);
        designer.start(QStringLiteral("probe"));
        QVERIFY(finished.wait(60000));
        QVERIFY2(finished.first().first().toString().contains(QStringLiteral("refused the key")),
                 qPrintable(finished.first().first().toString()));
    }

    void onlyTheCreatorsOwnKeyIsEverUsed()
    {
        // ADR-086: generation is paid by the person using it. A key in the environment (as a server or a build might
        // set one) is never taken up: with no key of the creator's own, nothing is sent at all.
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        const QByteArray elsewhere = "sk-ant-from-the-environment-not-the-creators";
        qputenv("ANTHROPIC_API_KEY", elsewhere);
        qq::DesignLoop designer(&s.server, s.window, s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO"));
        QVERIFY(!designer.hasKey());
        designer.start(QStringLiteral("a crate"));
        QVERIFY(!designer.running());
        QTest::qWait(300);
        QVERIFY(d.api.requests.isEmpty());

        // With the creator's key, that key and only that key is sent.
        using F = FakeAnthropic;
        designer.setKey(d.key);
        d.api.script = {F::said({F::text(QStringLiteral("ok"))}, QStringLiteral("end_turn"))};
        QSignalSpy finished(&designer, &qq::DesignLoop::finished);
        designer.start(QStringLiteral("a crate"));
        QVERIFY(finished.wait(20000));
        QCOMPARE(d.api.requests.size(), 1);
        QCOMPARE(d.api.requests.first().headers.value("x-api-key"), d.key.toUtf8());
        QVERIFY(!d.api.requests.first().raw.contains(elsewhere));
        qunsetenv("ANTHROPIC_API_KEY");

        // The panel says who pays before any key is given.
        designer.forgetKey();
        s.window->setProperty("designer", QVariant::fromValue<QObject *>(&designer));
        s.window->setProperty("agentOpen", true);
        QQuickItem *whoPays = nullptr;
        QTRY_VERIFY((whoPays = s.window->findChild<QQuickItem *>(QStringLiteral("agentWhoPays"))) && whoPays->isVisible());
        QVERIFY(whoPays->property("text").toString().contains(QStringLiteral("your own Anthropic account")));
        QVERIFY(whoPays->property("text").toString().contains(QStringLiteral("does not pay")));
        s.window->setProperty("designer", QVariant::fromValue<QObject *>(nullptr));
    }

    void theAgentPanelAsksForAKeyThenForADescription()
    {
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        qq::DesignLoop designer(&s.server, s.window, s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO"));
        s.window->setProperty("designer", QVariant::fromValue<QObject *>(&designer));
        s.window->setProperty("agentOpen", true);
        auto visible = [&s](const char *name) {
            auto *item = s.window->findChild<QQuickItem *>(QLatin1String(name));
            return item && item->isVisible();
        };
        QTRY_VERIFY(visible("agentStatus"));
        QVERIFY(visible("agentKey"));  // no key yet: asked for
        auto *start = s.window->findChild<QQuickItem *>(QStringLiteral("agentStart"));
        QVERIFY(start && !start->isEnabled());

        // Given a key and a description, it can start; the key field is gone, never to show the key again.
        designer.setKey(d.key);
        QTRY_VERIFY(!visible("agentKey"));
        s.window->findChild<QQuickItem *>(QStringLiteral("agentBrief"))->setProperty("text", QStringLiteral("A crate."));
        QTRY_VERIFY(start->isEnabled());
        s.window->setProperty("designer", QVariant::fromValue<QObject *>(nullptr));
    }

    void whatAFallbackReplacedIsNotRunOrSentBack()
    {
        Studio s;
        QVERIFY(s.open());
        DesignerSetup d;
        qq::DesignLoop designer(&s.server, s.window, s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO"));
        designer.setKey(d.key);
        using F = FakeAnthropic;
        const QJsonObject thinking{{QStringLiteral("type"), QStringLiteral("thinking")}, {QStringLiteral("thinking"), QString()},
                                   {QStringLiteral("signature"), QStringLiteral("sig")}};
        const QJsonObject fallback{{QStringLiteral("type"), QStringLiteral("fallback")},
                                   {QStringLiteral("from"), QJsonObject{{QStringLiteral("model"), QStringLiteral("claude-opus-5-5")}}},
                                   {QStringLiteral("to"), QJsonObject{{QStringLiteral("model"), QStringLiteral("claude-opus-4-8")}}}};
        d.api.script = {
            F::said({thinking,
                     F::use(QStringLiteral("old"), QStringLiteral("entity_add"),
                            {{QStringLiteral("kind"), QStringLiteral("Lamp")}, {QStringLiteral("name"), QStringLiteral("Replaced")}}),
                     fallback, F::text(QStringLiteral("Continuing.")),
                     F::use(QStringLiteral("new"), QStringLiteral("entity_add"),
                            {{QStringLiteral("kind"), QStringLiteral("Lamp")}, {QStringLiteral("name"), QStringLiteral("Kept")}})},
                    QStringLiteral("tool_use")),
            F::said({F::text(QStringLiteral("Done."))}, QStringLiteral("end_turn")),
        };
        QSignalSpy finished(&designer, &qq::DesignLoop::finished);
        designer.start(QStringLiteral("a lamp"));
        QVERIFY(finished.wait(20000));
        QVERIFY(s.entity(QStringLiteral("Kept")));
        QVERIFY(!s.entity(QStringLiteral("Replaced")));
        const QJsonArray sent = d.api.requests[1].body.value(QStringLiteral("messages")).toArray()[1].toObject()
                                    .value(QStringLiteral("content")).toArray();
        QStringList types;
        for (const QJsonValue &b : sent)
            types << b.toObject().value(QStringLiteral("type")).toString();
        QCOMPARE(types, (QStringList{QStringLiteral("fallback"), QStringLiteral("text"), QStringLiteral("tool_use")}));
    }

    // ── the real programs ──

    void qqMcpRelaysAClientToARunningStudio()
    {
        const QString pipe = QStringLiteral("qq-test-%1").arg(QRandomGenerator::global()->generate());
        QProcessEnvironment env = QProcessEnvironment::systemEnvironment();
        env.insert(QStringLiteral("QQ_MCP_PIPE"), pipe);

        QProcess bridge;
        bridge.setProcessEnvironment(env);
        bridge.start(QStringLiteral(QQ_MCP));
        QVERIFY(bridge.waitForStarted());

        // No Studio yet: said so, as an answer to the request.
        bridge.write(request(1, QStringLiteral("initialize"), {{QStringLiteral("protocolVersion"), QStringLiteral("2025-06-18")}}) + '\n');
        QJsonObject r = readLine(bridge);
        QCOMPARE(r.value(QStringLiteral("id")).toInt(), 1);
        QVERIFY(r.value(QStringLiteral("error")).toObject().value(QStringLiteral("message")).toString().contains(QStringLiteral("not running")));

        // A Studio opens: the next request finds it.
        QTemporaryDir project;
        QProcess studio;
        studio.setProcessEnvironment(env);
        studio.start(QStringLiteral(QQ_STUDIO), {QStringLiteral("--project"), project.path()});
        QVERIFY(studio.waitForStarted());
        QVERIFY(QTest::qWaitFor([&pipe] {
            QLocalSocket probe;
            probe.connectToServer(pipe);
            return probe.waitForConnected(200);
        }, 30000));

        bridge.write(request(2, QStringLiteral("initialize"), {{QStringLiteral("protocolVersion"), QStringLiteral("2025-06-18")}}) + '\n');
        r = readLine(bridge);
        QCOMPARE(r.value(QStringLiteral("result")).toObject().value(QStringLiteral("serverInfo")).toObject()
                     .value(QStringLiteral("name")).toString(),
                 QStringLiteral("qq-studio"));
        bridge.write(QByteArray(R"({"jsonrpc":"2.0","method":"notifications/initialized"})") + '\n');
        bridge.write(request(3, QStringLiteral("tools/call"),
                             {{QStringLiteral("name"), QStringLiteral("entity_add")},
                              {QStringLiteral("arguments"), QJsonObject{{QStringLiteral("kind"), QStringLiteral("Lamp")},
                                                                        {QStringLiteral("name"), QStringLiteral("Glow")}}}}) + '\n');
        r = readLine(bridge);
        QCOMPARE(r.value(QStringLiteral("id")).toInt(), 3);  // the notification had no answer, so this is the next line
        QCOMPARE(r.value(QStringLiteral("result")).toObject().value(QStringLiteral("isError")).toBool(), false);
        bridge.write(request(4, QStringLiteral("tools/call"), {{QStringLiteral("name"), QStringLiteral("scene_read")}}) + '\n');
        r = readLine(bridge);
        QVERIFY(r.value(QStringLiteral("result")).toObject().value(QStringLiteral("content")).toArray().first().toObject()
                    .value(QStringLiteral("text")).toString().contains(QStringLiteral("Glow")));

        // A second Studio on the same pipe stays out of the way, and says so (on standard error, which a Windows GUI
        // program writes to only when asked); a client connecting after it still reaches the first.
        QProcess second;
        QProcessEnvironment loud = env;
        loud.insert(QStringLiteral("QT_FORCE_STDERR_LOGGING"), QStringLiteral("1"));
        second.setProcessEnvironment(loud);
        second.setProcessChannelMode(QProcess::MergedChannels);
        second.start(QStringLiteral(QQ_STUDIO), {QStringLiteral("--project"), project.path()});
        QVERIFY(second.waitForStarted());
        QByteArray said;
        QVERIFY2(QTest::qWaitFor([&] { said += second.readAll(); return said.contains("already offers"); }, 30000),
                 said.constData());
        QProcess another;
        another.setProcessEnvironment(env);
        another.start(QStringLiteral(QQ_MCP));
        QVERIFY(another.waitForStarted());
        another.write(request(1, QStringLiteral("tools/call"), {{QStringLiteral("name"), QStringLiteral("scene_read")}}) + '\n');
        r = readLine(another);
        QVERIFY(r.value(QStringLiteral("result")).toObject().value(QStringLiteral("content")).toArray().first().toObject()
                    .value(QStringLiteral("text")).toString().contains(QStringLiteral("Glow")));
        another.closeWriteChannel();
        QVERIFY(another.waitForFinished(10000));
        second.kill();
        second.waitForFinished();

        // The end of the client's input ends the bridge.
        bridge.closeWriteChannel();
        QVERIFY(bridge.waitForFinished(10000));
        QCOMPARE(bridge.exitCode(), 0);
        studio.kill();
        studio.waitForFinished();
    }
};

QTEST_MAIN(TstAgent)
#include "tst_agent.moc"
