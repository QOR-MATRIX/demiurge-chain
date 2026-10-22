// The core's bridge as the shell uses it (ADR-104 decision 3): a command crosses and its answer comes back, a refusal
// comes back as the error the interface reads, and the first screen loads and shows what the core says. And the shell
// in Demiurge's one look (ADR-108): Theme is tokens.json's, value by value, and the faces it names are carried and drawn.

#include <QFile>
#include <QFontInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include "core.h"
#include "fonts.h"

Q_IMPORT_QML_PLUGIN(QORPlugin)

using qor::Core;

namespace {

QJsonObject source()
{
    QFile file(QStringLiteral(SHELL_TOKENS));
    if (!file.open(QIODevice::ReadOnly))
        return {};
    return QJsonDocument::fromJson(file.readAll()).object();
}

}  // namespace

class TestCore : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase()
    {
        qor::loadFonts();
    }

    void the_theme_is_the_one_look_value_by_value()
    {
        // Read from tokens.json itself, not from the file generated from it.
        const QJsonObject tokens = source();
        QVERIFY2(!tokens.isEmpty(), SHELL_TOKENS);
        QQmlEngine engine;
        auto *theme = engine.singletonInstance<QObject *>("QOR", "Theme");
        QVERIFY(theme);
        const QJsonObject color = tokens.value("color").toObject();
        const QList<std::pair<const char *, const char *>> colours = {
            {"void_", "void"}, {"base", "base"}, {"well", "well"}, {"surface", "surface"}, {"raised", "raised"},
            {"edge", "edge"}, {"accent", "accent"}, {"accentBright", "accentBright"}, {"ink", "ink"},
            {"inkBody", "inkBody"}, {"inkMuted", "inkMuted"}, {"inkFaint", "inkFaint"}, {"giltBright", "giltBright"},
            {"bad", "bad"}};
        for (const auto &[property, token] : colours)
            QCOMPARE(theme->property(property).value<QColor>(), QColor(color.value(token).toString()));
        const QJsonObject font = tokens.value("font").toObject();
        QCOMPARE(theme->property("display").toString(), font.value("display").toObject().value("family").toString());
        QCOMPARE(theme->property("ui").toString(), font.value("interface").toObject().value("family").toString());
        QCOMPARE(theme->property("text").toString(), font.value("body").toObject().value("family").toString());
        QCOMPARE(theme->property("mono").toString(), font.value("mono").toObject().value("family").toString());
        const QJsonObject radius = tokens.value("radius").toObject();
        QCOMPARE(theme->property("radiusControl").toInt(), radius.value("control").toInt());
        QCOMPARE(theme->property("radiusPanel").toInt(), radius.value("panel").toInt());
    }

    void every_face_is_carried_and_each_weight_drawn_from_its_own_file()
    {
        // Phones carry none of these, and a weight not carried is answered by another, so ask Qt which style it drew:
        // its name says its weight, or none at the regular weight ("Regular", "Semi Condensed").
        const QList<std::tuple<QString, QFont::Weight, QString>> faces = {
            {"Cinzel", QFont::DemiBold, ""},
            {"Barlow", QFont::Normal, ""},
            {"Barlow", QFont::Medium, "Medium"},
            {"Barlow Semi Condensed", QFont::Normal, ""},
            {"Barlow Semi Condensed", QFont::Medium, "Medium"},
            {"Barlow Semi Condensed", QFont::DemiBold, "SemiBold"},
            {"IBM Plex Mono", QFont::Normal, ""},
        };
        for (const auto &[family, weight, word] : faces) {
            QFont font(family);
            font.setWeight(weight);
            const QFontInfo drawn(font);
            const QString seen = drawn.family() + QLatin1Char(' ') + drawn.styleName();
            QCOMPARE(drawn.family(), family);
            if (word.isEmpty() && family != QLatin1String("Cinzel")) {
                for (const char *heavier : {"Medium", "SemiBold", "Bold", "Light", "Thin"})
                    QVERIFY2(!drawn.styleName().contains(QLatin1String(heavier)), qPrintable(seen));
            } else if (!word.isEmpty()) {
                QVERIFY2(drawn.styleName().contains(word), qPrintable(seen));
            }
        }
    }

    void the_core_answers_with_its_version_and_units()
    {
        Core core;
        QVERIFY(!core.version().isEmpty());
        const QVariantMap units = core.call(QStringLiteral("token_info")).value(QStringLiteral("ok")).toMap();
        QCOMPARE(units.value(QStringLiteral("symbol")).toString(), QStringLiteral("CGT"));
        QCOMPARE(units.value(QStringLiteral("sub_units_per_cgt")).toInt(), 1000);
    }

    void an_amount_is_read_exactly_and_crosses_as_a_string()
    {
        Core core;
        const QVariantMap read = core.call(QStringLiteral("parse_amount"), {{QStringLiteral("text"), QStringLiteral("250 Sparks")}})
                                     .value(QStringLiteral("ok"))
                                     .toMap();
        QCOMPARE(read.value(QStringLiteral("embers")).toString(), QStringLiteral("250000000000000000"));
        QCOMPARE(read.value(QStringLiteral("sparks")).toString(), QStringLiteral("250 Sparks"));
    }

    void a_refusal_comes_back_as_kind_and_message()
    {
        Core core;
        const QVariantMap error = core.call(QStringLiteral("parse_amount"), {{QStringLiteral("text"), QStringLiteral("lots")}})
                                      .value(QStringLiteral("error"))
                                      .toMap();
        QCOMPARE(error.value(QStringLiteral("kind")).toString(), QStringLiteral("bad_amount"));
        QVERIFY(!error.value(QStringLiteral("message")).toString().isEmpty());
        QCOMPARE(core.call(QStringLiteral("no_such_command")).value(QStringLiteral("error")).toMap().value(QStringLiteral("kind")).toString(),
                 QStringLiteral("internal"));
    }

    void the_first_screen_shows_what_the_core_says()
    {
        QQmlApplicationEngine engine;
        QStringList warnings;
        connect(&engine, &QQmlEngine::warnings, this, [&warnings](const QList<QQmlError> &list) {
            for (const QQmlError &e : list)
                warnings << e.toString();
        });
        engine.loadFromModule("QOR", "Main");
        QCOMPARE(engine.rootObjects().size(), 1);
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        auto *units = window->findChild<QQuickItem *>(QStringLiteral("units"));
        QVERIFY(units);
        QCOMPARE(units->property("text").toString(), QStringLiteral("1 CGT = 1,000 Sparks = 10^18 embers"));
        auto *amount = window->findChild<QQuickItem *>(QStringLiteral("amount"));
        QVERIFY(amount);
        amount->setProperty("text", QStringLiteral("1.25 CGT"));
        auto *reading = window->findChild<QQuickItem *>(QStringLiteral("reading"));
        QTRY_VERIFY(reading->property("text").toString().startsWith(QStringLiteral("1,250 Sparks")));
        // The wordmark is Cinzel, drawn from the app's own file.
        auto *wordmark = window->findChild<QQuickItem *>(QStringLiteral("wordmark"));
        QVERIFY(wordmark);
        QCOMPARE(QFontInfo(wordmark->property("font").value<QFont>()).family(), QStringLiteral("Cinzel"));
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }
};

QTEST_MAIN(TestCore)
#include "tst_core.moc"
