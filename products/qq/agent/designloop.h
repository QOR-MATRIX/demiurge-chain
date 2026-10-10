// DesignLoop: a game designed and built from a description, in QQ Studio (DIRECTION P3.3, the blueprint's "design
// loop", ADR-085).
//
// The creator writes what they want ("a neon rooftop chase at night, one minute long"). A model (Claude, through
// Anthropic's Messages API) writes a design brief, builds the scene with the Studio's own tools (the same fourteen an MCP
// client gets, called here without the protocol), plays it, looks at captured frames, judges them against the brief
// and revises, until it is done or the creator stops it. It cannot save or commit: what it built shows as unsaved, the
// creator reviews it, and either saves and commits it or takes it all back (`undo`).
//
// The provider's key lives in the operating system's keychain (keychain.h) and is read only to make a request; it is
// never logged. The conversation is append-only: each answer is kept as it came, and each turn's tool results go back
// together in one message. Rate limits and overload are waited out (as `retry-after` says, or with growing pauses); a
// refused key, a bad request or a refusal ends the run and says why.
//
// QQ_ANTHROPIC_URL points it at another server (https, or http on this computer only): how the checks run it against a
// stand-in without a key or a bill.

#pragma once

#include <QJsonArray>
#include <QJsonObject>
#include <QObject>
#include <QPointer>
#include <QString>
#include <QUrl>
#include <QVariantList>

class QNetworkAccessManager;
class QNetworkReply;
class QQuickWindow;

namespace qq {

class McpServer;
class SceneIO;

class DesignLoop : public QObject
{
    Q_OBJECT

    Q_PROPERTY(bool running READ running NOTIFY runningChanged)
    /// Whether a provider key is kept in the keychain (the key itself is never handed out).
    Q_PROPERTY(bool hasKey READ hasKey NOTIFY hasKeyChanged)
    /// One line on where the run stands.
    Q_PROPERTY(QString status READ status NOTIFY statusChanged)
    /// What has happened, for the panel: { kind ("said", "tool", "frame", "problem", "done"), text, image, failed }.
    Q_PROPERTY(QVariantList steps READ steps NOTIFY stepsChanged)
    /// Whether there is a scene from before the last run to go back to.
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY canUndoChanged)

public:
    DesignLoop(McpServer *tools, QQuickWindow *studio, SceneIO *io, QObject *parent = nullptr);

    bool running() const { return m_running; }
    bool hasKey() const;
    QString status() const { return m_status; }
    QVariantList steps() const { return m_steps; }
    bool canUndo() const { return !m_before.isEmpty() && !m_running; }

    /// Design and build what `description` says, from the scene as it is now.
    Q_INVOKABLE void start(const QString &description);
    /// Stop after nothing more: the request in flight is dropped, a tool running finishes, and play is stopped.
    Q_INVOKABLE void stop();
    /// Keep `key` in the keychain. "" or why it could not be kept.
    Q_INVOKABLE QString setKey(const QString &key);
    Q_INVOKABLE void forgetKey();
    /// Put the scene back as it was when the last run started. Logic files it wrote stay in the project's logic folder.
    Q_INVOKABLE bool undo();

    /// The model, and how it works: what ADR-085 records.
    static QString model() { return QStringLiteral("claude-opus-5-5"); }
    static QString systemPrompt();
    /// Where requests go: Anthropic's Messages API, or QQ_ANTHROPIC_URL if it is allowed (https, or http on loopback).
    static QUrl endpoint();
    /// At most this many answers from the model in one run.
    static constexpr int maxTurns = 60;

signals:
    void runningChanged();
    void hasKeyChanged();
    void statusChanged();
    void stepsChanged();
    void canUndoChanged();
    /// A run ended: "done", "stopped", or what went wrong.
    void finished(const QString &outcome);

private:
    void send();
    void received(QNetworkReply *reply);
    void runTools(const QJsonArray &uses, int next, QJsonArray results);
    void finish(const QString &outcome, bool problem);
    void addStep(const QString &kind, const QString &text, const QString &image = {}, bool failed = false);
    void setStatus(const QString &status);
    QJsonArray toolDefinitions() const;
    /// What of an answer goes back into the conversation: everything, but for what a fallback replaced (thinking and
    /// tool calls before the last `fallback` block), as the API asks.
    static QJsonArray echo(const QJsonArray &content);

    McpServer *m_tools;
    QPointer<QQuickWindow> m_studio;
    SceneIO *m_io;
    QNetworkAccessManager *m_net;
    QPointer<QNetworkReply> m_reply;
    QJsonArray m_messages;
    bool m_running = false;
    int m_turns = 0;
    int m_attempt = 0;
    /// Which run this is: anything left over from a run stopped before (a reply, a timer, a tool) is ignored.
    int m_run = 0;
    QString m_status;
    QVariantList m_steps;
    QString m_before;
};

}  // namespace qq
