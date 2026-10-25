// The games suite plays by the keys, and Esc and every other shortcut need its window to be the active one. Windows lets
// a program take the keyboard only while it is the one in front, or was started by it: a suite run by a build in the
// background, or one whose window something else on the PC has taken the keyboard from (an overlay, a notification),
// gets a window that only flashes in the taskbar. Asked here, Qt brings it to the front anyway, as a click on it would,
// so a suite's result is the code's and not the desktop's (HANDOFF §5). QOR Media's tests keep the same header.

#pragma once

#include <QGuiApplication>
#include <QTest>
#include <QTimer>
#include <QWindow>

#ifdef Q_OS_WIN
#include <QtGui/private/qguiapplication_p.h>
#endif

namespace activewindow {

/// Lets this suite's windows take the keyboard from whatever has it. Called once, before a window is asked to.
inline void mayTakeTheKeyboard()
{
#ifdef Q_OS_WIN
    using QNativeInterface::Private::QWindowsApplication;
    if (auto *windows = qGuiApp->nativeInterface<QWindowsApplication>())
        windows->setWindowActivationBehavior(QWindowsApplication::AlwaysActivateWindow);
#endif
}

/// Brings `window` to the front with the keyboard, and waits until it has it: asked again every half second, for five
/// seconds at most, since a window just shown is not always given it the first time.
[[nodiscard]] inline bool activate(QWindow *window)
{
    for (int i = 0; i < 10; ++i) {
        window->requestActivate();
        if (QTest::qWaitForWindowActive(window, 500))
            return true;
    }
    return false;
}

/// Asks for the keyboard back for `window`, now and then every tenth of a second, `tries` times at most, until it has it.
inline void reclaim(QWindow *window, int tries)
{
    if (window->isActive() || !window->isVisible() || tries == 0)
        return;
    window->requestActivate();
    QTimer::singleShot(100, window, [window, tries] { reclaim(window, tries - 1); });
}

/// Keeps the keyboard on `window` while it shows: whenever something else takes it mid-test (on the owner's PC, a
/// Linux program's window coming to the front through WSL), the window asks for it back at the next turn of the event
/// loop, and for a second after. A key a test sends while another window has it is lost, so the sooner the better.
inline void holdTheKeyboard(QWindow *window)
{
    QObject::connect(window, &QWindow::activeChanged, window, [window] {
        if (!window->isActive() && window->isVisible())
            QTimer::singleShot(0, window, [window] { reclaim(window, 10); });
    });
}

}  // namespace activewindow
