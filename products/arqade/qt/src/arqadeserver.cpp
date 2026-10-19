#include "arqadeserver.h"

#include "arqadelink.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkReply>
#include <QUrlQuery>

#include <algorithm>

namespace arqade {

namespace {

const QByteArray SessionCookie = QByteArrayLiteral("arq_session");

QString signInRefused(const QString &code, const QString &message)
{
    if (code == QLatin1String("not_authenticated"))
        return QObject::tr("Sign in to the launcher with your QOR ID, and ARQADE signs you in with it.");
    if (code == QLatin1String("unsupported_method"))
        return message;
    return QObject::tr("ARQADE could not sign you in: %1").arg(message);
}

QString whyNot(const QUrl &back)
{
    // ARQADE's own sign-in sends a browser home with ?qor_error=… when it cannot start or finish.
    const QString reason = QUrlQuery(back).queryItemValue(QStringLiteral("qor_error"));
    if (reason == QLatin1String("unconfigured"))
        return QObject::tr("ARQADE's sign-in is not set up on its server yet.");
    if (reason == QLatin1String("refused"))
        return QObject::tr("QOR ID did not complete ARQADE's sign-in. Try again.");
    return QObject::tr("ARQADE's server could not start a sign-in. Try again shortly.");
}

}  // namespace

QUrl ArqadeServer::defaultBase()
{
    const QByteArray named = qgetenv("ARQADE_SERVER");
    const QUrl base(named.isEmpty() ? QStringLiteral("https://qor-arqade-tau.vercel.app/") : QString::fromUtf8(named));
    return base.isValid() && !base.isRelative() ? base : QUrl(QStringLiteral("https://qor-arqade-tau.vercel.app/"));
}

ArqadeServer::ArqadeServer(ArqadeLink *link, const QUrl &base, QObject *parent)
    : QObject(parent), m_link(link), m_base(base)
{
    m_tick.setInterval(MinimumRefreshMs);
    connect(&m_tick, &QTimer::timeout, this, &ArqadeServer::refresh);
    if (m_link) {
        connect(m_link, &ArqadeLink::signedIn, this, &ArqadeServer::completed);
        connect(m_link, &ArqadeLink::signInFailed, this,
                [this](const QString &code, const QString &message) { set(State::SignedOut, signInRefused(code, message)); });
    }
}

bool ArqadeServer::isOurs(const QUrl &url) const
{
    return url.scheme() == m_base.scheme() && url.host().compare(m_base.host(), Qt::CaseInsensitive) == 0
        && url.port(url.scheme() == QLatin1String("https") ? 443 : 80)
        == m_base.port(m_base.scheme() == QLatin1String("https") ? 443 : 80);
}

QNetworkRequest ArqadeServer::request(const QUrl &url) const
{
    QNetworkRequest r(url);
    // Every redirect is read here, never followed: a sign-in's next step is chosen by this code, not by a server.
    r.setAttribute(QNetworkRequest::RedirectPolicyAttribute, QNetworkRequest::ManualRedirectPolicy);
    r.setTransferTimeout(15000);
    r.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("ARQADE/0.1.0 (QOR Launcher)"));
    return r;
}

void ArqadeServer::set(State state, const QString &message)
{
    if (state == m_state && message == m_message)
        return;
    m_state = state;
    m_message = message;
    poll();
    emit changed();
}

void ArqadeServer::poll()
{
    // Asked again while it is signed in, or until it can be reached; never while nobody is looking.
    const bool polling = m_active && (m_state == State::SignedIn || m_state == State::Unreachable);
    if (polling && !m_tick.isActive())
        m_tick.start();
    else if (!polling)
        m_tick.stop();
}

void ArqadeServer::setActive(bool active)
{
    if (active == m_active)
        return;
    m_active = active;
    poll();
    if (m_tick.isActive())
        refresh();
    emit changed();
}

void ArqadeServer::setLauncherSignedIn(bool signedIn)
{
    const bool was = m_launcherSignedIn;
    m_launcherSignedIn = signedIn;
    if (!signedIn) {
        forget();
        set(State::SignedOut, tr("Sign in to the launcher with your QOR ID, and ARQADE signs you in with it."));
    } else if (!was && m_state != State::SignedIn && m_state != State::SigningIn) {
        signIn();
    }
}

void ArqadeServer::forget()
{
    m_net.setCookieJar(new QNetworkCookieJar(&m_net));
    m_online = 0;
    m_me.clear();
    m_standings.clear();
    emit liveChanged();
}

void ArqadeServer::signIn()
{
    if (m_state == State::SigningIn)
        return;
    if (!m_link || !m_link->connected()) {
        set(State::SignedOut, tr("Open ARQADE from the QOR Launcher to sign in."));
        return;
    }
    set(State::SigningIn);
    QNetworkReply *reply = m_net.get(request(m_base.resolved(QUrl(QStringLiteral("api/auth/login")))));
    connect(reply, &QNetworkReply::finished, this, [this, reply] { loginAnswered(reply); });
}

void ArqadeServer::loginAnswered(QNetworkReply *reply)
{
    reply->deleteLater();
    const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
    const QUrl target = reply->attribute(QNetworkRequest::RedirectionTargetAttribute).toUrl();
    if (status == 0) {
        set(State::Unreachable, tr("ARQADE's server could not be reached. Trying again shortly."));
        return;
    }
    if (status < 300 || status >= 400 || !target.isValid()) {
        set(State::SignedOut, tr("ARQADE's server could not start a sign-in. Try again shortly."));
        return;
    }
    const QUrl next = reply->url().resolved(target);
    // Home again means it could not start: unconfigured, or its own failure.
    if (isOurs(next)) {
        set(State::SignedOut, whyNot(next));
        return;
    }
    // QOR ID's sign-in page. The launcher checks it is, and is the one that talks to QOR ID.
    m_link->signIn(next);
}

void ArqadeServer::completed(const QUrl &redirect)
{
    if (m_state != State::SigningIn)
        return;
    if (!isOurs(redirect)) {
        set(State::SignedOut, tr("The sign-in did not lead back to ARQADE, so it was not followed."));
        return;
    }
    QNetworkReply *reply = m_net.get(request(redirect));
    connect(reply, &QNetworkReply::finished, this, [this, reply] {
        reply->deleteLater();
        const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
        if (status == 0) {
            set(State::Unreachable, tr("ARQADE's server could not be reached. Trying again shortly."));
            return;
        }
        const QList<QNetworkCookie> held = m_net.cookieJar()->cookiesForUrl(m_base);
        const bool session = std::any_of(held.begin(), held.end(), [](const QNetworkCookie &c) {
            return c.name() == SessionCookie && !c.value().isEmpty();
        });
        if (!session) {
            const QUrl target = reply->attribute(QNetworkRequest::RedirectionTargetAttribute).toUrl();
            set(State::SignedOut, whyNot(reply->url().resolved(target)));
            return;
        }
        set(State::SignedIn);
        refresh();
    });
}

void ArqadeServer::refresh()
{
    if (m_state == State::Unreachable && m_launcherSignedIn && m_me.isEmpty()) {
        // It never got as far as a session: start again.
        set(State::Idle);
        signIn();
        return;
    }
    if (m_state != State::SignedIn && m_state != State::Unreachable)
        return;
    QNetworkReply *reply = m_net.get(request(m_base.resolved(QUrl(QStringLiteral("api/live")))));
    connect(reply, &QNetworkReply::finished, this, [this, reply] { liveAnswered(reply); });
}

void ArqadeServer::liveAnswered(QNetworkReply *reply)
{
    reply->deleteLater();
    const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
    if (status == 401) {
        // The session ended at ARQADE (it lasts eight hours) or at QOR ID: sign in again, once.
        forget();
        set(State::SignedOut, tr("Your ARQADE session ended."));
        if (m_launcherSignedIn && !m_renewed) {
            m_renewed = true;
            signIn();
        }
        return;
    }
    if (status != 200) {
        set(State::Unreachable, status == 0 ? tr("ARQADE's server could not be reached. Trying again shortly.")
                                            : tr("ARQADE's server is busy. Trying again shortly."));
        return;
    }
    m_renewed = false;
    const QJsonObject live = QJsonDocument::fromJson(reply->readAll()).object();
    const QJsonObject me = live.value(QStringLiteral("me")).toObject();
    m_me = {{QStringLiteral("id"), me.value(QStringLiteral("id")).toString()},
            {QStringLiteral("name"), me.value(QStringLiteral("name")).toString()}};
    m_online = live.value(QStringLiteral("online")).toVariant().toInt();

    QHash<QString, QVariantList> games;
    for (const QJsonValue &row : live.value(QStringLiteral("standings")).toArray()) {
        const QJsonObject s = row.toObject();
        const QString game = s.value(QStringLiteral("game")).toString();
        QVariantList &list = games[game];
        if (game.isEmpty() || list.size() >= RowsPerGame)
            continue;
        // Sums arrive as numbers or, from Postgres, as strings: both are read as whole numbers.
        const auto whole = [&s](const char *key) { return s.value(QLatin1String(key)).toVariant().toLongLong(); };
        const QString player = s.value(QStringLiteral("player")).toString();
        list.append(QVariantMap{{QStringLiteral("rank"), list.size() + 1},
                                {QStringLiteral("player"), player},
                                {QStringLiteral("alias"), s.value(QStringLiteral("alias")).toString()},
                                {QStringLiteral("wins"), whole("wins")},
                                {QStringLiteral("losses"), whole("losses")},
                                {QStringLiteral("draws"), whole("draws")},
                                {QStringLiteral("points"), whole("points")},
                                {QStringLiteral("me"), !player.isEmpty() && player == m_me.value(QStringLiteral("id"))}});
    }
    m_standings.clear();
    for (auto it = games.cbegin(); it != games.cend(); ++it)
        m_standings.insert(it.key(), it.value());

    const int every = live.value(QStringLiteral("refreshMs")).toInt();
    m_tick.setInterval(std::max(every, MinimumRefreshMs));
    set(State::SignedIn);
    emit liveChanged();
}

}  // namespace arqade
