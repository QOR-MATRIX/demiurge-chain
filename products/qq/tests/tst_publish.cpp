// Does QQ Studio publish through the launcher, and only through it? (DIRECTION P3.4, ADR-088)
//
// A stand-in launcher, a local server speaking the QOR host protocol, plays the launcher's part: it reads what the
// Studio's LauncherLink sends and answers as the launcher would. The launcher's own side (the spawn, the token's
// checks, the operating system's word on who connected, the dialog, the chain) is checked in the launcher's tests
// (tools/qor-launcher/src-tauri/src/host/tests.rs).

#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalServer>
#include <QLocalSocket>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QTest>
#include <QtEndian>
#include <QtQml/qqmlextensionplugin.h>

#include <functional>

#include "gpu.h"
#include "launcherlink.h"

static const QString chosenGpu = qq::preferHighPerformanceGpu();

Q_IMPORT_QML_PLUGIN(QQPlugin)
Q_IMPORT_QML_PLUGIN(QQ_StudioPlugin)

namespace {

QByteArray frame(const QJsonObject &object)
{
    const QByteArray body = QJsonDocument(object).toJson(QJsonDocument::Compact);
    QByteArray length(4, '\0');
    qToBigEndian(quint32(body.size()), length.data());
    return length + body;
}

/// The launcher's part: listens, records every frame the Studio sends, and answers each with `answer`.
struct FakeLauncher {
    QLocalServer server;
    QLocalSocket *peer = nullptr;
    QByteArray buffer;
    QList<QJsonObject> received;
    std::function<QByteArray(const QJsonObject &)> answer;

    FakeLauncher()
    {
        static int n = 0;
        QLocalServer::removeServer(name());
        server.listen(QStringLiteral("qq-publish-test-%1-%2").arg(QCoreApplication::applicationPid()).arg(++n));
        QObject::connect(&server, &QLocalServer::newConnection, &server, [this] {
            peer = server.nextPendingConnection();
            QObject::connect(peer, &QLocalSocket::readyRead, peer, [this] {
                buffer += peer->readAll();
                while (buffer.size() >= 4) {
                    const quint32 length = qFromBigEndian<quint32>(buffer.constData());
                    if (buffer.size() < 4 + qsizetype(length))
                        return;
                    const QJsonObject object = QJsonDocument::fromJson(buffer.mid(4, length)).object();
                    buffer.remove(0, 4 + length);
                    received << object;
                    if (answer)
                        peer->write(answer(object));
                }
            });
        });
    }

    QString name() const { return server.serverName(); }

    static QByteArray welcome()
    {
        return frame({{QStringLiteral("type"), QStringLiteral("welcome")},
                      {QStringLiteral("protocol"), 1},
                      {QStringLiteral("capabilities"),
                       QJsonArray{QStringLiteral("qontrol.commit"), QStringLiteral("cartridge.publish")}},
                      {QStringLiteral("session"), QString(32, QLatin1Char('0'))}});
    }

    /// Welcome the hello; answer the commit and the publish as given.
    void script(const QJsonObject &commitResult, const QJsonObject &publishAnswer)
    {
        answer = [=](const QJsonObject &got) -> QByteArray {
            const QString type = got.value(QStringLiteral("type")).toString();
            if (type == QLatin1String("hello"))
                return welcome();
            const qint64 id = got.value(QStringLiteral("id")).toInteger();
            const QString method = got.value(QStringLiteral("method")).toString();
            if (method == QLatin1String("qontrol.commit"))
                return frame({{QStringLiteral("type"), QStringLiteral("result")},
                              {QStringLiteral("id"), id},
                              {QStringLiteral("result"), commitResult}});
            QJsonObject reply = publishAnswer;
            reply.insert(QStringLiteral("id"), id);
            return frame(reply);
        };
    }

    QStringList methods() const
    {
        QStringList out;
        for (const QJsonObject &o : received)
            if (o.value(QStringLiteral("type")).toString() == QLatin1String("request"))
                out << o.value(QStringLiteral("method")).toString();
        return out;
    }
};

const QByteArray Token = QByteArray("0123456789abcdef").repeated(4);

QJsonObject minted()
{
    return {{QStringLiteral("type"), QStringLiteral("result")},
            {QStringLiteral("result"),
             QJsonObject{{QStringLiteral("action"), QStringLiteral("minted")},
                         {QStringLiteral("receipt"),
                          QJsonObject{{QStringLiteral("collection"), 7},
                                      {QStringLiteral("item"), 3},
                                      {QStringLiteral("name"), QStringLiteral("a-game")},
                                      {QStringLiteral("commit"),
                                       QJsonObject{{QStringLiteral("kind"), QStringLiteral("SHA-1")},
                                                   {QStringLiteral("id"), QString(40, QLatin1Char('c'))}}}}}}}};
}

QJsonObject declined()
{
    return {{QStringLiteral("type"), QStringLiteral("error")},
            {QStringLiteral("code"), QStringLiteral("declined")},
            {QStringLiteral("message"), QStringLiteral("declined at the confirmation prompt; nothing was signed or changed")}};
}

bool welcomed(qq::LauncherLink &link)
{
    return QTest::qWaitFor([&] { return link.available(); }, 5000);
}

}  // namespace

class TstPublish : public QObject
{
    Q_OBJECT

private slots:
    void theHelloCarriesTheTokenAndTheStudiosId()
    {
        FakeLauncher launcher;
        launcher.script({}, {});
        qq::LauncherLink link;
        QVERIFY(!link.available());
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));
        const QJsonObject hello = launcher.received.first();
        QCOMPARE(hello.value(QStringLiteral("type")).toString(), QStringLiteral("hello"));
        QCOMPARE(hello.value(QStringLiteral("token")).toString(), QString::fromLatin1(Token));
        QCOMPARE(hello.value(QStringLiteral("product")).toString(), QStringLiteral("qq-studio"));
        QCOMPARE(hello.value(QStringLiteral("protocol")).toInt(), 1);
        QCOMPARE(link.capabilities(), (QStringList{QStringLiteral("qontrol.commit"), QStringLiteral("cartridge.publish")}));
        QVERIFY(link.reason().isEmpty());
    }

    void publishingCommitsFirstThenPublishesAndReportsTheReceipt()
    {
        FakeLauncher launcher;
        launcher.script({{QStringLiteral("commit"), QString(40, QLatin1Char('c'))}}, minted());
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));

        QSignalSpy published(&link, &qq::LauncherLink::published);
        QTemporaryDir project;
        link.publish(QUrl::fromLocalFile(project.path()), QStringLiteral("Publish the first level"));
        QVERIFY(link.busy());
        QVERIFY(published.wait(5000));
        QVERIFY(!link.busy());
        QCOMPARE(launcher.methods(), (QStringList{QStringLiteral("qontrol.commit"), QStringLiteral("cartridge.publish")}));

        const QJsonObject commit = launcher.received.at(1).value(QStringLiteral("params")).toObject();
        QCOMPARE(commit.value(QStringLiteral("project")).toString(), QDir::toNativeSeparators(project.path()));
        QCOMPARE(commit.value(QStringLiteral("message")).toString(), QStringLiteral("Publish the first level"));
        QCOMPARE(launcher.received.at(2).value(QStringLiteral("params")).toObject().value(QStringLiteral("project")),
                 commit.value(QStringLiteral("project")));

        const QVariantMap answer = published.first().first().toMap();
        QCOMPARE(answer.value(QStringLiteral("action")).toString(), QStringLiteral("minted"));
        QCOMPARE(answer.value(QStringLiteral("receipt")).toMap().value(QStringLiteral("item")).toInt(), 3);
    }

    void aDeclineIsSaidAndNothingMoreIsAsked()
    {
        FakeLauncher launcher;
        launcher.script({{QStringLiteral("commit"), QJsonValue::Null}}, declined());
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));
        QSignalSpy failed(&link, &qq::LauncherLink::failed);
        QSignalSpy published(&link, &qq::LauncherLink::published);
        QTemporaryDir project;
        link.publish(QUrl::fromLocalFile(project.path()), QStringLiteral("Publish"));
        QVERIFY(failed.wait(5000));
        QCOMPARE(failed.first().at(0).toString(), QStringLiteral("declined"));
        QTest::qWait(200);
        QCOMPARE(published.count(), 0);
        QCOMPARE(launcher.methods().size(), 2);
        QVERIFY(!link.busy());
        QVERIFY(link.available());
    }

    void aRefusedHandshakeLeavesPublishingUnavailableWithTheReason()
    {
        FakeLauncher launcher;
        launcher.answer = [](const QJsonObject &) {
            return frame({{QStringLiteral("type"), QStringLiteral("error")},
                          {QStringLiteral("id"), QJsonValue::Null},
                          {QStringLiteral("code"), QStringLiteral("not_authorised")},
                          {QStringLiteral("message"), QStringLiteral("this connection was not the one the launcher started")}});
        };
        qq::LauncherLink link;
        QSignalSpy lost(&link, &qq::LauncherLink::lost);
        link.connectTo(launcher.name(), Token);
        QVERIFY(QTest::qWaitFor([&] { return link.reason().contains(QStringLiteral("not the one")); }, 5000));
        QVERIFY(!link.available());
        link.publish(QUrl::fromLocalFile(QDir::tempPath()), QStringLiteral("Publish"));
        QVERIFY(!link.busy());
        QCOMPARE(lost.count(), 0);
    }

    void framesInPiecesOrTogetherAreReadAlike()
    {
        FakeLauncher launcher;
        launcher.answer = [](const QJsonObject &) { return QByteArray(); };
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(QTest::qWaitFor([&] { return launcher.peer && !launcher.received.isEmpty(); }, 5000));
        // The welcome, a byte at a time.
        const QByteArray welcome = FakeLauncher::welcome();
        for (char byte : welcome) {
            launcher.peer->write(&byte, 1);
            launcher.peer->flush();
            QCoreApplication::processEvents();
        }
        QVERIFY(welcomed(link));

        // Two frames in one write: the commit's answer and the publish's, as the launcher might send them.
        QSignalSpy published(&link, &qq::LauncherLink::published);
        QTemporaryDir project;
        link.publish(QUrl::fromLocalFile(project.path()), QStringLiteral("Publish"));
        QVERIFY(QTest::qWaitFor([&] { return launcher.methods().size() == 1; }, 5000));
        QJsonObject publishAnswer = minted();
        publishAnswer.insert(QStringLiteral("id"), 2);
        launcher.peer->write(frame({{QStringLiteral("type"), QStringLiteral("result")},
                                    {QStringLiteral("id"), 1},
                                    {QStringLiteral("result"), QJsonObject{{QStringLiteral("commit"), QJsonValue::Null}}}})
                             + frame(publishAnswer));
        QVERIFY(published.wait(5000));
    }

    void aFrameOverTheLimitIsNotRead()
    {
        FakeLauncher launcher;
        launcher.answer = [](const QJsonObject &) {
            QByteArray length(4, '\0');
            qToBigEndian(quint32(2 << 20), length.data());
            return length + QByteArray(64, ' ');
        };
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(QTest::qWaitFor([&] { return link.reason().contains(QStringLiteral("over the limit")); }, 5000));
        QVERIFY(!link.available());
    }

    void whenTheLauncherClosesTheChannelTheStudioIsTold()
    {
        FakeLauncher launcher;
        launcher.script({}, {});
        qq::LauncherLink link;
        QSignalSpy lost(&link, &qq::LauncherLink::lost);
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));
        launcher.peer->disconnectFromServer();
        QVERIFY(lost.wait(5000));
        QVERIFY(!link.available());
    }

    /// On a folder that is not versioned, Publish can still be pressed, says what is missing, and asks the launcher
    /// nothing: a grey button with its reason in a tooltip was missed (10 October 2026).
    void publishingAnUnversionedFolderSaysWhatIsMissingAndAsksNothing()
    {
        QTemporaryDir project;
        QQmlApplicationEngine engine;
        engine.setInitialProperties(
            {{QStringLiteral("model"), QUrl::fromLocalFile(QDir(QStringLiteral(QQ_FIXTURES)).filePath(QStringLiteral("orb.gltf")))},
             {QStringLiteral("project"), QUrl::fromLocalFile(project.path())}});
        engine.loadFromModule("QQ.Studio", "Main");
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *button = window->findChild<QQuickItem *>(QStringLiteral("publishButton"));

        FakeLauncher launcher;
        launcher.script({}, minted());
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));
        window->setProperty("launcher", QVariant::fromValue<QObject *>(&link));
        QVERIFY(QTest::qWaitFor([&] { return button->isEnabled(); }, 5000));

        QMetaObject::invokeMethod(window, "publish");
        QVERIFY(window->property("notice").toString().contains(QStringLiteral("not versioned")));
        QVERIFY(window->property("noticeIsProblem").toBool());
        QTest::qWait(200);
        QVERIFY(launcher.methods().isEmpty());

        window->setProperty("launcher", QVariant());
        QMetaObject::invokeMethod(window, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    void theStudioWindowPublishesThroughTheLauncherAndSaysWhatHappened()
    {
        QTemporaryDir project;
        QDir(project.path()).mkdir(QStringLiteral(".git"));
        QQmlApplicationEngine engine;
        engine.setInitialProperties(
            {{QStringLiteral("model"), QUrl::fromLocalFile(QDir(QStringLiteral(QQ_FIXTURES)).filePath(QStringLiteral("orb.gltf")))},
             {QStringLiteral("project"), QUrl::fromLocalFile(project.path())}});
        engine.loadFromModule("QQ.Studio", "Main");
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *button = window->findChild<QQuickItem *>(QStringLiteral("publishButton"));
        QVERIFY(button);

        // Not started by the launcher: nothing to publish through, and the button says so by being off.
        QVERIFY(!button->isEnabled());
        qq::LauncherLink unstarted;
        unstarted.unavailable(QStringLiteral("Open QQ Studio from the launcher to publish."));
        window->setProperty("launcher", QVariant::fromValue<QObject *>(&unstarted));
        QVERIFY(!button->isEnabled());
        QCOMPARE(window->property("publishHint").toString(), QStringLiteral("Open QQ Studio from the launcher to publish."));

        FakeLauncher launcher;
        launcher.script({{QStringLiteral("commit"), QString(40, QLatin1Char('c'))}}, minted());
        qq::LauncherLink link;
        link.connectTo(launcher.name(), Token);
        QVERIFY(welcomed(link));
        window->setProperty("launcher", QVariant::fromValue<QObject *>(&link));
        QVERIFY(QTest::qWaitFor([&] { return button->isEnabled(); }, 5000));

        // An unsaved scene is saved first, so the commit carries it.
        QVERIFY(window->property("dirty").toBool());
        QMetaObject::invokeMethod(window, "publish");
        QVERIFY(QTest::qWaitFor([&] { return launcher.methods().size() == 2; }, 5000));
        QVERIFY(QFile::exists(QDir(project.path()).filePath(QStringLiteral("scenes/untitled.qml"))));
        QVERIFY(QTest::qWaitFor(
            [&] { return window->property("notice").toString().contains(QStringLiteral("Cartridge 7/3")); }, 5000));
        QVERIFY(!window->property("noticeIsProblem").toBool());

        // A decline is said plainly, and is not a problem.
        launcher.script({{QStringLiteral("commit"), QJsonValue::Null}}, declined());
        QMetaObject::invokeMethod(window, "publish");
        QVERIFY(QTest::qWaitFor(
            [&] { return window->property("notice").toString().contains(QStringLiteral("you declined")); }, 5000));
        QVERIFY(!window->property("noticeIsProblem").toBool());

        // A long refusal can be read whole: hovering the notice shows all of it, the reason after the hash included.
        // The owner's first refused publish ended at "the mint 0xa761…" (15 October 2026).
        const QString reason = QStringLiteral("5DSNDZGfA8SnQw2RfU8uBtD5Vx7J5qsKkZXrWJRbH2rLUV34 holds nothing on Demiurge "
                                              "Development, so it cannot send a mint; nothing was asked or signed. Fund "
                                              "this account on Demiurge Development, then try again. THE-REASON-ENDS-HERE");
        launcher.script({{QStringLiteral("commit"), QJsonValue::Null}},
                        {{QStringLiteral("type"), QStringLiteral("error")},
                         {QStringLiteral("code"), QStringLiteral("unfunded")},
                         {QStringLiteral("message"), QString(reason).repeated(2)}});
        QMetaObject::invokeMethod(window, "publish");
        QVERIFY(QTest::qWaitFor(
            [&] { return window->property("notice").toString().contains(QStringLiteral("THE-REASON-ENDS-HERE")); }, 5000));
        QVERIFY(window->property("noticeIsProblem").toBool());
        auto *label = window->findChild<QQuickItem *>(QStringLiteral("noticeLabel"));
        QVERIFY(label);
        QVERIFY2(label->property("truncated").toBool(), "the test needs a notice too long for the toolbar");
        auto *tip = window->findChild<QObject *>(QStringLiteral("noticeTip"));
        QVERIFY(tip);
        QVERIFY(!tip->property("visible").toBool());
        QTest::mouseMove(window, label->mapToScene(QPointF(label->width() / 2, label->height() / 2)).toPoint());
        QVERIFY(QTest::qWaitFor([&] { return tip->property("visible").toBool(); }, 5000));
        QVERIFY(tip->property("text").toString().endsWith(QStringLiteral("THE-REASON-ENDS-HERE")));

        window->setProperty("launcher", QVariant());
        QMetaObject::invokeMethod(window, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }
};

QTEST_MAIN(TstPublish)
#include "tst_publish.moc"
