#include "core.h"

#include <QJsonDocument>
#include <QJsonObject>

#include "qor_core.h"

namespace qor {

QString Core::version() const
{
    return QString::fromUtf8(qor_core_version());
}

QVariantMap Core::call(const QString &command, const QVariantMap &args) const
{
    const QByteArray request = QJsonDocument(QJsonObject{{QStringLiteral("command"), command},
                                                         {QStringLiteral("args"), QJsonObject::fromVariantMap(args)}})
                                   .toJson(QJsonDocument::Compact);
    char *answer = qor_core_call(request.constData());
    const QByteArray text(answer);
    qor_core_free(answer);

    QJsonParseError error{};
    const QJsonDocument document = QJsonDocument::fromJson(text, &error);
    if (error.error != QJsonParseError::NoError || !document.isObject())
        return {{QStringLiteral("error"), QVariantMap{{QStringLiteral("kind"), QStringLiteral("internal")},
                                                      {QStringLiteral("message"), QStringLiteral("the core's answer was not JSON")}}}};
    return document.object().toVariantMap();
}

}  // namespace qor
