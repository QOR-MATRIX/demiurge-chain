#include "designloop.h"

#include <QHostAddress>
#include <QJsonDocument>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQuickWindow>
#include <QTimer>

#include "keychain.h"
#include "mcpserver.h"
#include "sceneio.h"
#include "studiotools.h"

namespace qq {

namespace {

constexpr int maxAttempts = 5;

QString shortened(const QString &text, int most = 400)
{
    return text.size() <= most ? text : text.left(most - 1) + QChar(0x2026);
}

QString compact(const QJsonValue &v)
{
    return QString::fromUtf8(QJsonDocument(v.toObject()).toJson(QJsonDocument::Compact));
}

/// An MCP tool result as Anthropic's tool_result content: text blocks and base64 images.
QJsonArray resultContent(const McpServer::Result &r)
{
    QJsonArray out;
    for (const QJsonValue &c : r.content) {
        const QJsonObject item = c.toObject();
        if (item.value(QStringLiteral("type")) == QStringLiteral("image")) {
            out.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("image")},
                                   {QStringLiteral("source"),
                                    QJsonObject{{QStringLiteral("type"), QStringLiteral("base64")},
                                                {QStringLiteral("media_type"), item.value(QStringLiteral("mimeType"))},
                                                {QStringLiteral("data"), item.value(QStringLiteral("data"))}}}});
        } else {
            out.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("text")},
                                   {QStringLiteral("text"), item.value(QStringLiteral("text")).toString()}});
        }
    }
    if (out.isEmpty())
        out.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("text")}, {QStringLiteral("text"), QStringLiteral("(done)")}});
    return out;
}

}  // namespace

DesignLoop::DesignLoop(McpServer *tools, QQuickWindow *studio, SceneIO *io, QObject *parent)
    : QObject(parent), m_tools(tools), m_studio(studio), m_io(io), m_net(new QNetworkAccessManager(this))
{
}

bool DesignLoop::hasKey() const
{
    return keychain::has();
}

QString DesignLoop::setKey(const QString &key)
{
    const QString trimmed = key.trimmed();
    if (trimmed.isEmpty())
        return tr("There is no key there to keep.");
    const QString why = keychain::store(trimmed);
    emit hasKeyChanged();
    return why;
}

void DesignLoop::forgetKey()
{
    keychain::remove();
    emit hasKeyChanged();
}

QUrl DesignLoop::endpoint()
{
    const QUrl fixed(QStringLiteral("https://api.anthropic.com/v1/messages"));
    const QString set = qEnvironmentVariable("QQ_ANTHROPIC_URL");
    if (set.isEmpty())
        return fixed;
    // Another server only if the key cannot be overheard on the way: https, or plain http to this computer.
    const QUrl url(set);
    const bool local = QHostAddress(url.host()).isLoopback() || url.host() == QStringLiteral("localhost");
    if (url.scheme() == QStringLiteral("https") || (url.scheme() == QStringLiteral("http") && local))
        return url;
    return fixed;
}

QString DesignLoop::systemPrompt()
{
    return StudioTools::instructions() + QStringLiteral(
        "\n\nThe creator has described a game. Make it, in this order:\n"
        "1. Brief. Before any tool, write a short design brief: the fantasy, the core loop, the controls, the look "
        "(palette, light, mood) and the scope, kept small enough to build in this scene.\n"
        "2. Build it with the tools, from what is in the scene (read it first). Name every entity for what it is.\n"
        "3. Play it. Wait for things to settle, use input_set to play as a player would, and capture frames at more "
        "than one moment. Read the log for errors.\n"
        "4. Judge the frames against the brief: is it readable at a glance, is the composition clear, does it move as "
        "it should, would the loop be fun for a minute? Stop, revise what falls short, and play again.\n"
        "5. When it meets the brief, stop play and finish with a short summary: what you built, what the creator "
        "should look at, and anything you could not do.\n"
        "Keep going without asking questions: the creator will review everything you did before keeping it.");
}

QJsonArray DesignLoop::toolDefinitions() const
{
    QJsonArray out;
    for (const QJsonValue &t : m_tools->tools()) {
        const QJsonObject tool = t.toObject();
        out.append(QJsonObject{{QStringLiteral("name"), tool.value(QStringLiteral("name"))},
                               {QStringLiteral("description"), tool.value(QStringLiteral("description"))},
                               {QStringLiteral("input_schema"), tool.value(QStringLiteral("inputSchema"))}});
    }
    return out;
}

QJsonArray DesignLoop::echo(const QJsonArray &content)
{
    int boundary = -1;
    for (int i = 0; i < content.size(); ++i) {
        if (content[i].toObject().value(QStringLiteral("type")) == QStringLiteral("fallback"))
            boundary = i;
    }
    QJsonArray out;
    for (int i = 0; i < content.size(); ++i) {
        const QString type = content[i].toObject().value(QStringLiteral("type")).toString();
        const bool replaced = i < boundary
                              && (type == QStringLiteral("thinking") || type == QStringLiteral("redacted_thinking")
                                  || type == QStringLiteral("tool_use"));
        if (!replaced)
            out.append(content[i]);
    }
    return out;
}

void DesignLoop::setStatus(const QString &status)
{
    if (status == m_status)
        return;
    m_status = status;
    emit statusChanged();
}

void DesignLoop::addStep(const QString &kind, const QString &text, const QString &image, bool failed)
{
    m_steps.append(QVariantMap{{QStringLiteral("kind"), kind},
                               {QStringLiteral("text"), text},
                               {QStringLiteral("image"), image},
                               {QStringLiteral("failed"), failed}});
    emit stepsChanged();
}

void DesignLoop::start(const QString &description)
{
    if (m_running || description.trimmed().isEmpty())
        return;
    if (!hasKey()) {
        setStatus(tr("A provider key is needed first."));
        return;
    }
    QObject *scene = m_studio ? m_studio->property("scene").value<QObject *>() : nullptr;
    if (!scene) {
        setStatus(tr("There is no scene to build in."));
        return;
    }
    if (m_studio->property("playing").toBool())
        QMetaObject::invokeMethod(m_studio, "stop");
    m_before = m_io->write(scene, m_studio->property("sceneFile").toUrl());
    m_steps.clear();
    emit stepsChanged();
    m_usage = {{QStringLiteral("input"), 0}, {QStringLiteral("cacheWrite"), 0}, {QStringLiteral("cacheRead"), 0},
               {QStringLiteral("output"), 0}, {QStringLiteral("answers"), 0}};
    emit usageChanged();
    m_messages = QJsonArray{QJsonObject{{QStringLiteral("role"), QStringLiteral("user")},
                                        {QStringLiteral("content"), description.trimmed()}}};
    m_turns = 0;
    m_attempt = 0;
    ++m_run;
    m_running = true;
    emit runningChanged();
    emit canUndoChanged();
    setStatus(tr("Writing a brief…"));
    send();
}

void DesignLoop::stop()
{
    if (!m_running)
        return;
    if (m_reply)
        m_reply->abort();
    finish(QStringLiteral("stopped"), false);
}

void DesignLoop::send()
{
    if (!m_running)
        return;
    const QJsonObject body{
        {QStringLiteral("model"), model()},
        {QStringLiteral("max_tokens"), 16000},
        {QStringLiteral("system"), systemPrompt()},
        {QStringLiteral("tools"), toolDefinitions()},
        {QStringLiteral("messages"), m_messages},
        {QStringLiteral("thinking"), QJsonObject{{QStringLiteral("type"), QStringLiteral("adaptive")}}},
        // Building and judging a game is long agentic work; Claude Opus 5.5 defaults to medium.
        {QStringLiteral("output_config"), QJsonObject{{QStringLiteral("effort"), QStringLiteral("high")}}},
        // Everything before the newest message is the same as last time: cached, and read back at a tenth of the cost.
        {QStringLiteral("cache_control"), QJsonObject{{QStringLiteral("type"), QStringLiteral("ephemeral")}}},
        // A declined request is run again on the model Anthropic recommends for why it was declined.
        {QStringLiteral("fallbacks"), QStringLiteral("default")},
    };

    QNetworkRequest request(endpoint());
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    request.setRawHeader("anthropic-version", "2023-06-01");
    request.setRawHeader("anthropic-beta", "server-side-fallback-2026-07-01");
    {
        // The key is read for this request alone and goes nowhere but its header.
        const QString key = keychain::load();
        if (key.isEmpty()) {
            finish(tr("The provider key is no longer in the keychain."), true);
            return;
        }
        request.setRawHeader("x-api-key", key.toUtf8());
    }
    request.setTransferTimeout(10 * 60 * 1000);
    m_reply = m_net->post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));
    QNetworkReply *reply = m_reply;
    connect(reply, &QNetworkReply::finished, this, [this, reply, run = m_run] {
        reply->deleteLater();
        if (run == m_run)
            received(reply);
    });
}

void DesignLoop::received(QNetworkReply *reply)
{
    if (!m_running || reply->error() == QNetworkReply::OperationCanceledError)
        return;
    const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
    const QJsonObject answer = QJsonDocument::fromJson(reply->readAll()).object();

    if (status != 200) {
        const QJsonObject error = answer.value(QStringLiteral("error")).toObject();
        const QString what = error.value(QStringLiteral("message")).toString(reply->errorString());
        const bool retryable = status == 0 || status == 408 || status == 409 || status == 429 || status >= 500;
        if (retryable && ++m_attempt < maxAttempts) {
            bool ok = false;
            const int after = reply->rawHeader("retry-after").toInt(&ok);
            const int waitMs = ok ? std::clamp(after, 0, 120) * 1000 : (1000 << std::min(m_attempt, 5));
            setStatus(status == 429 ? tr("Rate limited: trying again in %1 s.").arg(waitMs / 1000)
                                    : tr("The provider is busy: trying again in %1 s.").arg(waitMs / 1000));
            QTimer::singleShot(waitMs, this, [this, run = m_run] {
                if (run == m_run)
                    send();
            });
            return;
        }
        if (status == 401)
            finish(tr("The provider refused the key. Check it, or enter another."), true);
        else if (status == 0)
            finish(tr("The provider could not be reached: %1").arg(what), true);
        else
            finish(tr("The provider answered %1: %2").arg(status).arg(what), true);
        return;
    }
    m_attempt = 0;

    // What this answer used on the creator's key, as the provider counted it.
    const QJsonObject used = answer.value(QStringLiteral("usage")).toObject();
    auto add = [this, &used](const char *field, const char *from) {
        m_usage[QLatin1String(field)] = m_usage.value(QLatin1String(field)).toLongLong()
                                        + used.value(QLatin1String(from)).toInteger();
    };
    add("input", "input_tokens");
    add("cacheWrite", "cache_creation_input_tokens");
    add("cacheRead", "cache_read_input_tokens");
    add("output", "output_tokens");
    m_usage[QStringLiteral("answers")] = m_usage.value(QStringLiteral("answers")).toInt() + 1;
    emit usageChanged();

    const QJsonArray content = answer.value(QStringLiteral("content")).toArray();
    const QString stop = answer.value(QStringLiteral("stop_reason")).toString();
    // Kept as it came (append-only), before anything is acted on.
    m_messages.append(QJsonObject{{QStringLiteral("role"), QStringLiteral("assistant")},
                                  {QStringLiteral("content"), echo(content)}});
    ++m_turns;

    QJsonArray uses;
    for (const QJsonValue &v : echo(content)) {
        const QJsonObject block = v.toObject();
        const QString type = block.value(QStringLiteral("type")).toString();
        if (type == QStringLiteral("text") && !block.value(QStringLiteral("text")).toString().trimmed().isEmpty())
            addStep(QStringLiteral("said"), block.value(QStringLiteral("text")).toString());
        else if (type == QStringLiteral("tool_use"))
            uses.append(block);
    }

    if (stop == QStringLiteral("refusal")) {
        finish(tr("The model declined to go on with this."), true);
    } else if (stop == QStringLiteral("max_tokens")) {
        // A tool call cut off is not run: its input may be incomplete.
        finish(tr("The model's answer was cut off; nothing in it was run."), true);
    } else if (stop == QStringLiteral("tool_use") && !uses.isEmpty()) {
        if (m_turns >= maxTurns) {
            finish(tr("Stopped after %1 answers, the most one run may take.").arg(maxTurns), true);
            return;
        }
        runTools(uses, 0, {});
    } else if (stop == QStringLiteral("pause_turn")) {
        send();
    } else {
        finish(QStringLiteral("done"), false);
    }
}

void DesignLoop::runTools(const QJsonArray &uses, int next, QJsonArray results)
{
    if (!m_running)
        return;
    if (next >= uses.size()) {
        // Every result of a turn goes back in one message.
        m_messages.append(QJsonObject{{QStringLiteral("role"), QStringLiteral("user")}, {QStringLiteral("content"), results}});
        setStatus(tr("Thinking…"));
        send();
        return;
    }
    const QJsonObject use = uses[next].toObject();
    const QString name = use.value(QStringLiteral("name")).toString();
    setStatus(tr("Using %1…").arg(name));
    QPointer<DesignLoop> self(this);
    m_tools->call(name, use.value(QStringLiteral("input")).toObject(),
                  [self, uses, next, results, use, name, run = m_run](const McpServer::Result &r) mutable {
                      if (!self || !self->m_running || self->m_run != run)
                          return;
                      QString image;
                      QString text;
                      for (const QJsonValue &c : r.content) {
                          const QJsonObject item = c.toObject();
                          if (item.value(QStringLiteral("type")) == QStringLiteral("image"))
                              image = QStringLiteral("data:%1;base64,%2")
                                          .arg(item.value(QStringLiteral("mimeType")).toString(),
                                               item.value(QStringLiteral("data")).toString());
                          else if (text.isEmpty())
                              text = item.value(QStringLiteral("text")).toString();
                      }
                      const QString input = compact(use.value(QStringLiteral("input")));
                      self->addStep(image.isEmpty() ? QStringLiteral("tool") : QStringLiteral("frame"),
                                    shortened(name + QLatin1Char(' ') + input) + (r.isError ? QStringLiteral(" — ") + shortened(text, 200) : QString()),
                                    image, r.isError);
                      QJsonObject result{{QStringLiteral("type"), QStringLiteral("tool_result")},
                                         {QStringLiteral("tool_use_id"), use.value(QStringLiteral("id"))},
                                         {QStringLiteral("content"), resultContent(r)}};
                      if (r.isError)
                          result.insert(QStringLiteral("is_error"), true);
                      results.append(result);
                      self->runTools(uses, next + 1, results);
                  });
}

void DesignLoop::finish(const QString &outcome, bool problem)
{
    if (!m_running)
        return;
    m_running = false;
    // Play is not left running behind the creator's back.
    if (m_studio && m_studio->property("playing").toBool())
        m_tools->call(QStringLiteral("stop"), {}, [](const McpServer::Result &) {});
    if (problem)
        addStep(QStringLiteral("problem"), outcome, {}, true);
    else
        addStep(QStringLiteral("done"), outcome == QStringLiteral("done") ? tr("Done. Review it, then save and commit it, or undo it.")
                                                                          : tr("Stopped."));
    setStatus(problem ? outcome : outcome == QStringLiteral("done") ? tr("Done: review it.") : tr("Stopped."));
    emit runningChanged();
    emit canUndoChanged();
    emit finished(outcome);
}

bool DesignLoop::undo()
{
    if (m_running || m_before.isEmpty() || !m_studio)
        return false;
    QVariant done;
    QMetaObject::invokeMethod(m_studio, "restoreText", Q_RETURN_ARG(QVariant, done), Q_ARG(QVariant, m_before));
    if (!done.toBool())
        return false;
    m_before.clear();
    emit canUndoChanged();
    setStatus(tr("Put back as it was before the run."));
    return true;
}

}  // namespace qq
