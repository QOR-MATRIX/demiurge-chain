// The core's bridge as the shell uses it (ADR-104 decision 3): a command crosses and its answer comes back, a refusal
// comes back as the error the interface reads, and the first screen loads and shows what the core says.

#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include "core.h"

Q_IMPORT_QML_PLUGIN(QORPlugin)

using qor::Core;

class TestCore : public QObject
{
    Q_OBJECT

private slots:
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
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }
};

QTEST_MAIN(TestCore)
#include "tst_core.moc"
