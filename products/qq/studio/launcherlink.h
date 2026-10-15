// LauncherLink: QQ Studio's side of the QOR host protocol (ADR-046, ADR-088; DIRECTION P3.4).
//
// The launcher starts the Studio with --launcher-channel <name> and a one-time token on its stdin. The link connects
// to that channel, proves it is the process the launcher started (the token, and the operating system's word, which
// the launcher checks), and then asks the launcher, never itself, to commit a project and to publish it as its
// Cartridge. The Studio holds no key and never sees a transaction's bytes: the launcher builds it and asks the person in
// its own dialog, which names QQ Studio.
//
// Frames are a 4-byte big-endian length and a JSON object, at most 1 MiB. When the launcher closes the channel after a
// good handshake the link says so (lost), and the Studio exits: a product ends with its launcher (ADR-046 decision 2).
// A Studio started any other way works as before, but cannot publish; `reason` says why.

#pragma once

#include <QByteArray>
#include <QJsonObject>
#include <QLocalSocket>
#include <QObject>
#include <QStringList>
#include <QUrl>
#include <QVariantMap>

namespace qq {

class LauncherLink : public QObject
{
    Q_OBJECT
    /// Connected and welcomed: Publish can be asked for.
    Q_PROPERTY(bool available READ available NOTIFY changed)
    /// A publish is under way.
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    /// Why it is not available, in words for the person; empty when it is.
    Q_PROPERTY(QString reason READ reason NOTIFY changed)

public:
    /// The id the launcher started the Studio as; the hello must say the same.
    static constexpr const char *ProductId = "qq-studio";
    static constexpr int Protocol = 1;
    /// The largest frame either side sends: 1 MiB.
    static constexpr qint64 FrameLimit = 1 << 20;

    explicit LauncherLink(QObject *parent = nullptr);

    /// The token's line from stdin, where the launcher wrote it and then closed it. Empty if there is none.
    static QByteArray readToken();

    /// Connect to the launcher's channel and say hello with `token`.
    void connectTo(const QString &channel, const QByteArray &token);
    /// Not started by the launcher: say so, and offer nothing.
    void unavailable(const QString &reason);

    bool available() const { return m_welcomed && m_socket.state() == QLocalSocket::ConnectedState; }
    bool busy() const { return m_step != Step::Idle; }
    QString reason() const { return m_reason; }
    QStringList capabilities() const { return m_capabilities; }

    /// Commit everything changed in `project` with `message`, then publish it as its Cartridge. Ends in published()
    /// or failed().
    Q_INVOKABLE void publish(const QUrl &project, const QString &message);

signals:
    void changed();
    /// Done: `answer` is the launcher's (`action` minted, revised or unchanged; the receipt or the Cartridge).
    void published(const QVariantMap &answer);
    /// Refused or failed: a stable code (declined, vault_locked, qontrol, rpc, …) and a sentence for the person.
    void failed(const QString &code, const QString &message);
    /// The launcher closed the channel after welcoming the Studio.
    void lost();

private:
    enum class Step { Idle, Committing, Publishing };

    void send(const QJsonObject &frame);
    void request(const QString &method, const QJsonObject &params);
    void readFrames();
    void handle(const QJsonObject &frame);
    void finish();
    void fail(const QString &code, const QString &message);

    QLocalSocket m_socket;
    QByteArray m_buffer;
    QByteArray m_token;
    bool m_welcomed = false;
    QStringList m_capabilities;
    QString m_reason;
    quint64 m_nextId = 1;
    quint64 m_waitingFor = 0;
    Step m_step = Step::Idle;
    QString m_project;
};

}  // namespace qq
