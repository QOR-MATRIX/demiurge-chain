// ARQADE's interface (P7.19): the module loads without a warning, draws its portal and cards on the graphics card, shows
// the player and their Sparks as the launcher sends them, and holds still with reduced motion. It wears Demiurge's one
// look (ADR-108): its Theme is the source's values, every text is in the look's faces, scale and colours, and its
// primary reads dark on ember and takes the keyboard's focus ring. It opens a real window, so it needs a desktop session.
// With ARQADE_SHOT set to a folder, it saves what it drew there, for a person to look at.

#include <QDir>
#include <QFile>
#include <QFontInfo>
#include <QFontMetricsF>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJSValue>
#include <QJsonObject>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include <cmath>
#include <functional>

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

// Every item the window shows, in order.
void visibleItems(QQuickItem *item, QList<QQuickItem *> &into)
{
    if (!item || !item->isVisible())
        return;
    into << item;
    for (QQuickItem *child : item->childItems())
        visibleItems(child, into);
}

// The one look's source, read as the generator reads it.
QJsonObject tokens()
{
    QFile file(QStringLiteral(ARQADE_TOKENS));
    if (!file.open(QIODevice::ReadOnly))
        return {};
    return QJsonDocument::fromJson(file.readAll()).object();
}

// WCAG 2.2's contrast ratio of two opaque colours.
double contrast(const QColor &a, const QColor &b)
{
    const auto luminance = [](const QColor &c) {
        const auto lin = [](double v) { return v <= 0.03928 ? v / 12.92 : std::pow((v + 0.055) / 1.055, 2.4); };
        return 0.2126 * lin(c.redF()) + 0.7152 * lin(c.greenF()) + 0.0722 * lin(c.blueF());
    };
    const double x = luminance(a), y = luminance(b);
    return (std::max(x, y) + 0.05) / (std::min(x, y) + 0.05);
}

// Theme's colour names and the source's, for those Theme carries.
const QList<std::pair<const char *, const char *>> kColours = {
    {"void_", "void"},     {"base", "base"},           {"well", "well"},           {"surface", "surface"},
    {"raised", "raised"},  {"edge", "edge"},           {"edgeSoft", "edgeSoft"},   {"accent", "accent"},
    {"accentBright", "accentBright"}, {"accentDim", "accentDim"}, {"inkOnAccent", "onAccent"}, {"ink", "ink"},
    {"inkBody", "inkBody"}, {"inkMuted", "inkMuted"},  {"inkFaint", "inkFaint"},   {"gilt", "gilt"},
    {"giltBright", "giltBright"}, {"giltDim", "giltDim"}, {"ok", "ok"},           {"warn", "warn"},
    {"bad", "bad"},
};

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

    // Theme is the one look's source, value by value: read from tokens.json, not from the file generated from it.
    void the_theme_is_the_one_looks_source_value_by_value()
    {
        const QJsonObject source = tokens();
        QVERIFY2(!source.isEmpty(), ARQADE_TOKENS);
        QObject *theme = engine->singletonInstance<QObject *>("ARQADE", "Theme");
        QVERIFY(theme);

        const QJsonObject colour = source["color"].toObject();
        for (const auto &[ours, theirs] : kColours)
            QVERIFY2(theme->property(ours).value<QColor>() == QColor(colour[theirs].toString()),
                     qPrintable(QStringLiteral("%1 is %2, the source says %3")
                                    .arg(ours, theme->property(ours).value<QColor>().name(), colour[theirs].toString())));

        const QJsonObject font = source["font"].toObject();
        QCOMPARE(theme->property("display").toString(), font["display"].toObject()["family"].toString());
        QCOMPARE(theme->property("ui").toString(), font["interface"].toObject()["family"].toString());
        QCOMPARE(theme->property("body").toString(), font["body"].toObject()["family"].toString());

        const QJsonObject size = source["size"].toObject();
        for (const QString &step : size.keys()) {
            const QByteArray name = "size" + step.left(1).toUpper().toLatin1() + step.mid(1).toLatin1();
            QVERIFY2(theme->property(name).toInt() == size[step].toInt(), name.constData());
        }
        const QJsonObject tracking = source["tracking"].toObject();
        for (const QString &kind : tracking.keys()) {
            const QByteArray name = "tracking" + kind.left(1).toUpper().toLatin1() + kind.mid(1).toLatin1();
            QCOMPARE(theme->property(name).toReal(), tracking[kind].toString().chopped(2).toDouble());
        }
        const QJsonObject radius = source["radius"].toObject();
        QCOMPARE(theme->property("radiusControl").toInt(), radius["control"].toInt());
        QCOMPARE(theme->property("radiusPanel").toInt(), radius["panel"].toInt());

        const QJsonObject motion = source["motion"].toObject();
        for (const char *step : {"quick", "standard", "enter", "sheen"})
            QVERIFY2(theme->property(step).toInt() == motion[step].toInt(), step);
        QStringList curve = motion["ease"].toString().section('(', 1).chopped(1).split(',');
        QVariantList want;
        for (const QString &n : curve)
            want << n.trimmed().toDouble();
        want << 1.0 << 1.0;
        QVariantList got;
        QVariant ease = theme->property("ease");
        if (ease.canConvert<QJSValue>())
            ease = ease.value<QJSValue>().toVariant();
        for (const QVariant &v : ease.toList())
            got << v.toDouble();
        QCOMPARE(got, want);

        const QJsonObject focus = source["focus"].toObject();
        QCOMPARE(theme->property("focusWidth").toInt(), focus["width"].toInt());
        QCOMPARE(theme->property("focusHalo").toInt(), focus["halo"].toInt());
        QCOMPARE(theme->property("focus").value<QColor>(), QColor(colour[focus["color"].toString()].toString()));
    }

    // Every text the window shows is in one of the look's faces, at a step of its scale (the showcase's Cinzel titles may
    // be larger), in one of its colours. The reading face is carried, not found on the system: it is in the executable,
    // and it draws wider than Barlow Semi Condensed, which answers to "Barlow" too.
    void every_text_is_in_the_looks_faces_scale_and_colours()
    {
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QObject *theme = engine->singletonInstance<QObject *>("ARQADE", "Theme");
        QSet<QRgb> colours;
        for (const auto &[ours, _] : kColours)
            colours << theme->property(ours).value<QColor>().rgb();
        const QSet<int> scale = {10, 11, 12, 13, 15, 18, 24, 38};
        const QStringList faces = {theme->property("display").toString(), theme->property("ui").toString(),
                                   theme->property("body").toString()};

        for (const char *file : {"Cinzel.ttf", "BarlowSemiCondensed-Regular.ttf", "Barlow-Regular.ttf", "Barlow-Medium.ttf"})
            QVERIFY2(QFile::exists(QStringLiteral(":/qt/qml/ARQADE/fonts/") + QLatin1String(file)), file);
        QFont reading(theme->property("body").toString(), 15);
        QFont narrow(theme->property("ui").toString(), 15);
        const QString words = QStringLiteral("Play the games of the Demiurge devnet");
        QVERIFY2(QFontMetricsF(reading).horizontalAdvance(words) > QFontMetricsF(narrow).horizontalAdvance(words) * 1.04,
                 "Barlow draws no wider than Barlow Semi Condensed: the reading face is not the one carried");
        // And at its regular weight it is Barlow's Regular, not the Medium standing in for it.
        const QFontInfo drawn(reading);
        QVERIFY2(drawn.styleName() == QLatin1String("Regular"),
                 qPrintable(QStringLiteral("reading text is drawn in %1 %2").arg(drawn.family(), drawn.styleName())));

        QList<QQuickItem *> items;
        visibleItems(window->contentItem(), items);
        int texts = 0, inReading = 0;
        QStringList wrong;
        for (QQuickItem *item : items) {
            if (!item->inherits("QQuickText") || item->property("text").toString().trimmed().isEmpty())
                continue;
            ++texts;
            const QFont font = item->property("font").value<QFont>();
            const QColor colour = item->property("color").value<QColor>();
            const QString what = item->property("text").toString().left(40);
            const bool display = font.family() == faces[0];
            inReading += font.family() == faces[2];
            if (!faces.contains(font.family()))
                wrong << QStringLiteral("\"%1\" in %2").arg(what, font.family());
            if (!scale.contains(font.pixelSize()) && !(display && font.pixelSize() >= 18))
                wrong << QStringLiteral("\"%1\" at %2 pixels").arg(what).arg(font.pixelSize());
            if (!colours.contains(colour.rgb()))
                wrong << QStringLiteral("\"%1\" in %2").arg(what, colour.name());
        }
        QVERIFY2(texts > 20, qPrintable(QString::number(texts)));
        QVERIFY2(inReading > 5, "the reading face is hardly used");
        QVERIFY2(wrong.isEmpty(), qPrintable(wrong.join('\n')));
    }

    // The primary is ember with dark ink on it, 4.5:1 or more, and the keyboard finds it under the shared focus ring.
    void the_primary_reads_dark_on_ember_and_takes_the_focus_ring()
    {
        QObject *theme = engine->singletonInstance<QObject *>("ARQADE", "Theme");
        QList<QQuickItem *> items;
        visibleItems(window->contentItem(), items);
        QQuickItem *primary = nullptr;
        for (QQuickItem *item : items)
            if (item->property("primary").toBool())
                primary = item;
        QVERIFY(primary);
        const auto *label = primary->property("contentItem").value<QQuickItem *>();
        QVERIFY(label);
        const QColor ink = label->property("color").value<QColor>();
        QCOMPARE(ink, theme->property("inkOnAccent").value<QColor>());
        QVERIFY2(contrast(ink, theme->property("accent").value<QColor>()) >= 4.5, "dark ink on ember under 4.5:1");
        QVERIFY2(contrast(ink, theme->property("accentBright").value<QColor>()) >= 4.5, "dark ink on lit ember under 4.5:1");

        QQuickItem *ring = nullptr;
        QList<QQuickItem *> all;
        const std::function<void(QQuickItem *)> every = [&](QQuickItem *i) {
            all << i;
            for (QQuickItem *c : i->childItems())
                every(c);
        };
        every(primary);
        for (QQuickItem *i : all)
            if (i->objectName() == QLatin1String("focusRing"))
                ring = i;
        QVERIFY(ring);
        QVERIFY(!ring->isVisible());
        primary->forceActiveFocus(Qt::TabFocusReason);
        QTRY_VERIFY(ring->isVisible());
        QCOMPARE(ring->property("border").value<QObject *>()->property("color").value<QColor>(),
                 theme->property("focus").value<QColor>());
        QTest::qWait(300);
        save(window, "focus.png");
        primary->setFocus(false);
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
        // Every transition the look times is immediate.
        for (const char *step : {"quick", "standard", "enter", "sheen"})
            QVERIFY2(theme->property(step).toInt() == 0, step);
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
