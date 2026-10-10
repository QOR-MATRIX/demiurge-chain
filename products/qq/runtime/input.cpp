#include "input.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>

#include <cmath>

#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <xinput.h>
#endif

namespace qq {

namespace {

/// The keys Input reads; any other is left alone entirely.
bool isGameKey(int key)
{
    switch (key) {
    case Qt::Key_W: case Qt::Key_A: case Qt::Key_S: case Qt::Key_D:
    case Qt::Key_Up: case Qt::Key_Down: case Qt::Key_Left: case Qt::Key_Right:
    case Qt::Key_Space: case Qt::Key_Shift:
        return true;
    default:
        return false;
    }
}

bool isText(const QObject *o)
{
    return o && (o->inherits("QQuickTextInput") || o->inherits("QQuickTextEdit"));
}

/// Whether a key on its way to `receiver` is being typed into a text field, which a game must not read: the receiver
/// itself, the item with focus in the window it is sent to, or the application's focus.
bool typing(const QObject *receiver)
{
    if (isText(receiver))
        return true;
    if (const auto *window = qobject_cast<const QQuickWindow *>(receiver))
        return isText(window->activeFocusItem());
    return isText(QGuiApplication::focusObject());
}

QVector2D clampedToOne(QVector2D v)
{
    const float length = v.length();
    return length > 1.0f ? v / length : v;
}

#ifdef Q_OS_WIN
constexpr int pollMs = 16;
// XInput is slow to ask about an empty slot, so with no pad connected the slots are asked about every two seconds.
constexpr int idlePollsBetweenScans = 2000 / pollMs;
#endif

}  // namespace

QVector2D withDeadZone(QVector2D stick, float dead)
{
    const float length = stick.length();
    if (length <= dead)
        return {};
    const float scaled = std::min(1.0f, (length - dead) / (1.0f - dead));
    return stick / length * scaled;
}

Input::Input(QObject *parent) : QObject(parent)
{
    if (qApp)
        qApp->installEventFilter(this);
    connect(qGuiApp, &QGuiApplication::applicationStateChanged, this, [this](Qt::ApplicationState state) {
        if (state != Qt::ApplicationActive)
            release();
    });
#ifdef Q_OS_WIN
    m_poll = new QTimer(this);
    m_poll->setInterval(pollMs);
    connect(m_poll, &QTimer::timeout, this, &Input::pollGamepad);
    m_poll->start();
#endif
}

Input::~Input()
{
    if (qApp)
        qApp->removeEventFilter(this);
}

Input *Input::create(QQmlEngine *, QJSEngine *)
{
    return new Input;
}

QVector2D Input::move() const
{
    auto held = [this](int a, int b = 0) { return m_keys.contains(a) || (b && m_keys.contains(b)) ? 1.0f : 0.0f; };
    const QVector2D keys(held(Qt::Key_D) - held(Qt::Key_A), held(Qt::Key_W, Qt::Key_Up) - held(Qt::Key_S, Qt::Key_Down));
    return clampedToOne(keys + withDeadZone(m_padLeft));
}

QVector2D Input::look() const
{
    const float turn = (m_keys.contains(Qt::Key_Right) ? 1.0f : 0.0f) - (m_keys.contains(Qt::Key_Left) ? 1.0f : 0.0f);
    return clampedToOne(QVector2D(turn, 0) + withDeadZone(m_padRight));
}

void Input::setReadsHardware(bool reads)
{
    if (reads == m_readsHardware)
        return;
    m_readsHardware = reads;
    m_padIndex = -1;
    m_padIdlePolls = 0;
    emit readsHardwareChanged();
}

void Input::pointerMoved(QPointF delta)
{
    m_pointer += delta;
}

QPointF Input::takePointer()
{
    const QPointF taken = m_pointer;
    m_pointer = {};
    return taken;
}

void Input::gamepadState(QVector2D left, QVector2D right, bool jump, bool run, bool connected)
{
    const bool wasConnected = m_padConnected;
    if (!connected) {
        left = right = {};
        jump = run = false;
    }
    if (left == m_padLeft && right == m_padRight && jump == m_padJump && run == m_padRun && connected == wasConnected)
        return;
    m_padLeft = left;
    m_padRight = right;
    m_padJump = jump;
    m_padRun = run;
    m_padConnected = connected;
    emit changed();
    if (connected != wasConnected)
        emit gamepadChanged();
}

void Input::release()
{
    m_pointer = {};
    if (m_keys.isEmpty())
        return;
    m_keys.clear();
    emit changed();
}

bool Input::eventFilter(QObject *watched, QEvent *event)
{
    const QEvent::Type type = event->type();
    if (type == QEvent::KeyPress || type == QEvent::KeyRelease) {
        const auto *key = static_cast<QKeyEvent *>(event);
        if (!key->isAutoRepeat() && isGameKey(key->key())) {
            const bool changedNow = type == QEvent::KeyPress ? (!typing(watched) && !m_keys.contains(key->key()))
                                                             : m_keys.remove(key->key());
            if (type == QEvent::KeyPress && changedNow)
                m_keys.insert(key->key());
            if (changedNow)
                emit changed();
        }
    } else if (type == QEvent::FocusOut && watched->isWindowType()) {
        release();
    }
    return QObject::eventFilter(watched, event);
}

void Input::pollGamepad()
{
#ifdef Q_OS_WIN
    if (!m_readsHardware)
        return;
    if (m_padIndex < 0) {
        if (m_padIdlePolls++ % idlePollsBetweenScans != 0)
            return;
        for (DWORD i = 0; i < XUSER_MAX_COUNT; ++i) {
            XINPUT_STATE state{};
            if (XInputGetState(i, &state) == ERROR_SUCCESS) {
                m_padIndex = int(i);
                break;
            }
        }
        if (m_padIndex < 0)
            return;
    }
    XINPUT_STATE state{};
    if (XInputGetState(DWORD(m_padIndex), &state) != ERROR_SUCCESS) {
        m_padIndex = -1;
        m_padIdlePolls = 1;
        gamepadState({}, {}, false, false, false);
        return;
    }
    const XINPUT_GAMEPAD &pad = state.Gamepad;
    auto axis = [](SHORT v) { return std::max(-1.0f, float(v) / 32767.0f); };
    const WORD runButtons = XINPUT_GAMEPAD_LEFT_THUMB | XINPUT_GAMEPAD_LEFT_SHOULDER | XINPUT_GAMEPAD_RIGHT_SHOULDER;
    gamepadState(QVector2D(axis(pad.sThumbLX), axis(pad.sThumbLY)), QVector2D(axis(pad.sThumbRX), axis(pad.sThumbRY)),
                 (pad.wButtons & XINPUT_GAMEPAD_A) != 0, (pad.wButtons & runButtons) != 0, true);
#endif
}

}  // namespace qq
