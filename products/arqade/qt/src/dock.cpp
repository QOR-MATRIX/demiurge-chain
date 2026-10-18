#include "dock.h"

#include <QGuiApplication>
#include <QScreen>

#include <cmath>

namespace arqade {

QRect Dock::logical(const QRect &physical, qreal ratio)
{
    if (ratio <= 0)
        ratio = 1;
    return QRect(int(std::lround(physical.x() / ratio)), int(std::lround(physical.y() / ratio)),
                 int(std::lround(physical.width() / ratio)), int(std::lround(physical.height() / ratio)));
}

void Dock::place(QWindow *window, const QRect &rect, qint64 parent)
{
    if (!window || rect.width() <= 0 || rect.height() <= 0)
        return;
    m_rect = rect;
    if (parent != m_parent) {
        m_parent = parent;
        delete m_host;
        m_host = parent ? QWindow::fromWinId(WId(parent)) : nullptr;
    }
    emit changed();
    if (!m_fullScreen)
        apply(window);
}

void Dock::apply(QWindow *window)
{
    if (m_host) {
        window->setFlags(Qt::FramelessWindowHint);
        window->setParent(m_host);
        // A child's position is in the parent's device-independent pixels; the launcher measured physical ones.
        const qreal ratio = m_host->screen() ? m_host->screen()->devicePixelRatio() : window->devicePixelRatio();
        window->setGeometry(logical(m_rect, ratio));
    } else {
        window->setParent(nullptr);
        window->setFlags(Qt::Window);
        if (m_rect.isValid())
            window->setGeometry(logical(m_rect, window->devicePixelRatio()));
    }
}

void Dock::toggleFullScreen(QWindow *window)
{
    if (!window)
        return;
    if (!m_fullScreen) {
        QScreen *screen = m_host && m_host->screen() ? m_host->screen() : window->screen();
        window->setParent(nullptr);
        window->setFlags(Qt::Window | Qt::FramelessWindowHint);
        if (screen)
            window->setScreen(screen);
        window->showFullScreen();
        m_fullScreen = true;
    } else {
        m_fullScreen = false;
        window->showNormal();
        apply(window);
        window->show();
    }
    window->requestActivate();
    emit changed();
}

}  // namespace arqade
