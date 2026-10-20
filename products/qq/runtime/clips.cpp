#include "clips.h"

#include <QVariant>

namespace qq {

Clips::Clips(QObject *parent) : QObject(parent) {}

void Clips::setModel(QObject *model)
{
    if (model == m_model)
        return;
    m_model = model;
    scan();
}

QStringList Clips::names() const
{
    QStringList out;
    for (const Clip &clip : m_clips)
        out << clip.name;
    return out;
}

void Clips::scan()
{
    m_clips.clear();
    if (m_model) {
        // The loader parents each Timeline to the scene it made, and its TimelineAnimation to the Timeline (Qt's
        // qssgqmlutilities.cpp). Found by class name, so QQ needs none of Qt Quick Timeline's private headers.
        const QList<QObject *> all = m_model->findChildren<QObject *>();
        for (QObject *timeline : all) {
            if (!timeline->inherits("QQuickTimeline"))
                continue;
            Clip clip;
            clip.timeline = timeline;
            clip.end = timeline->property("endFrame").toDouble();
            for (QObject *child : timeline->children()) {
                if (child->inherits("QQuickKeyframeGroup")) {
                    // What it moves, and that thing's value at rest, read before any clip is posed.
                    QObject *target = child->property("target").value<QObject *>();
                    const QByteArray property = child->property("property").toString().toUtf8();
                    if (target && !property.isEmpty())
                        clip.rest.append({target, property, target->property(property.constData())});
                    continue;
                }
                if (!child->inherits("QQuickTimelineAnimation"))
                    continue;
                clip.name = child->objectName();
                child->setProperty("running", false);  // the game's clock plays it, not the wall's
            }
            if (clip.name.isEmpty())
                clip.name = QStringLiteral("clip %1").arg(m_clips.size() + 1);
            timeline->setProperty("enabled", false);
            m_clips.append(clip);
        }
    }
    emit changed();
}

double Clips::lengthOf(const QString &name) const
{
    for (const Clip &clip : m_clips) {
        if (clip.name == name)
            return clip.end / 1000.0;
    }
    return 0;
}

void Clips::stop(Clip &clip)
{
    if (!clip.on)
        return;
    clip.on = false;
    if (clip.timeline)
        clip.timeline->setProperty("enabled", false);
    for (const Rest &rest : std::as_const(clip.rest)) {
        if (rest.target)
            rest.target->setProperty(rest.property.constData(), rest.value);
    }
}

bool Clips::pose(const QString &name, double seconds)
{
    bool found = false;
    // Every other one off and put back first, so two never pose the same joint at once.
    for (Clip &clip : m_clips) {
        if (clip.name != name)
            stop(clip);
    }
    for (Clip &clip : m_clips) {
        if (!clip.timeline || clip.name != name)
            continue;
        clip.on = true;
        clip.timeline->setProperty("enabled", true);
        clip.timeline->setProperty("currentFrame", qBound(0.0, seconds * 1000.0, clip.end));
        found = true;
    }
    return found;
}

void Clips::rest()
{
    for (Clip &clip : m_clips)
        stop(clip);
}

}  // namespace qq
