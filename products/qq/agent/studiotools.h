// StudioTools: what an agent can do in QQ Studio (DIRECTION P3.3, blueprint "The agent: autonomous design and build").
//
// Each tool works the Studio the way its own panels do, through the window's functions and SceneIO, so what an agent
// changes is what the creator sees: the scene tree, the inspector, the unsaved mark. The tools:
//
//   scene_read      the scene as its canonical QML, and each entity's fields as JSON
//   kinds           what can be added: every kind's fields, their types, defaults, ranges and choices
//   scene_set       the scene's own fields (sky, light, gravity)
//   entity_add      add an entity of a kind, with fields
//   entity_set      change an entity's fields
//   entity_remove   take an entity out
//   behaviour_write write logic/<name>.qml in the project (rebuilt at once if the world is playing), and drive an entity
//   play, stop      play a copy of the scene, and throw it away
//   input_set       the player's hands in play: move, look, jump, run (as a gamepad would give them)
//   wait            let frames pass, up to ten seconds
//   view_set        where the editing eye looks from
//   frame_capture   the viewport as an image
//   log_read        what the Studio has said: notices, logic errors, QML warnings
//
// The agent cannot save or commit: the scene shows as unsaved, and the creator saves (Ctrl+S) and commits in Projects.
// Edits wait for Stop; logic and input are what play is for.

#pragma once

#include <QJsonArray>
#include <QJsonObject>
#include <QObject>
#include <QPointer>
#include <QString>
#include <QVariantMap>

#include "mcpserver.h"

class QQmlEngine;
class QQuickWindow;

namespace qq {

class SceneIO;

/// What the Studio has said, kept for the agent to read: Qt's messages (QML warnings, logic that throws), the Studio's
/// notices and play's problems. A ring of the last 500, each numbered; a line said again and again is counted.
class StudioLog : public QObject
{
    Q_OBJECT

public:
    static StudioLog *instance();
    /// Start keeping Qt's messages (passing each on to whatever handled them before).
    static void install();

    void append(const QString &level, const QString &text);
    /// Entries after `since`, oldest first, and the number to ask from next time.
    QJsonObject since(qint64 since) const;

private:
    struct Entry {
        qint64 seq;
        QString level;
        QString text;
        int repeats = 1;
    };
    QList<Entry> m_entries;
    qint64 m_next = 1;
};

class StudioTools : public QObject
{
    Q_OBJECT

public:
    /// The tools of the Studio window `studio` (the root of QQ.Studio's Main.qml), offered through `server`.
    StudioTools(QQuickWindow *studio, QQmlEngine *engine, McpServer *server, QObject *parent = nullptr);

    /// What a model should know before it calls anything: QQ's units, axes, and how a game is built here.
    static QString instructions();

private slots:
    void noticeChanged();
    void playProblemChanged();

private:
    void addTools();
    McpServer::Result kinds();
    McpServer::Result sceneRead();
    McpServer::Result entityAdd(const QJsonObject &args);
    McpServer::Result entitySet(const QJsonObject &args);
    McpServer::Result entityRemove(const QJsonObject &args);
    McpServer::Result sceneSet(const QJsonObject &args);
    McpServer::Result behaviourWrite(const QJsonObject &args);
    McpServer::Result play();
    McpServer::Result stop();
    McpServer::Result inputSet(const QJsonObject &args);
    McpServer::Result viewSet(const QJsonObject &args);
    McpServer::Result frameCapture(const QJsonObject &args);
    McpServer::Result logRead(const QJsonObject &args);
    QString status() const;

    QObject *scene() const;
    QObject *playScene() const;
    bool playing() const;
    QUrl project() const;
    QObject *entity(const QString &name) const;
    /// An entity's fields as JSON: vectors as [x, y, z], colours "#rrggbb", files relative to the project.
    QJsonObject fieldsOf(QObject *entity) const;
    QJsonValue valueOf(QObject *entity, const QString &field) const;
    /// Set `field` from JSON, checked against the field's type, range and choices: "" or what is wrong.
    QString setField(QObject *entity, const QString &field, const QJsonValue &value);
    QString setFields(QObject *entity, const QJsonObject &fields);
    /// Tell the creator what the agent did, in the Studio's notice line.
    void say(const QString &text, bool problem = false);
    void refresh();

    QPointer<QQuickWindow> m_studio;
    QQmlEngine *m_engine;
    SceneIO *m_io;
    McpServer *m_server;
    QJsonObject m_kinds;
    bool m_drivingInput = false;
};

}  // namespace qq
