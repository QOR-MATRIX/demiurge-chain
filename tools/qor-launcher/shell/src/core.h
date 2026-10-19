// Core: the Rust core's bridge as QML sees it (ADR-104 decision 3; tools/qor-launcher/core/include/qor_core.h).
//
// A screen asks the core something by name, with named arguments, and gets back what the core answered: a map with
// `ok`, the answer, or `error`, the core's error as the Windows launcher's interface already reads it ({kind, message}).
// Amounts come back as strings of embers, since a u128 does not fit in a QML number. Nothing here holds or can read a
// key: the core never answers with one.

#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

namespace qor {

class Core : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    /// The core's version, as the core says it.
    Q_PROPERTY(QString version READ version CONSTANT)

public:
    explicit Core(QObject *parent = nullptr) : QObject(parent) {}

    QString version() const;

    /// Ask the core `command` with `args`; answered with {ok: ...} or {error: {kind, message}}.
    Q_INVOKABLE QVariantMap call(const QString &command, const QVariantMap &args = {}) const;
};

}  // namespace qor
