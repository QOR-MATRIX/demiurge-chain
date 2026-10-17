// MediaLink: QOR Media's side of the QOR host protocol (ADR-046, ADR-094; docs/blueprints/library.md section 1).
//
// The launcher starts QOR Media with --launcher-channel <name> and a one-time token on its stdin. The link connects,
// says hello with the token, and then keeps one request waiting, `media.next`, which the launcher answers with its next
// command (open a file, sit at a rectangle of its window, hide, wear its colours) or with nothing after its wait; the link
// asks again at once. What QOR Media has to tell the launcher (where playback stopped, a file's length and size, that it
// closed) is queued and rides on the next `media.next`. QOR Media never holds a key or a token of the person's.
//
// Frames are a 4-byte big-endian length and a JSON object, at most 1 MiB. When the launcher closes the channel after a
// good handshake the link says so (lost), and QOR Media exits: a product ends with its launcher (ADR-046 decision 2).

#pragma once

#include <QByteArray>
#include <QJsonArray>
#include <QJsonObject>
#include <QLocalSocket>
#include <QObject>
#include <QRect>
#include <QVariantMap>

namespace qormedia {

class MediaLink : public QObject
{
    Q_OBJECT
    /// Connected and welcomed by a launcher that offers media.next.
    Q_PROPERTY(bool connected READ connected NOTIFY changed)

public:
    static constexpr const char *ProductId = "qor-media";
    static constexpr int Protocol = 1;
    static constexpr qint64 FrameLimit = 1 << 20;

    explicit MediaLink(QObject *parent = nullptr);

    /// The token's line from stdin, where the launcher wrote it and then closed it. Empty if there is none.
    static QByteArray readToken();

    /// Connect to the launcher's channel and say hello with `token`.
    void connectTo(const QString &channel, const QByteArray &token);

    bool connected() const { return m_welcomed && m_socket.state() == QLocalSocket::ConnectedState; }

    /// Where playback of `path` stopped, in seconds.
    Q_INVOKABLE void reportPosition(const QString &path, double seconds);
    /// A file's length (seconds, or 0 if unknown) and picture size (0 if none), learned as it played.
    Q_INVOKABLE void reportLearned(const QString &path, double duration, int width, int height);
    /// The person closed what was open.
    Q_INVOKABLE void reportClosed();

    /// Reports waiting for the next media.next, for the tests.
    QJsonArray pendingReports() const { return m_reports; }

signals:
    void changed();
    void openRequested(const QString &path, const QString &kind, double position);
    /// A rectangle of the launcher's window, in physical pixels, and the window's native handle (0 when none).
    void placeRequested(const QRect &rect, qint64 parent);
    void hideRequested();
    void themeRequested(const QVariantMap &colors);
    /// The launcher closed the channel after welcoming QOR Media.
    void lost();

private:
    void send(const QJsonObject &frame);
    void next();
    void readFrames();
    void handle(const QJsonObject &frame);
    void command(const QJsonObject &command);
    void report(const QJsonObject &report);

    QLocalSocket m_socket;
    QByteArray m_buffer;
    QByteArray m_token;
    bool m_welcomed = false;
    quint64 m_nextId = 1;
    quint64 m_waitingFor = 0;
    QJsonArray m_reports;
};

}  // namespace qormedia
