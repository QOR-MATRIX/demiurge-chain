// LogicFile: a game's logic, written in QML and kept in the project (`logic/<name>.qml`), built while a world plays and
// built again whenever the file is saved, without stopping the game (DIRECTION P3.2).
//
// The file's root is a Logic (Logic.qml). Each build reads the file's bytes afresh, never the engine's cache, so what
// runs is what was last saved. A build that fails leaves the logic that was running in place and says why in `error`;
// the next good save replaces it. The world itself is not rebuilt: entities stay where play has put them, and only the
// logic starts over.

#pragma once

#include <QByteArray>
#include <QObject>
#include <QPointer>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

class QFileSystemWatcher;
class QTimer;

namespace qq {

class LogicFile : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// The logic's file, as a whole URL (a file on this computer, or one bundled in qrc:, which is never rebuilt).
    Q_PROPERTY(QUrl source READ source WRITE setSource NOTIFY sourceChanged)
    /// Built only while active: while the world plays.
    Q_PROPERTY(bool active READ active WRITE setActive NOTIFY activeChanged)
    /// The running Logic, or null.
    Q_PROPERTY(QObject *object READ object NOTIFY objectChanged)
    /// Why the last build failed; empty after a good one.
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    /// How many times the logic has been built since it became active.
    Q_PROPERTY(int builds READ builds NOTIFY objectChanged)
    /// Set on the logic each time it is built, before it starts, and again whenever they change.
    Q_PROPERTY(QVariantMap properties READ properties WRITE setProperties NOTIFY propertiesChanged)

public:
    explicit LogicFile(QObject *parent = nullptr);
    ~LogicFile() override;

    QUrl source() const { return m_source; }
    void setSource(const QUrl &source);
    bool active() const { return m_active; }
    void setActive(bool active);
    QObject *object() const { return m_object; }
    QString error() const { return m_error; }
    int builds() const { return m_builds; }
    QVariantMap properties() const { return m_properties; }
    void setProperties(const QVariantMap &properties);

    /// Build again from the file now, if it has changed.
    Q_INVOKABLE void reload();

signals:
    void sourceChanged();
    void activeChanged();
    void objectChanged();
    void errorChanged();
    void propertiesChanged();

private:
    void restart();
    void watch();
    void setError(const QString &error);
    void dropObject();

    QUrl m_source;
    bool m_active = false;
    QPointer<QObject> m_object;
    QString m_error;
    int m_builds = 0;
    QVariantMap m_properties;
    QByteArray m_attempted;  // the bytes last built from, whether or not they built
    QFileSystemWatcher *m_watcher = nullptr;
    QTimer *m_settle = nullptr;
};

}  // namespace qq
