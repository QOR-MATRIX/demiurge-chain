// SoftDot: a round, soft-edged spot of light, made in memory: the sprite QQ's particles are drawn with. Made rather than
// shipped as an image, so a scene needs no texture file to have sparks, embers or mist.

#pragma once

#include <QtQml/qqmlregistration.h>
#include <QtQuick3D/qquick3dtexturedata.h>

namespace qq {

class SoftDot : public QQuick3DTextureData
{
    Q_OBJECT
    QML_ELEMENT

    /// How much of the spot is solid before it fades: 0 a pure glow, towards 1 a disc with a soft rim.
    Q_PROPERTY(qreal core READ core WRITE setCore NOTIFY coreChanged)

public:
    explicit SoftDot(QQuick3DObject *parent = nullptr);

    qreal core() const { return m_core; }
    void setCore(qreal core);

signals:
    void coreChanged();

private:
    void make();

    qreal m_core = 0.15;
};

}  // namespace qq
