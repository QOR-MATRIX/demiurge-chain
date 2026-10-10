// SceneIO: QQ scenes to and from their files (ADR-083 decision 3, DIRECTION P3.1).
//
// A scene file is QML, `scenes/<name>.qml` in a Qontrol project, in one canonical layout: an import line, the Scene with
// its fields one per line, then each entity as a block with its fields one per line, all in the order each type
// declares. The same scene is always the same bytes, and changing one value changes one line, so Projects shows a
// creator's change as what it is.
//
// The writer knows no type: every entity declares `kind` (its QML type's name) and `fields` (what a file keeps). A value
// is written by its type: numbers to four decimal places with trailing zeros dropped, colours as "#rrggbb", vectors as
// Qt.vector3d(x, y, z), URLs relative to the scene file.

#pragma once

#include <QObject>
#include <QString>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

class QQmlEngine;
class QJSEngine;

namespace qq {

class SceneIO : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    explicit SceneIO(QObject *parent = nullptr);

    static SceneIO *create(QQmlEngine *qml, QJSEngine *js);

    /// The canonical text of `scene`. With `file`, URLs are written relative to the file's folder.
    Q_INVOKABLE QString write(QObject *scene, const QUrl &file = {}) const;

    /// Write `scene` to `file`, whole or not at all. An empty string, or what went wrong.
    Q_INVOKABLE QString save(QObject *scene, const QUrl &file) const;

    /// Build the scene in `file` (on this computer, or bundled in qrc:) under `parent` (a 3D node). The new scene, or
    /// null with `lastError` set.
    Q_INVOKABLE QObject *load(const QUrl &file, QObject *parent);

    /// Build a scene from its text, as if read from `file` (which relative URLs resolve against).
    Q_INVOKABLE QObject *loadText(const QString &text, const QUrl &file, QObject *parent);

    /// Play `scene`: a copy built from its text under `parent`, with `playing` set. The scene itself is not touched, so
    /// stopping is discarding the copy. The copy, or null with `lastError` set.
    Q_INVOKABLE QObject *play(QObject *scene, const QUrl &file, QObject *parent);

    /// A URL an entity holds, made whole against the file the entity was read from (Qt 6 keeps URL properties as
    /// written, so a model's "../assets/x.glb" stays relative until something resolves it).
    Q_INVOKABLE QUrl resolvedUrl(QObject *entity, const QUrl &url) const;

    /// The type of one of an object's properties, as the inspector edits it: "real", "int", "bool", "string", "url",
    /// "color" or "vector3d"; "" for anything else.
    Q_INVOKABLE QString fieldType(QObject *object, const QString &name) const;

    /// Copy a file an entity uses into the project: a model (and every file it refers to) or a sound into `assets/`, a
    /// logic file into `logic/`. Its new URL, or an empty URL with `lastError` set. A file already inside the project is
    /// left where it is.
    Q_INVOKABLE QUrl adopt(const QUrl &model, const QUrl &project);

    /// Write a logic file, `logic/<name>.qml` in the project, whole or not at all: how the Studio starts a new behaviour,
    /// and how the agent writes one (P3.3). `name` is made safe for a file name. Its URL, or an empty URL with
    /// `lastError` set.
    Q_INVOKABLE QUrl writeLogic(const QUrl &project, const QString &name, const QString &text);

    /// A 2D QQ scene of format 1 (`.qq.json`, ADR-082's preview) as a canonical QQ scene: { "qml": text, "imported":
    /// how many entities it became, "skipped": [what has no 3D form yet], "error": "" or why it could not be read }.
    Q_INVOKABLE QVariantMap importQqJson(const QUrl &file);

    // ── Editing: what the Studio's panels do, and what the agent's tools will do (P3.3) ──

    /// The kinds an entity can be: "Shape", "Lamp", "Prop", "Sun", "Ground", "Player", "Emitter", "Sound", "Behaviour".
    Q_INVOKABLE QStringList kinds() const;

    /// The scene's entities, in file order.
    Q_INVOKABLE QVariantList entities(QObject *scene) const;

    /// Add an entity of `kind` to the scene, with `properties` set; it, or null with `lastError` set.
    Q_INVOKABLE QObject *add(QObject *scene, const QString &kind, const QVariantMap &properties = {});

    /// Take an entity out of its scene and delete it. False if it is not an entity of a scene.
    Q_INVOKABLE bool remove(QObject *entity);

    /// Put a whole scene away (the Studio, opening another or stopping play). Its physics bodies are destroyed at once,
    /// before their world; everything else when control returns to the event loop.
    Q_INVOKABLE void discard(QObject *scene);

    /// A project folder as the Studio shows it: { "name", "repository": whether it is a Qontrol (git) repository,
    /// "scenes": the names of scenes/<name>.qml, sorted, "logic": the names of logic/<name>.qml, sorted }.
    Q_INVOKABLE QVariantMap project(const QUrl &folder) const;

    Q_PROPERTY(QString lastError READ lastError NOTIFY lastErrorChanged)
    QString lastError() const { return m_lastError; }

signals:
    void lastErrorChanged();

private:
    void fail(const QString &why);
    QQmlEngine *engine() const;

    QQmlEngine *m_engine = nullptr;
    QString m_lastError;
};

/// A number as a scene file writes it: four decimal places at most, no trailing zeros, never "-0".
QString formatNumber(double value);

}  // namespace qq
