// ArqadeServer: ARQADE's own server, as the app talks to it (ADR-102 decision 1, DIRECTION P7.19).
//
// The ARQADE service (products/arqade, on Vercel) keeps the rankings, the arenas and, from P7.22, the runs it confirms.
// It knows a player by its session cookie, `arq_session`, which it sets at the end of its QOR ID sign-in. The app signs
// in exactly as a browser does, with one difference: the launcher, not a web page, completes the step at QOR ID.
//
//   1. GET  <server>/api/auth/login       ARQADE answers 302 to QOR ID's sign-in page, and sets `arq_login` (its
//                                         PKCE state) on this app's cookie jar.
//   2. arqade.signin{authorize_url}       Over the launcher's channel. The launcher checks the address is QOR ID's sign-in
//                                         page, asks QOR ID with its own token (/api/v1/oauth/handoff), checks the
//                                         answer lands on ARQADE, and returns it.
//   3. GET  <redirect>                    ARQADE's callback exchanges the code with the verifier only it holds, and sets
//                                         `arq_session`.
//
// Nothing travels that a browser sign-in does not carry, and nothing is written to disk: the cookie jar lives in memory,
// so the session ends with the app, which ends when the person signs out of the launcher. No address, code or cookie is
// ever logged.
//
// The arenas (P7.24) are played through it as the website plays them: a match created, joined, moved in, resigned or
// cancelled, and asked how it stands, each answered by the match as the server keeps it (`matchAnswered`) or why not
// (`matchRefused`). Each answer names the match its request asked about, or none for a new one, since requests cross:
// a poll of a finished match can be answered after a rematch was asked for. A post carries the server's own origin, as
// the website's pages do, which its arena routes require.

#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QPointer>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

namespace arqade {

class ArqadeLink;

class ArqadeServer : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("Made by main()")
    Q_PROPERTY(State state READ state NOTIFY changed)
    /// Why the state is what it is, for the person: empty when there is nothing to say.
    Q_PROPERTY(QString message READ message NOTIFY changed)
    /// Poll while true: the app is showing and someone is looking.
    Q_PROPERTY(bool active READ active WRITE setActive NOTIFY changed)
    /// Players seen in the last 45 seconds.
    Q_PROPERTY(int online READ online NOTIFY liveChanged)
    /// The signed-in player as ARQADE knows them: { id, name }.
    Q_PROPERTY(QVariantMap me READ me NOTIFY liveChanged)
    /// Rankings by game ("all", "flux", "reversi"): lists of { rank, player, alias, wins, losses, draws, points, me }.
    Q_PROPERTY(QVariantMap standings READ standings NOTIFY liveChanged)
    /// The live arcade's matches, the player's own first: { id, game, status, alias (the host's), seat (0 for someone
    /// else's), moves }. A waiting match of someone else's is one to join.
    Q_PROPERTY(QVariantList matches READ matches NOTIFY liveChanged)

public:
    enum class State {
        /// Not asked to sign in yet.
        Idle,
        SigningIn,
        SignedIn,
        /// Not signed in, after trying, or because the launcher is not.
        SignedOut,
        /// The server did not answer; tried again on the next tick.
        Unreachable,
    };
    Q_ENUM(State)

    static constexpr int MinimumRefreshMs = 15000;
    static constexpr int RowsPerGame = 50;

    /// The server named by ARQADE_SERVER when it is set, else ARQADE's own.
    static QUrl defaultBase();

    ArqadeServer(ArqadeLink *link, const QUrl &base, QObject *parent = nullptr);

    State state() const { return m_state; }
    QString message() const { return m_message; }
    bool active() const { return m_active; }
    void setActive(bool active);
    int online() const { return m_online; }
    QVariantMap me() const { return m_me; }
    QVariantMap standings() const { return m_standings; }
    QVariantList matches() const { return m_matches; }

    /// Whether the launcher has a QOR ID session. On, ARQADE signs in once; off, it forgets its session.
    Q_INVOKABLE void setLauncherSignedIn(bool signedIn);
    Q_INVOKABLE void signIn();
    Q_INVOKABLE void refresh();

    /// The arenas (P7.24). A new match of `game`, "flux" or "reversi"; a match as it stands; and an action on one:
    /// "join", "move" (at `cell`), "resign" or "cancel", at the `revision` the player last saw. Each is answered once, by
    /// matchAnswered or matchRefused, with the id it asked about (empty for createMatch). Anything else is refused here,
    /// and nothing is sent.
    Q_INVOKABLE void createMatch(const QString &game);
    Q_INVOKABLE void fetchMatch(const QString &id);
    Q_INVOKABLE void actOnMatch(const QString &id, const QString &action, int cell, int revision);

    /// Whether `url` is on this server, scheme, host and port: the only place the app follows a sign-in to.
    bool isOurs(const QUrl &url) const;

signals:
    void changed();
    void liveChanged();
    /// A match as the server keeps it: { id, game, status, state: { cells, turn, winner, placed, flipped }, revision,
    /// moves, deadline, serverNow, host, guest, seat, legal }. `asked` is the id the request named, empty for a new match.
    void matchAnswered(const QVariantMap &match, const QString &asked);
    void matchRefused(const QString &message, const QString &asked);

private:
    QNetworkRequest request(const QUrl &url) const;
    void set(State state, const QString &message = {});
    void poll();
    void loginAnswered(class QNetworkReply *reply);
    void completed(const QUrl &redirect);
    void liveAnswered(class QNetworkReply *reply);
    void forget();
    void post(const QString &path, const QJsonObject &body, const QString &asked);
    void matchReplied(class QNetworkReply *reply, const QString &asked);

    QPointer<ArqadeLink> m_link;
    QUrl m_base;
    QNetworkAccessManager m_net;
    QTimer m_tick;
    State m_state = State::Idle;
    QString m_message;
    bool m_active = false;
    bool m_launcherSignedIn = false;
    /// Signed in again once after a session ended, until the server answers again: never in a loop.
    bool m_renewed = false;
    int m_online = 0;
    QVariantMap m_me;
    QVariantMap m_standings;
    QVariantList m_matches;
};

}  // namespace arqade
