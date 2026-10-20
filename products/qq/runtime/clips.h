// A glTF model's animations, played on the game's clock (DIRECTION P3.12, ADR-105).
//
// Qt's runtime loader turns each animation in a glTF file into a Timeline, skeletal (joints turned and moved) and morph
// (target weights) alike, and starts the first one playing on the wall clock. Clips stops that: it finds the loader's
// timelines, names them by their animations, and poses one at a time at whatever time it is told, so a Prop's clip
// moves with the game's steps, holds when the world pauses, and is the same in a replay. A timeline switched off leaves
// what it moved where it was, so Clips keeps each animated property's value from before any clip, and puts it back when
// that clip stops: changing clips, or none, returns the model to its rest pose first.

#pragma once

#include <QList>
#include <QObject>
#include <QPointer>
#include <QStringList>
#include <QVariant>
#include <QtQml/qqmlregistration.h>

namespace qq {

class Clips : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// The RuntimeLoader whose animations these are.
    Q_PROPERTY(QObject *model READ model WRITE setModel NOTIFY changed)
    /// The animations' names, in the file's order (one with no name is "clip 1", "clip 2", …).
    Q_PROPERTY(QStringList names READ names NOTIFY changed)

public:
    explicit Clips(QObject *parent = nullptr);

    QObject *model() const { return m_model; }
    void setModel(QObject *model);
    QStringList names() const;

    /// Find the model's animations again (once it has loaded), and stop Qt's own playing of them.
    Q_INVOKABLE void scan();
    /// An animation's length in seconds, or 0 if there is none by that name.
    Q_INVOKABLE double lengthOf(const QString &name) const;
    /// Pose the model as `name` has it `seconds` in, and no other animation: false if there is none by that name.
    Q_INVOKABLE bool pose(const QString &name, double seconds);
    /// Every animation off: the model as the file draws it at rest.
    Q_INVOKABLE void rest();

signals:
    void changed();

private:
    struct Rest {
        QPointer<QObject> target;
        QByteArray property;
        QVariant value;
    };
    struct Clip {
        QString name;
        QPointer<QObject> timeline;
        double end = 0;  // milliseconds, as Qt's importer keeps a timeline's frames
        QList<Rest> rest;
        bool on = false;
    };
    /// Switch a clip off and put what it moved back as it was.
    static void stop(Clip &clip);
    QPointer<QObject> m_model;
    QList<Clip> m_clips;
};

}  // namespace qq
