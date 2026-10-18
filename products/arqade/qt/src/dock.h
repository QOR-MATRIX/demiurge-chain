// Dock: ARQADE's window inside the launcher's (ADR-102 decision 1), exactly as QOR Media's (tools/qor-media/src/dock.h,
// ADR-094 decision 1).
//
// The launcher sends its window's native handle and a rectangle of its client area in physical pixels. ARQADE's window
// becomes a child of the launcher's (Qt's foreign-window parenting), so it moves, resizes, minimises and stacks with it,
// and sits over the launcher's ARQADE screen exactly. Full screen lifts it out to a top-level window on the same screen
// and puts it back. Without a launcher (started by hand), it is an ordinary window.

#pragma once

#include <QObject>
#include <QPointer>
#include <QRect>
#include <QWindow>
#include <QtQml/qqmlregistration.h>

namespace arqade {

class Dock : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool docked READ docked NOTIFY changed)
    Q_PROPERTY(bool fullScreen READ fullScreen NOTIFY changed)

public:
    explicit Dock(QObject *parent = nullptr) : QObject(parent) {}

    bool docked() const { return m_parent != 0; }
    bool fullScreen() const { return m_fullScreen; }

    /// Sit at `rect` (physical pixels) of the window `parent` names; 0 for no parent.
    Q_INVOKABLE void place(QWindow *window, const QRect &rect, qint64 parent);
    /// Lift out to full screen, or back into the launcher.
    Q_INVOKABLE void toggleFullScreen(QWindow *window);

    /// A rectangle in physical pixels as the device-independent one Qt places a child window by.
    static QRect logical(const QRect &physical, qreal ratio);

signals:
    void changed();

private:
    void apply(QWindow *window);

    qint64 m_parent = 0;
    QRect m_rect;
    QPointer<QWindow> m_host;
    bool m_fullScreen = false;
};

}  // namespace arqade
