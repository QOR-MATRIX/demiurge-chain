// Does a QQ world draw what it says? (DIRECTION P3.1, ADR-083)
//
// Each case builds a small world from the runtime module "QQ", renders it in a real window on the machine's GPU, grabs
// the frame and judges the pixels: not what the scene declares, what was drawn. The model is the fixture
// tests/fixtures/orb.gltf (make_orb.py writes it): an orange sphere with a thin ring of emissive cyan around its
// equator, loaded at runtime with no import step.
//
// The camera looks straight at the sphere from the front, so the sphere is centred and the ring is a horizontal line
// through the middle of the frame, reaching past the sphere's silhouette on both sides.

#include <QDir>
#include <QFile>
#include <QImage>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickView>
#include <QTemporaryDir>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>
#include <QtMath>

#include <cmath>

Q_IMPORT_QML_PLUGIN(QQPlugin)

namespace {

constexpr int Width = 640;
constexpr int Height = 360;

/// Rec. 709 luma of a pixel, 0 to 255.
double luma(QRgb p)
{
    return 0.2126 * qRed(p) + 0.7152 * qGreen(p) + 0.0722 * qBlue(p);
}

/// The average luma of a rectangle of pixels.
double meanLuma(const QImage &image, const QRect &rect)
{
    double sum = 0;
    int n = 0;
    for (int y = rect.top(); y <= rect.bottom(); ++y) {
        for (int x = rect.left(); x <= rect.right(); ++x) {
            sum += luma(image.pixel(x, y));
            ++n;
        }
    }
    return n ? sum / n : 0;
}

QString fixture(const QString &name)
{
    return QUrl::fromLocalFile(QDir(QStringLiteral(QQ_FIXTURES)).filePath(name)).toString();
}

/// A world with the sun, the ground and one prop, seen from straight in front of the prop.
QByteArray worldSource(const QString &model, bool bloom)
{
    return QStringLiteral(R"(
import QtQuick
import QQ

World {
    width: %1
    height: %2
    bloom: %3
    eye: Qt.vector3d(0, 0, 5.5)
    target: Qt.vector3d(0, 0, 0)
    Sun { elevation: 35; azimuth: -30 }
    Prop { objectName: "prop"; source: "%4" }
}
)")
        .arg(Width)
        .arg(Height)
        .arg(bloom ? QStringLiteral("true") : QStringLiteral("false"), model)
        .toUtf8();
}

/// Show a world, wait until its prop has loaded or failed and a few frames are drawn, and return the frame.
struct Rendered {
    QImage frame;
    int status = -1;
    QString error;
    QStringList warnings;
};

Rendered render(const QByteArray &source)
{
    Rendered out;
    QTemporaryDir dir;
    const QString file = dir.filePath(QStringLiteral("World.qml"));
    QFile f(file);
    if (!f.open(QIODevice::WriteOnly))
        return out;
    f.write(source);
    f.close();

    QQuickView view;
    QObject::connect(view.engine(), &QQmlEngine::warnings, [&out](const QList<QQmlError> &errors) {
        for (const QQmlError &e : errors)
            out.warnings << e.toString();
    });
    view.setResizeMode(QQuickView::SizeRootObjectToView);
    view.resize(Width, Height);
    view.setSource(QUrl::fromLocalFile(file));
    if (view.status() != QQuickView::Ready) {
        for (const QQmlError &e : view.errors())
            out.warnings << e.toString();
        return out;
    }
    view.show();
    if (!QTest::qWaitForWindowExposed(&view))
        return out;

    QObject *prop = view.rootObject()->findChild<QObject *>(QStringLiteral("prop"));
    if (!prop)
        return out;
    // Prop.Status: Empty 0, Ready 1, Failed 2.
    if (!QTest::qWaitFor([prop] { return prop->property("status").toInt() != 0; }, 10000))
        return out;
    out.status = prop->property("status").toInt();
    out.error = prop->property("errorString").toString();

    // Let the renderer settle: the sky probe, shadows and bloom take a few frames. A still scene draws once and then
    // waits, so frames are asked for, and counted as they are drawn.
    int frames = 0;
    QObject::connect(&view, &QQuickWindow::frameSwapped, [&frames] { ++frames; });
    for (int i = 0; i < 200 && frames < 8; ++i) {
        view.update();
        QTest::qWait(25);
    }
    if (frames < 8)
        return out;
    out.frame = view.grabWindow().convertToFormat(QImage::Format_RGB32);
    // QQ_SAVE_FRAMES=<folder> keeps every frame judged, to look at when a case fails.
    if (const QString keep = qEnvironmentVariable("QQ_SAVE_FRAMES"); !keep.isEmpty()) {
        static int n = 0;
        out.frame.save(QDir(keep).filePath(QStringLiteral("frame-%1.png").arg(++n)));
    }
    return out;
}

}  // namespace

class TstWorld : public QObject
{
    Q_OBJECT

private slots:
    void aGltfModelLoadsAtRuntimeAndIsDrawnLitInItsOwnColour();
    void bloomLightsPixelsBeyondTheGlowingBand();
    void aModelThatCannotBeReadIsAnErrorNotACrash();
};

void TstWorld::aGltfModelLoadsAtRuntimeAndIsDrawnLitInItsOwnColour()
{
    const Rendered r = render(worldSource(fixture(QStringLiteral("orb.gltf")), true));
    QVERIFY2(r.warnings.isEmpty(), qPrintable(r.warnings.join(QLatin1Char('\n'))));
    QCOMPARE(r.status, 1);
    QVERIFY(!r.frame.isNull());
    QCOMPARE(r.frame.size(), QSize(Width, Height) * r.frame.devicePixelRatio());

    const int w = r.frame.width();
    const int h = r.frame.height();
    // On the sphere, above the ring: the body's orange, lit.
    const QRgb body = r.frame.pixel(w / 2, h / 2 - h / 9);
    QVERIFY2(qRed(body) > 110 && qRed(body) > qGreen(body) && qGreen(body) > qBlue(body),
             qPrintable(QStringLiteral("body %1,%2,%3").arg(qRed(body)).arg(qGreen(body)).arg(qBlue(body))));

    // A corner is the sky, not the model, and not black: the sky is drawn and lit.
    const QRgb corner = r.frame.pixel(w / 20, h / 20);
    QVERIFY2(luma(corner) > 3 && luma(corner) < luma(body),
             qPrintable(QStringLiteral("corner %1,%2,%3").arg(qRed(corner)).arg(qGreen(corner)).arg(qBlue(corner))));

    // Lit, not flat: the sun is high on the left, so the sphere's upper left is brighter than its lower right.
    const int r6 = h / 10;  // about six tenths of the sphere's radius on screen
    const QRgb lit = r.frame.pixel(w / 2 - r6, h / 2 - r6);
    const QRgb shaded = r.frame.pixel(w / 2 + r6, h / 2 + r6);
    QVERIFY2(luma(lit) > luma(shaded) + 15,
             qPrintable(QStringLiteral("upper left %1, lower right %2").arg(luma(lit)).arg(luma(shaded))));
}

void TstWorld::bloomLightsPixelsBeyondTheGlowingBand()
{
    const Rendered on = render(worldSource(fixture(QStringLiteral("orb.gltf")), true));
    const Rendered off = render(worldSource(fixture(QStringLiteral("orb.gltf")), false));
    QCOMPARE(on.status, 1);
    QCOMPARE(off.status, 1);

    // Just above the ring's two ends, outside the sphere's silhouette: sky without bloom, sky and halo with it.
    const int w = on.frame.width();
    const int h = on.frame.height();
    // Pixels per world unit at the sphere: 5.5 units away, with a 60-degree vertical field of view.
    const double perUnit = h / (2 * 5.5 * std::tan(qDegreesToRadians(30.0)));
    const int reach = int(1.18 * perUnit);  // the ring's radius: its ends lie outside the unit sphere
    const int out = qMax(4, h / 60);
    const QRect leftHalo(w / 2 - reach - out, h / 2 - 3 * out, 2 * out, 2 * out);
    const QRect rightHalo(w / 2 + reach - out, h / 2 - 3 * out, 2 * out, 2 * out);

    const double withBloom = meanLuma(on.frame, leftHalo) + meanLuma(on.frame, rightHalo);
    const double without = meanLuma(off.frame, leftHalo) + meanLuma(off.frame, rightHalo);
    QVERIFY2(withBloom > without + 4,
             qPrintable(QStringLiteral("halo with bloom %1, without %2").arg(withBloom).arg(without)));
}

void TstWorld::aModelThatCannotBeReadIsAnErrorNotACrash()
{
    const Rendered r = render(worldSource(fixture(QStringLiteral("no-such-model.gltf")), true));
    QCOMPARE(r.status, 2);
    QVERIFY(!r.error.isEmpty());
    QVERIFY(!r.frame.isNull());
}

QTEST_MAIN(TstWorld)
#include "tst_world.moc"
