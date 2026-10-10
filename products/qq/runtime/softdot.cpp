#include "softdot.h"

#include <QByteArray>
#include <QSize>

#include <algorithm>
#include <cmath>

namespace qq {

namespace {
constexpr int side = 64;
}

SoftDot::SoftDot(QQuick3DObject *parent) : QQuick3DTextureData(parent)
{
    make();
}

void SoftDot::setCore(qreal core)
{
    core = std::clamp(core, 0.0, 0.95);
    if (qFuzzyCompare(core, m_core))
        return;
    m_core = core;
    make();
    emit coreChanged();
}

void SoftDot::make()
{
    QByteArray pixels(side * side * 4, Qt::Uninitialized);
    auto *p = reinterpret_cast<uchar *>(pixels.data());
    for (int y = 0; y < side; ++y) {
        for (int x = 0; x < side; ++x) {
            const double dx = (x + 0.5) / side * 2.0 - 1.0;
            const double dy = (y + 0.5) / side * 2.0 - 1.0;
            const double r = std::sqrt(dx * dx + dy * dy);
            // Solid to `core`, then a smooth fall to nothing at the rim.
            const double t = std::clamp((r - m_core) / (1.0 - m_core), 0.0, 1.0);
            const double alpha = (1.0 - t) * (1.0 - t) * (1.0 - t * 0.5);
            const auto a = uchar(std::lround(std::clamp(alpha, 0.0, 1.0) * 255.0));
            *p++ = 255;
            *p++ = 255;
            *p++ = 255;
            *p++ = a;
        }
    }
    setSize(QSize(side, side));
    setFormat(QQuick3DTextureData::RGBA8);
    setHasTransparency(true);
    setTextureData(pixels);
}

}  // namespace qq
