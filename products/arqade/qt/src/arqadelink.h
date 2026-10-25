// ArqadeLink: ARQADE's side of the QOR host protocol (ADR-046, ADR-102).
//
// The launcher starts ARQADE with --launcher-channel <name> and a one-time token on its stdin. The link connects, says
// hello with the token, and keeps one request waiting, `arqade.next`, which the launcher answers with its next command:
// show at a rectangle of its window, hide, wear its colours, who is playing and their Sparks, the person's preferences,
// open a game (a `qor://arqade` link named it, ADR-118), or quit. ARQADE holds no key and no token of the person's.
//
// Two other requests: signing in to ARQADE's server (`arqade.signin`), and opening ARQADE's website in the launcher's own
// guarded window (`arqade.web`), where today's versions of the games play until their rebuilds arrive. The launcher
// answers requests one at a time, so either waits for the request already waiting to come back, at most the launcher's
// short wait, and goes next.
//
// Frames are a 4-byte big-endian length and a JSON object, at most 1 MiB. When the launcher closes the channel after a
// good handshake, the link says so (lost) and ARQADE exits: a product ends with its launcher (ADR-046 decision 2).

#pragma once

#include <QByteArray>
#include <QJsonObject>
#include <QLocalSocket>
#include <QObject>
#include <QRect>
#include <QUrl>
#include <QStringList>
#include <QVariantMap>

namespace arqade {

class ArqadeLink : public QObject
{
    Q_OBJECT
    /// Connected and welcomed by a launcher that offers arqade.next.
    Q_PROPERTY(bool connected READ connected NOTIFY changed)

public:
    static constexpr const char *ProductId = "arqade";
    static constexpr int Protocol = 1;
    static constexpr qint64 FrameLimit = 1 << 20;

    explicit ArqadeLink(QObject *parent = nullptr);

    /// The token's line from stdin, where the launcher wrote it and then closed it. Empty if there is none.
    static QByteArray readToken();

    /// Connect to the launcher's channel and say hello with `token`.
    void connectTo(const QString &channel, const QByteArray &token);

    bool connected() const { return m_welcomed && m_socket.state() == QLocalSocket::ConnectedState; }

    /// Ask the launcher to sign the person in to ARQADE's server for QOR ID's sign-in page `authorizeUrl`, with the
    /// launcher's own session. Answered by signedIn(redirect) or signInFailed.
    Q_INVOKABLE void signIn(const QUrl &authorizeUrl);

    /// Ask the launcher to open ARQADE's website in its window for it. Answered by webFailed if it could not.
    Q_INVOKABLE void openWeb();

    /// Requests waiting to be sent, by method, for the tests.
    QStringList waiting() const;

signals:
    void changed();
    /// A rectangle of the launcher's window, in physical pixels, and the window's native handle (0 when none).
    void placeRequested(const QRect &rect, qint64 parent);
    void hideRequested();
    void themeRequested(const QVariantMap &colors);
    /// Who is playing: { signed_in, name, level, sparks } with `sparks` already written out by the launcher.
    void playerChanged(const QVariantMap &player);
    /// The person's preferences: { reduced_motion }.
    void prefsChanged(const QVariantMap &prefs);
    /// The launcher asks ARQADE to close: the person signed out of the launcher.
    void quitRequested();
    /// Open a game, by its card's name ("orbital", "flux"): a `qor://arqade` link named it (ADR-118).
    void playRequested(const QString &game);
    /// QOR ID's answer to a sign-in: an address on ARQADE's server that completes it.
    void signedIn(const QUrl &redirect);
    void signInFailed(const QString &code, const QString &message);
    void webFailed(const QString &message);
    /// The launcher closed the channel after welcoming ARQADE.
    void lost();

private:
    void send(const QJsonObject &frame);
    void ask(const QString &method, const QJsonObject &params);
    void queue(const QString &method, const QJsonObject &params);
    void next();
    void readFrames();
    void handle(const QJsonObject &frame);
    void command(const QJsonObject &command);

    QLocalSocket m_socket;
    QByteArray m_buffer;
    QByteArray m_token;
    bool m_welcomed = false;
    bool m_canSignIn = false;
    bool m_canOpenWeb = false;
    quint64 m_nextId = 1;
    quint64 m_waitingFor = 0;
    QString m_waitingMethod;
    /// Requests to send after the one waiting comes back, in order: at most one of each method, the newest.
    QList<QPair<QString, QJsonObject>> m_pending;
};

}  // namespace arqade
