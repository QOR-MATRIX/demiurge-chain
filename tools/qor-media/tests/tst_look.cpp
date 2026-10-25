// QOR Media in Demiurge's one look (ADR-108): Theme is tokens.json's, value by value; the faces it names are carried
// and drawn, each weight from a file of its own; less motion is the launcher's setting; and the keyboard reaches every
// button, shown by the one focus ring.

#include <QFile>
#include <QFontDatabase>
#include <QFontInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest>

#include "activewindow.h"
#include "look.h"

Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

namespace {

QJsonObject source()
{
    QFile file(QStringLiteral(QORMEDIA_TOKENS));
    if (!file.open(QIODevice::ReadOnly))
        return {};
    return QJsonDocument::fromJson(file.readAll()).object();
}

// WCAG's contrast ratio of two colours.
double contrast(const QColor &a, const QColor &b)
{
    const auto lum = [](const QColor &c) {
        const auto lin = [](double v) { return v <= 0.03928 ? v / 12.92 : std::pow((v + 0.055) / 1.055, 2.4); };
        return 0.2126 * lin(c.redF()) + 0.7152 * lin(c.greenF()) + 0.0722 * lin(c.blueF());
    };
    const double x = lum(a), y = lum(b);
    return (std::max(x, y) + 0.05) / (std::min(x, y) + 0.05);
}

}  // namespace

class TestLook : public QObject
{
    Q_OBJECT

    QQmlEngine engine;

    QObject *theme() { return engine.singletonInstance<QObject *>("QOR.Media", "Theme"); }

private slots:
    void initTestCase()
    {
        activewindow::mayTakeTheKeyboard();
        qormedia::loadFonts();
    }

    void the_theme_is_the_one_look_value_by_value()
    {
        // Read from tokens.json itself, not from the file generated from it.
        const QJsonObject tokens = source();
        QVERIFY2(!tokens.isEmpty(), QORMEDIA_TOKENS);
        QObject *t = theme();
        QVERIFY(t);
        const QJsonObject color = tokens.value("color").toObject();
        const QList<std::pair<const char *, const char *>> colours = {
            {"void_", "void"}, {"base", "base"}, {"surface", "surface"}, {"raised", "raised"}, {"well", "well"},
            {"edge", "edge"}, {"accent", "accent"}, {"accentBright", "accentBright"}, {"accentDim", "accentDim"},
            {"ink", "ink"}, {"inkBody", "inkBody"}, {"inkMuted", "inkMuted"}, {"inkFaint", "inkFaint"},
            {"inkOnAccent", "onAccent"}};
        for (const auto &[property, token] : colours)
            QCOMPARE(t->property(property).value<QColor>(), QColor(color.value(token).toString()));
        const QJsonObject font = tokens.value("font").toObject();
        QCOMPARE(t->property("ui").toString(), font.value("interface").toObject().value("family").toString());
        QCOMPARE(t->property("text").toString(), font.value("body").toObject().value("family").toString());
        QCOMPARE(t->property("mono").toString(), font.value("mono").toObject().value("family").toString());
        const QJsonObject size = tokens.value("size").toObject();
        for (const char *step : {"caption", "ui", "body", "title", "heading", "display", "figure"}) {
            const QByteArray name = "size" + QByteArray(step).left(1).toUpper() + QByteArray(step).mid(1);
            QCOMPARE(t->property(name).toInt(), size.value(step).toInt());
        }
        const QJsonObject radius = tokens.value("radius").toObject();
        QCOMPARE(t->property("radiusControl").toInt(), radius.value("control").toInt());
        QCOMPARE(t->property("radiusPanel").toInt(), radius.value("panel").toInt());
        const QJsonObject motion = tokens.value("motion").toObject();
        t->setProperty("reducedMotion", false);
        for (const char *name : {"quick", "standard", "enter"})
            QCOMPARE(t->property(name).toInt(), motion.value(name).toInt());
        const QJsonObject focus = tokens.value("focus").toObject();
        QCOMPARE(t->property("focusWidth").toInt(), focus.value("width").toInt());
        QCOMPARE(t->property("focusHalo").toInt(), focus.value("halo").toInt());
        QCOMPARE(t->property("focus").value<QColor>(), QColor(color.value(focus.value("color").toString()).toString()));
        // The ink on an ember button reads at 4.5:1 or more; white, which Elsewhere's button wore, read 2.9:1.
        QVERIFY(contrast(t->property("inkOnAccent").value<QColor>(), t->property("accent").value<QColor>()) >= 4.5);
        QVERIFY(contrast(t->property("inkFaint").value<QColor>(), t->property("raised").value<QColor>()) >= 4.5);
    }

    void every_face_is_carried_and_each_weight_drawn_from_its_own_file()
    {
        // A weight not carried is answered by another (Barlow's Medium for its Regular, drawn wide), so ask Qt which
        // style it drew, not only which family: its name says its weight ("Semi Condensed Medium"), or none at the
        // regular weight ("Regular", "Semi Condensed").
        const QList<std::tuple<QString, QFont::Weight, QString>> faces = {
            {"Barlow", QFont::Normal, ""},
            {"Barlow", QFont::Medium, "Medium"},
            {"Barlow Semi Condensed", QFont::Normal, ""},
            {"Barlow Semi Condensed", QFont::Medium, "Medium"},
            {"Barlow Semi Condensed", QFont::DemiBold, "SemiBold"},
            {"IBM Plex Mono", QFont::Normal, ""},
            {"IBM Plex Mono", QFont::Medium, "Medium"},
        };
        for (const auto &[family, weight, word] : faces) {
            QFont font(family);
            font.setWeight(weight);
            const QFontInfo drawn(font);
            const QString seen = drawn.family() + QLatin1Char(' ') + drawn.styleName();
            QCOMPARE(drawn.family(), family);
            if (word.isEmpty()) {
                for (const char *heavier : {"Medium", "SemiBold", "Bold", "Light", "Thin"})
                    QVERIFY2(!drawn.styleName().contains(QLatin1String(heavier)), qPrintable(seen));
            } else {
                QVERIFY2(drawn.styleName().contains(word), qPrintable(seen));
            }
        }
    }

    void less_motion_is_the_launchers_setting_and_stills_every_transition()
    {
        qputenv("QOR_REDUCED_MOTION", "1");
        QVERIFY(qormedia::prefersReducedMotion());
        qputenv("QOR_REDUCED_MOTION", "0");
        QVERIFY(!qormedia::prefersReducedMotion());
        qunsetenv("QOR_REDUCED_MOTION");

        QObject *t = theme();
        t->setProperty("reducedMotion", true);
        for (const char *name : {"quick", "standard", "enter"})
            QCOMPARE(t->property(name).toInt(), 0);
        t->setProperty("reducedMotion", false);
        QVERIFY(t->property("quick").toInt() > 0);
    }

    void tab_reaches_every_button_shows_the_ring_and_return_presses()
    {
        QQmlComponent component(&engine);
        component.setData(R"(
            import QtQuick
            import QOR.Media
            Window {
                width: 420; height: 120; visible: true
                Row {
                    x: 20; y: 30; spacing: 16
                    IconButton { objectName: "play"; iconName: "play" }
                    IconButton { objectName: "next"; iconName: "next" }
                    TextButton { objectName: "save"; text: "Save"; primary: true }
                }
            }
        )", QUrl(QStringLiteral("qrc:/look.qml")));
        std::unique_ptr<QObject> root(component.create());
        QVERIFY2(root, qPrintable(component.errorString()));
        auto *window = qobject_cast<QQuickWindow *>(root.get());
        QVERIFY(window);
        QVERIFY(activewindow::activate(window));

        const auto ring = [](QQuickItem *control) {
            auto *r = control->findChild<QQuickItem *>(QStringLiteral("focusRing"));
            return r && r->isVisible();
        };
        for (const char *name : {"play", "next", "save"}) {
            QTest::keyClick(window, Qt::Key_Tab);
            QQuickItem *focused = window->activeFocusItem();
            QVERIFY(focused);
            QCOMPARE(focused->objectName(), QString::fromLatin1(name));
            QVERIFY(focused->property("visualFocus").toBool());
            QVERIFY2(ring(focused), name);
        }
        auto *save = window->activeFocusItem();
        QSignalSpy pressed(save, SIGNAL(clicked()));
        QTest::keyClick(window, Qt::Key_Return);
        QCOMPARE(pressed.size(), 1);

        // A click on an icon button presses it and leaves the keyboard where it was, so a view's own keys still work.
        auto *play = window->findChild<QQuickItem *>(QStringLiteral("play"));
        QVERIFY(play);
        QSignalSpy clicked(play, SIGNAL(clicked()));
        QTest::mouseClick(window, Qt::LeftButton, {}, play->mapToScene(QPointF(play->width() / 2, play->height() / 2)).toPoint());
        QCOMPARE(clicked.size(), 1);
        QCOMPARE(window->activeFocusItem(), save);
        QVERIFY(!ring(play));
    }
};

QTEST_MAIN(TestLook)
#include "tst_look.moc"
