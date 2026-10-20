// Does a QQ world draw what it says? (DIRECTION P3.1, ADR-083)
//
// Each case builds a small world from the runtime module "QQ", renders it in a real window on the machine's GPU, grabs
// the frame and judges the pixels: not what the scene declares, what was drawn. The model is the fixture
// tests/fixtures/orb.gltf (make_orb.py writes it): an orange sphere with a thin ring of emissive cyan around its
// equator, loaded at runtime with no import step.
//
// The camera looks straight at the sphere from the front, so the sphere is centred and the ring is a horizontal line
// through the middle of the frame, reaching past the sphere's silhouette on both sides.
//
// Materials (P3.14) are judged the same way: a picture on a shape counted copy by copy, its edges sharp or soft; a
// pictured shape lit as a plain one is; a normal map turning a face toward the light; a spot light's pool on the ground,
// moving as the light turns; an HDR sky seen, lighting what is under it, and turned; and the ground drawn as wide as
// what stands on it. The fixtures are written by make_looks.py.

#include <QDir>
#include <QFile>
#include <QImage>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickView>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>

#include "gpu.h"
#include <QtMath>

#include <cmath>

// Drawn with the GPU the Studio and the Player draw with: chosen before QTEST_MAIN makes the application.
static const QString chosenGpu = qq::preferHighPerformanceGpu();

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

    // A world with a prop waits for it to load or fail; one without (the materials' cases) draws at once.
    if (QObject *prop = view.rootObject()->findChild<QObject *>(QStringLiteral("prop"))) {
        // Prop.Status: Empty 0, Ready 1, Failed 2.
        if (!QTest::qWaitFor([prop] { return prop->property("status").toInt() != 0; }, 10000))
            return out;
        out.status = prop->property("status").toInt();
        out.error = prop->property("errorString").toString();
    }

    // Let the renderer settle: the sky probe, shadows and bloom take a few frames. A still scene draws once and then
    // waits, so frames are asked for, and counted as they are drawn.
    // Counted on this thread through `counting`, so no count reaches `frames` from the render thread, or after it.
    int frames = 0;
    {
        QObject counting;
        QObject::connect(&view, &QQuickWindow::frameSwapped, &counting, [&frames] { ++frames; });
        for (int i = 0; i < 200 && frames < 8; ++i) {
            view.update();
            QTest::qWait(25);
        }
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

/// A world `Width` by `Height` with `body` in it, seen from `eye` looking at `target`.
QByteArray world(const QString &settings, const QString &body)
{
    return QStringLiteral("import QtQuick\nimport QQ\nWorld {\n    width: %1\n    height: %2\n%3\n%4\n}\n")
        .arg(Width)
        .arg(Height)
        .arg(settings, body)
        .toUtf8();
}

/// Where a point in the world is drawn in `frame`, seen from `eye` looking at `target` through the World's camera (a
/// 60-degree vertical field of view, the world's up kept up).
QPoint onScreen(const QImage &frame, QVector3D eye, QVector3D target, QVector3D point)
{
    const QVector3D forward = (target - eye).normalized();
    const QVector3D right = QVector3D::crossProduct(forward, QVector3D(0, 1, 0)).normalized();
    const QVector3D up = QVector3D::crossProduct(right, forward);
    const QVector3D v = point - eye;
    const double depth = QVector3D::dotProduct(v, forward);
    const double half = std::tan(qDegreesToRadians(30.0));
    const double aspect = double(frame.width()) / frame.height();
    const double x = QVector3D::dotProduct(v, right) / depth / (half * aspect);
    const double y = QVector3D::dotProduct(v, up) / depth / half;
    return QPoint(int((x + 1) / 2 * frame.width()), int((1 - y) / 2 * frame.height()));
}

QString rgb(QRgb p)
{
    return QStringLiteral("%1,%2,%3").arg(qRed(p)).arg(qGreen(p)).arg(qBlue(p));
}

/// A checker's colours as drawn: 'r' red, 'b' blue, ' ' neither (a blend, or something else).
char checkerColour(QRgb p)
{
    if (qRed(p) > qBlue(p) + 40)
        return 'r';
    if (qBlue(p) > qRed(p) + 40)
        return 'b';
    return ' ';
}

/// Along a row of `frame` from x0 to x1: how many times the checker changes colour, and how many pixels are neither.
struct Run {
    int changes = 0;
    int between = 0;
    QString seen;
};
Run along(const QImage &frame, int y, int x0, int x1)
{
    Run run;
    char last = 0;
    for (int x = x0; x <= x1; ++x) {
        const char c = checkerColour(frame.pixel(x, y));
        if (c == ' ') {
            ++run.between;
            continue;
        }
        if (last && c != last)
            ++run.changes;
        if (c != last)
            run.seen += QLatin1Char(c);
        last = c;
    }
    return run;
}

}  // namespace

class TstWorld : public QObject
{
    Q_OBJECT

private slots:
    void aGltfModelLoadsAtRuntimeAndIsDrawnLitInItsOwnColour();
    void bloomLightsPixelsBeyondTheGlowingBand();
    void aModelThatCannotBeReadIsAnErrorNotACrash();
    // Materials (P3.14).
    void aPictureLiesOnAShapeOneCopyEveryImageSizeUnits();
    void aPicturedShapeIsLitAsAPlainOneIs();
    void aNormalMapTurnsAFaceTowardTheLight();
    void aSpotlightLightsItsConeAndTurnsWithIt();
    void anHdrSkyIsSeenLightsTheWorldAndTurns();
    void theGroundIsDrawnAsWideAsItsSlabWithItsPicture();
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

void TstWorld::aPictureLiesOnAShapeOneCopyEveryImageSizeUnits()
{
    // A wall 4 units wide and 2 tall, square to the eye, with the 2 x 2 checker on it: one copy a unit, so a square every
    // half unit, as many across as its width says and as tall as they are wide (not stretched to the face).
    const QVector3D eye(0, 0, 5.5);
    const QString settings = QStringLiteral("    eye: Qt.vector3d(0, 0, 5.5)\n    target: Qt.vector3d(0, 0, 0)\n"
                                            "    Sun { elevation: 30; azimuth: 0; brightness: 1.5 }");
    const QString wall = QStringLiteral(
        "    Shape { position: Qt.vector3d(0, 0, -0.1); scale: Qt.vector3d(4, 2, 0.2); colour: \"#ffffff\"; roughness: 1;"
        " image: \"%1\"; imageSize: %2; crisp: %3 }");
    const Rendered crisp = render(world(settings, wall.arg(fixture(QStringLiteral("checker.png")), QStringLiteral("1"),
                                                           QStringLiteral("true"))));
    QVERIFY2(crisp.warnings.isEmpty(), qPrintable(crisp.warnings.join(QLatin1Char('\n'))));
    QVERIFY(!crisp.frame.isNull());
    const QImage &f = crisp.frame;
    // A quarter unit above the middle, inside a row of squares, from 1.9 units left to 1.9 right: eight squares.
    const int y = onScreen(f, eye, {}, QVector3D(0, 0.25f, 0)).y();
    const int x0 = onScreen(f, eye, {}, QVector3D(-1.9f, 0.25f, 0)).x();
    const int x1 = onScreen(f, eye, {}, QVector3D(1.9f, 0.25f, 0)).x();
    const Run across = along(f, y, x0, x1);
    QVERIFY2(across.changes == 7, qPrintable(QStringLiteral("%1 changes: %2").arg(across.changes).arg(across.seen)));
    // Sharp: hardly a pixel between red and blue.
    QVERIFY2(across.between <= 6, qPrintable(QStringLiteral("%1 pixels between").arg(across.between)));
    // Up the middle of a column, from 0.9 below to 0.9 above: four squares, so squares, not stretched.
    int changesUp = 0;
    char last = 0;
    const int column = onScreen(f, eye, {}, QVector3D(0.25f, 0, 0)).x();
    const int top = onScreen(f, eye, {}, QVector3D(0.25f, 0.9f, 0)).y();
    const int bottom = onScreen(f, eye, {}, QVector3D(0.25f, -0.9f, 0)).y();
    for (int yy = top; yy <= bottom; ++yy) {
        const char c = checkerColour(f.pixel(column, yy));
        if (c != ' ' && last && c != last)
            ++changesUp;
        if (c != ' ')
            last = c;
    }
    QCOMPARE(changesUp, 3);

    // Twice the size: half as many copies.
    const Rendered larger = render(world(settings, wall.arg(fixture(QStringLiteral("checker.png")), QStringLiteral("2"),
                                                            QStringLiteral("true"))));
    QCOMPARE(along(larger.frame, y, x0, x1).changes, 3);

    // Not crisp: the picture is blended between its pixels, so many times as much of the row is neither red nor blue.
    const Rendered soft = render(world(settings, wall.arg(fixture(QStringLiteral("checker.png")), QStringLiteral("1"),
                                                          QStringLiteral("false"))));
    const Run blended = along(soft.frame, y, x0, x1);
    QVERIFY2(blended.between > (x1 - x0) / 8 && blended.between > 5 * across.between,
             qPrintable(QStringLiteral("%1 of %2 pixels between, %3 when crisp")
                            .arg(blended.between).arg(x1 - x0).arg(across.between)));
}

void TstWorld::aPicturedShapeIsLitAsAPlainOneIs()
{
    // A white box with a mid-grey picture draws as a grey box of that colour does, under the same sun and sky: a picture
    // is lit by Qt's own lighting as a plain shape is, and its colours are converted to linear light the same way.
    const QString settings = QStringLiteral("    eye: Qt.vector3d(2.5, 2, 4.5)\n    target: Qt.vector3d(0, 0, 0)\n"
                                            "    Sun { elevation: 40; azimuth: 30 }");
    const QString box = QStringLiteral("    Shape { scale: Qt.vector3d(2, 2, 2); roughness: 0.6; %1 }");
    const Rendered plain = render(world(settings, box.arg(QStringLiteral("colour: \"#808080\""))));
    const Rendered pictured = render(world(settings, box.arg(QStringLiteral("colour: \"#ffffff\"; image: \"%1\"")
                                                                 .arg(fixture(QStringLiteral("grey.png"))))));
    QVERIFY2(pictured.warnings.isEmpty(), qPrintable(pictured.warnings.join(QLatin1Char('\n'))));
    const QVector3D eye(2.5f, 2, 4.5f);
    // A point on each of the three faces seen, all three in the sun: the front, the right side and the top.
    for (const QVector3D &point : {QVector3D(0.2f, -0.2f, 1), QVector3D(1, -0.2f, 0.2f), QVector3D(0.2f, 1, 0.2f)}) {
        const QPoint at = onScreen(plain.frame, eye, {}, point);
        const QRgb a = plain.frame.pixel(at);
        const QRgb b = pictured.frame.pixel(at);
        const int most = qMax(qAbs(qRed(a) - qRed(b)), qMax(qAbs(qGreen(a) - qGreen(b)), qAbs(qBlue(a) - qBlue(b))));
        QVERIFY2(most <= 6, qPrintable(QStringLiteral("plain %1, pictured %2").arg(rgb(a), rgb(b))));
        QVERIFY2(luma(a) > 20, qPrintable(rgb(a)));
    }
}

void TstWorld::aNormalMapTurnsAFaceTowardTheLight()
{
    // A face square to the eye, lit from its right (the sun's azimuth 60, from +x) and then from its left (-60). Flat, it
    // is lit alike from either side. With a relief whose every normal leans toward +x, it turns to the light on the right
    // and away from the light on the left.
    const QVector3D eye(0, 0, 5);
    const QString settings = QStringLiteral("    eye: Qt.vector3d(0, 0, 5)\n    target: Qt.vector3d(0, 0, 0)\n"
                                            "    skyLight: 0.1\n    bloom: false\n"
                                            "    Sun { elevation: 10; azimuth: %1; brightness: 0.5 }");
    const QString face = QStringLiteral("    Shape { position: Qt.vector3d(0, 0, -0.1); scale: Qt.vector3d(3, 2, 0.2);"
                                        " colour: \"#808080\"; roughness: 1%1 }");
    const QString relief = QStringLiteral("; normalMap: \"%1\"").arg(fixture(QStringLiteral("bumps.png")));
    auto lit = [&](int azimuth, const QString &extra) {
        const Rendered r = render(world(settings.arg(azimuth), face.arg(extra)));
        const QPoint at = onScreen(r.frame, eye, {}, QVector3D(0, 0, 0));
        return meanLuma(r.frame, QRect(at - QPoint(8, 8), QSize(16, 16)));
    };
    const double flatRight = lit(60, QString());
    const double flatLeft = lit(-60, QString());
    QVERIFY2(flatRight > 20 && flatRight < 230, qPrintable(QStringLiteral("flat: %1").arg(flatRight)));
    QVERIFY2(qAbs(flatRight - flatLeft) < 5, qPrintable(QStringLiteral("flat: %1 and %2").arg(flatRight).arg(flatLeft)));
    const double bumpedRight = lit(60, relief);
    const double bumpedLeft = lit(-60, relief);
    QVERIFY2(bumpedRight > bumpedLeft + 15,
             qPrintable(QStringLiteral("relief: from the right %1, from the left %2").arg(bumpedRight).arg(bumpedLeft)));
}

void TstWorld::aSpotlightLightsItsConeAndTurnsWithIt()
{
    // Four units above a grey ground in a dark world, shining straight down in a 30-degree cone: a pool of light under
    // it, darkness two units away. Tipped 26.6 degrees forward, the pool moves two units toward -z.
    const QVector3D eye(0, 8, 8);
    const QString settings = QStringLiteral("    eye: Qt.vector3d(0, 8, 8)\n    target: Qt.vector3d(0, 0, 0)\n"
                                            "    skyLight: 0.02\n    bloom: false");
    const QString scene = QStringLiteral(
        "    Ground { extent: 20; colour: \"#9a9a9a\" }\n"
        "    Spotlight { position: Qt.vector3d(0, 4, 0); eulerRotation: Qt.vector3d(%1, 0, 0); cone: 30; softness: 0.1;"
        " brightness: 30; reach: 20 }");
    const Rendered down = render(world(settings, scene.arg(-90)));
    QVERIFY2(down.warnings.isEmpty(), qPrintable(down.warnings.join(QLatin1Char('\n'))));
    auto brightness = [&eye](const Rendered &r, const QVector3D &point) {
        const QPoint at = onScreen(r.frame, eye, {}, point);
        return meanLuma(r.frame, QRect(at - QPoint(4, 4), QSize(8, 8)));
    };
    const double under = brightness(down, QVector3D(0, 0, 0));
    const double beside = brightness(down, QVector3D(2, 0, 0));
    QVERIFY2(under > 60 && beside < under / 4, qPrintable(QStringLiteral("under %1, beside %2").arg(under).arg(beside)));

    // atan(2 / 4) is 26.57 degrees: the cone's middle now meets the ground at z = -2.
    const Rendered tipped = render(world(settings, scene.arg(-90 + 26.565)));
    const double moved = brightness(tipped, QVector3D(0, 0, -2));
    const double left = brightness(tipped, QVector3D(0, 0, 0));
    QVERIFY2(moved > 50 && left < moved / 3, qPrintable(QStringLiteral("at -2: %1, under: %2").arg(moved).arg(left)));
}

void TstWorld::anHdrSkyIsSeenLightsTheWorldAndTurns()
{
    // The sky is half red, half blue, brighter than white. Looking along -z: the sky ahead is one half; a white ball's
    // face toward the eye looks back at the other half, so it is lit by that one's colour. Turned half round, both swap.
    const QString settings = QStringLiteral("    eye: Qt.vector3d(0, 0, 0)\n    target: Qt.vector3d(0, 0, -1)\n"
                                            "    bloom: false\n    skyLight: 1\n    sky: \"%1\"\n    skyTurn: %2");
    const QString ball = QStringLiteral("    Shape { form: \"sphere\"; position: Qt.vector3d(0, 0, -4);"
                                        " scale: Qt.vector3d(1.5, 1.5, 1.5); colour: \"#ffffff\"; roughness: 1 }");
    auto look = [&](int turn) {
        const Rendered r = render(world(settings.arg(fixture(QStringLiteral("sky.hdr"))).arg(turn), ball));
        return r;
    };
    const Rendered ahead = look(0);
    QVERIFY2(ahead.warnings.isEmpty(), qPrintable(ahead.warnings.join(QLatin1Char('\n'))));
    const int w = ahead.frame.width();
    const int h = ahead.frame.height();
    const QRgb skyAhead = ahead.frame.pixel(w / 2, h / 8);
    const QRgb ballFace = ahead.frame.pixel(w / 2, h / 2);
    const char seen = checkerColour(skyAhead);
    QVERIFY2(seen != ' ', qPrintable(QStringLiteral("the sky ahead is %1").arg(rgb(skyAhead))));
    QVERIFY2(checkerColour(ballFace) != ' ' && checkerColour(ballFace) != seen,
             qPrintable(QStringLiteral("sky %1, ball %2").arg(rgb(skyAhead), rgb(ballFace))));

    const Rendered turned = look(180);
    const QRgb skyNow = turned.frame.pixel(w / 2, h / 8);
    const QRgb ballNow = turned.frame.pixel(w / 2, h / 2);
    QVERIFY2(checkerColour(skyNow) != ' ' && checkerColour(skyNow) != seen,
             qPrintable(QStringLiteral("turned, the sky ahead is %1").arg(rgb(skyNow))));
    QVERIFY2(checkerColour(ballNow) == seen, qPrintable(QStringLiteral("turned, the ball is %1").arg(rgb(ballNow))));

    // A sky dimmer than white is left as it is by bloom, which glows only from what is brighter: bleeding the whole
    // frame back over a sky image would wash it out.
    const QString dim = QStringLiteral("    eye: Qt.vector3d(0, 0, 0)\n    target: Qt.vector3d(0, 0, -1)\n"
                                       "    bloom: %1\n    skyLight: 1\n    sky: \"%2\"");
    const QRgb without = render(world(dim.arg(QStringLiteral("false"), fixture(QStringLiteral("dim.hdr"))), QString()))
                             .frame.pixel(w / 2, h / 2);
    const QRgb with = render(world(dim.arg(QStringLiteral("true"), fixture(QStringLiteral("dim.hdr"))), QString()))
                          .frame.pixel(w / 2, h / 2);
    QVERIFY2(qAbs(luma(with) - luma(without)) < 8, qPrintable(QStringLiteral("bloom %1, none %2").arg(rgb(with), rgb(without))));
}

void TstWorld::theGroundIsDrawnAsWideAsItsSlabWithItsPicture()
{
    // A ground 4 units across, the checker on it a copy a unit: both colours on it, and beyond its edge at 2 units, the
    // sky below the horizon, not more ground.
    const QVector3D eye(0, 9, 6);
    const QString settings = QStringLiteral("    eye: Qt.vector3d(0, 9, 6)\n    target: Qt.vector3d(0, 0, 0)\n"
                                            "    groundHorizon: \"#000000\"\n    groundBottom: \"#000000\"\n"
                                            "    Sun { elevation: 60; azimuth: 0; brightness: 1.5 }");
    const Rendered r = render(world(settings, QStringLiteral("    Ground { extent: 4; colour: \"#ffffff\"; image: \"%1\" }")
                                                  .arg(fixture(QStringLiteral("checker.png")))));
    QVERIFY2(r.warnings.isEmpty(), qPrintable(r.warnings.join(QLatin1Char('\n'))));
    const int y = onScreen(r.frame, eye, {}, QVector3D(0, 0, 0.25f)).y();
    const Run across = along(r.frame, y, onScreen(r.frame, eye, {}, QVector3D(-1.9f, 0, 0.25f)).x(),
                             onScreen(r.frame, eye, {}, QVector3D(1.9f, 0, 0.25f)).x());
    QVERIFY2(across.changes >= 6, qPrintable(QStringLiteral("%1 changes: %2").arg(across.changes).arg(across.seen)));
    const QPoint beyond = onScreen(r.frame, eye, {}, QVector3D(3, 0, 0.25f));
    QVERIFY2(luma(r.frame.pixel(beyond)) < 25, qPrintable(QStringLiteral("beyond the edge %1").arg(rgb(r.frame.pixel(beyond)))));
}

QTEST_MAIN(TstWorld)
#include "tst_world.moc"
