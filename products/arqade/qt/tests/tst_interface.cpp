// ARQADE's interface (P7.19): the module loads without a warning, draws its portal and cards on the graphics card, shows
// the player and their Sparks as the launcher sends them, and holds still with reduced motion. It opens a real window, so
// it needs a desktop session. With ARQADE_SHOT set to a folder, it saves what it drew there, for a person to look at.

#include <QDir>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include "arqadelink.h"
#include "arqadeserver.h"
#include "fonts.h"

Q_IMPORT_QML_PLUGIN(ARQADEPlugin)

using arqade::ArqadeLink;
using arqade::ArqadeServer;

namespace {

// Every text the window shows, joined, to look for words in.
QString textsOf(QQuickItem *item)
{
    QString all;
    if (!item || !item->isVisible())
        return all;
    const QVariant text = item->property("text");
    if (text.isValid() && text.typeId() == QMetaType::QString)
        all += text.toString() + QLatin1Char('\n');
    for (QQuickItem *child : item->childItems())
        all += textsOf(child);
    return all;
}

void save(QQuickWindow *window, const QString &name)
{
    const QByteArray folder = qgetenv("ARQADE_SHOT");
    if (folder.isEmpty())
        return;
    QDir().mkpath(QString::fromLocal8Bit(folder));
    window->grabWindow().save(QDir(QString::fromLocal8Bit(folder)).filePath(name));
}

}  // namespace

class TestInterface : public QObject
{
    Q_OBJECT

    QQmlApplicationEngine *engine = nullptr;
    ArqadeLink *link = nullptr;
    ArqadeServer *server = nullptr;
    QQuickWindow *window = nullptr;
    QStringList warnings;

private slots:
    void initTestCase()
    {
        arqade::loadFonts();
        link = new ArqadeLink(this);
        // A server nobody answers at: the interface never reaches the network in a test.
        server = new ArqadeServer(link, QUrl("http://127.0.0.1:9/"), this);
        engine = new QQmlApplicationEngine(this);
        connect(engine, &QQmlEngine::warnings, this, [this](const QList<QQmlError> &list) {
            for (const QQmlError &e : list)
                warnings << e.toString();
        });
        engine->setInitialProperties({{"link", QVariant::fromValue(link)},
                                      {"server", QVariant::fromValue(server)},
                                      {"launched", false}});
        engine->loadFromModule("ARQADE", "Main");
        QCOMPARE(engine->rootObjects().size(), 1);
        window = qobject_cast<QQuickWindow *>(engine->rootObjects().first());
        QVERIFY(window);
        window->resize(1440, 900);
    }

    void it_loads_without_a_warning()
    {
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void it_draws_on_the_graphics_card()
    {
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QSignalSpy swapped(window, &QQuickWindow::frameSwapped);
        QTRY_VERIFY_WITH_TIMEOUT(swapped.size() >= 30, 15000);
        const QImage frame = window->grabWindow();
        QVERIFY(!frame.isNull());
        // The core is ember-bright and the void is near black: a frame that drew the scene has both.
        int bright = 0;
        for (int y = 0; y < frame.height(); y += 4)
            for (int x = 0; x < frame.width(); x += 4) {
                const QColor c = frame.pixelColor(x, y);
                if (c.red() > 200 && c.green() > 90 && c.blue() < 120)
                    ++bright;
            }
        QVERIFY2(bright > 50, qPrintable(QStringLiteral("only %1 ember pixels").arg(bright)));
        save(window, "home.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void standalone_it_says_where_to_sign_in()
    {
        const QString shown = textsOf(window->contentItem());
        QVERIFY(shown.contains("Open ARQADE from the QOR Launcher"));
        QCOMPARE(server->state(), ArqadeServer::State::SignedOut);
    }

    void it_shows_the_player_and_their_sparks()
    {
        window->setProperty("player", QVariantMap{{"signed_in", true}, {"name", "yaldabaoth"}, {"level", 7},
                                                  {"sparks", "37,000,000 Sparks"}, {"cgt", "37,000"}});
        QTRY_VERIFY(textsOf(window->contentItem()).contains("37,000,000"));
        const QString shown = textsOf(window->contentItem());
        QVERIFY(shown.contains("yaldabaoth"));
        QVERIFY(shown.contains("Level 7"));
        QVERIFY(shown.contains("Sparks"));
        QVERIFY(shown.contains("37,000 CGT in your wallet"));
        // What rewards are, said where Sparks are shown (ADR-102 decision 7).
        QVERIFY(shown.contains("test CGT, with no value"));
        QTest::qWait(500);
        save(window, "player.png");
    }

    void a_balance_finer_than_a_spark_is_shown_whole()
    {
        window->setProperty("player", QVariantMap{{"signed_in", true}, {"name", "y"}, {"sparks", "0.0004 CGT"}});
        QTRY_VERIFY(textsOf(window->contentItem()).contains("0.0004 CGT"));
    }

    void every_game_and_arena_is_there()
    {
        const QString shown = textsOf(window->contentItem());
        for (const char *name : {"Void Runner", "Synapse", "Orbital", "Rift Survivor", "Flux Four", "Rift Reversi"})
            QVERIFY2(shown.contains(QLatin1String(name)), name);
        QVERIFY(!shown.contains("Energy"));

        // The arenas and their rankings, at the foot of the page.
        auto *page = window->findChild<QQuickItem *>("page");
        QVERIFY(page);
        page->setProperty("contentY", page->property("contentHeight").toReal() - page->height());
        QTest::qWait(600);
        save(window, "arenas.png");
        page->setProperty("contentY", 0);
    }

    void reduced_motion_holds_it_still()
    {
        QObject *theme = engine->singletonInstance<QObject *>("ARQADE", "Theme");
        QVERIFY(theme);
        theme->setProperty("reducedMotion", true);
        QTest::qWait(300);
        const QImage a = window->grabWindow();
        QTest::qWait(600);
        const QImage b = window->grabWindow();
        QCOMPARE(a, b);
        save(window, "still.png");
        theme->setProperty("reducedMotion", false);
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void it_fits_a_narrow_launcher()
    {
        window->resize(760, 560);
        QTest::qWait(500);
        save(window, "narrow.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }
};

QTEST_MAIN(TestInterface)
#include "tst_interface.moc"
