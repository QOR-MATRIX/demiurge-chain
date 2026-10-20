#include "input.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>

#include <algorithm>
#include <cmath>

#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <xinput.h>
#endif

namespace qq {

namespace {

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

QString mouseName(Qt::MouseButton button)
{
    switch (button) {
    case Qt::LeftButton: return QStringLiteral("Mouse Left");
    case Qt::RightButton: return QStringLiteral("Mouse Right");
    case Qt::MiddleButton: return QStringLiteral("Mouse Middle");
    default: return {};
    }
}

/// Jump and run, as they always were: what a scene's InputAction restates, and what forget() goes back to.
struct BuiltIn {
    const char *name;
    QStringList keys;
    QStringList pad;
};
const QList<BuiltIn> &builtIns()
{
    static const QList<BuiltIn> list{
        {"jump", {QStringLiteral("Space")}, {QStringLiteral("A")}},
        {"run", {QStringLiteral("Shift")}, {QStringLiteral("LS"), QStringLiteral("LB"), QStringLiteral("RB")}},
    };
    return list;
}

QStringList cleaned(const QStringList &names)
{
    QStringList out;
    for (const QString &n : names) {
        const QString t = n.trimmed();
        if (!t.isEmpty() && !out.contains(t, Qt::CaseInsensitive))
            out << t;
    }
    return out;
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

QString Input::keyName(int key)
{
    if (key >= Qt::Key_A && key <= Qt::Key_Z)
        return QString(QChar(char('A' + (key - Qt::Key_A))));
    if (key >= Qt::Key_0 && key <= Qt::Key_9)
        return QString(QChar(char('0' + (key - Qt::Key_0))));
    if (key >= Qt::Key_F1 && key <= Qt::Key_F12)
        return QStringLiteral("F%1").arg(key - Qt::Key_F1 + 1);
    switch (key) {
    case Qt::Key_Space: return QStringLiteral("Space");
    case Qt::Key_Return: case Qt::Key_Enter: return QStringLiteral("Enter");
    case Qt::Key_Escape: return QStringLiteral("Escape");
    case Qt::Key_Tab: return QStringLiteral("Tab");
    case Qt::Key_Backspace: return QStringLiteral("Backspace");
    case Qt::Key_Shift: return QStringLiteral("Shift");
    case Qt::Key_Control: return QStringLiteral("Ctrl");
    case Qt::Key_Alt: return QStringLiteral("Alt");
    case Qt::Key_Up: return QStringLiteral("Up");
    case Qt::Key_Down: return QStringLiteral("Down");
    case Qt::Key_Left: return QStringLiteral("Left");
    case Qt::Key_Right: return QStringLiteral("Right");
    default: return {};
    }
}

int Input::keyOf(const QString &name)
{
    static const QHash<QString, int> keys = [] {
        QHash<QString, int> all;
        for (int k = Qt::Key_A; k <= Qt::Key_Z; ++k)
            all.insert(keyName(k).toLower(), k);
        for (int k = Qt::Key_0; k <= Qt::Key_9; ++k)
            all.insert(keyName(k).toLower(), k);
        for (int k = Qt::Key_F1; k <= Qt::Key_F12; ++k)
            all.insert(keyName(k).toLower(), k);
        for (int k : {Qt::Key_Space, Qt::Key_Return, Qt::Key_Escape, Qt::Key_Tab, Qt::Key_Backspace, Qt::Key_Shift,
                      Qt::Key_Control, Qt::Key_Alt, Qt::Key_Up, Qt::Key_Down, Qt::Key_Left, Qt::Key_Right})
            all.insert(keyName(k).toLower(), k);
        all.insert(QStringLiteral("esc"), Qt::Key_Escape);
        all.insert(QStringLiteral("return"), Qt::Key_Return);
        all.insert(QStringLiteral("control"), Qt::Key_Control);
        return all;
    }();
    return keys.value(name.trimmed().toLower(), 0);
}

Input::Input(QObject *parent) : QObject(parent)
{
    for (const BuiltIn &b : builtIns())
        m_actions.insert(QLatin1String(b.name), {b.keys, b.pad});
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
    if (m_replaying)
        return m_replayMove;
    auto held = [this](int a, int b = 0) { return m_keys.contains(a) || (b && m_keys.contains(b)) ? 1.0f : 0.0f; };
    const QVector2D keys(held(Qt::Key_D) - held(Qt::Key_A), held(Qt::Key_W, Qt::Key_Up) - held(Qt::Key_S, Qt::Key_Down));
    return clampedToOne(keys + withDeadZone(m_padLeft));
}

QVector2D Input::look() const
{
    if (m_replaying)
        return m_replayLook;
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
    QSet<QString> buttons;
    if (jump)
        buttons.insert(QStringLiteral("A"));
    if (run)
        buttons.insert(QStringLiteral("LB"));
    padState(left, right, buttons, connected);
}

void Input::gamepadButtons(const QStringList &down)
{
    QSet<QString> buttons;
    for (const QString &b : down)
        buttons.insert(b.trimmed());
    padState(m_padLeft, m_padRight, buttons, true);
}

void Input::padState(QVector2D left, QVector2D right, const QSet<QString> &buttons, bool connected)
{
    const bool wasConnected = m_padConnected;
    QSet<QString> down = connected ? buttons : QSet<QString>();
    if (!connected)
        left = right = {};
    // A pad button pressed while listening becomes the binding, and counts for nothing else until it is let go.
    m_swallowedPad &= down;
    if (!m_listening.isEmpty()) {
        for (const QString &b : std::as_const(down)) {
            if (!m_pad.contains(b) && !m_swallowedPad.contains(b)) {
                m_swallowedPad.insert(b);
                bind(true, b);
                break;
            }
        }
    }
    down -= m_swallowedPad;
    if (left == m_padLeft && right == m_padRight && down == m_pad && connected == wasConnected)
        return;
    m_padLeft = left;
    m_padRight = right;
    m_pad = down;
    m_padConnected = connected;
    settle();
    emit changed();
    if (connected != wasConnected)
        emit gamepadChanged();
}

void Input::release()
{
    m_pointer = {};
    if (m_keys.isEmpty() && m_mouse.isEmpty() && m_holds.isEmpty())
        return;
    m_keys.clear();
    m_mouse.clear();
    m_holds.clear();
    settle();
    emit changed();
}

void Input::define(const QString &name, const QStringList &keys, const QStringList &pad)
{
    if (name.trimmed().isEmpty())
        return;
    m_actions.insert(name.trimmed(), {cleaned(keys), cleaned(pad)});
    settle();
}

void Input::forget(const QString &name)
{
    m_holds.remove(name);
    m_actions.remove(name);
    for (const BuiltIn &b : builtIns()) {
        if (name == QLatin1String(b.name))
            m_actions.insert(name, {b.keys, b.pad});
    }
    settle();
}

bool Input::held(const QString &name) const
{
    return m_replaying ? m_replayHeld.contains(name) : m_down.contains(name);
}

QVariantMap Input::takeStep()
{
    const QVector2D m = move();
    const QVector2D l = look();
    QStringList down(m_down.cbegin(), m_down.cend());
    down.sort();
    const QVariantMap step{{QStringLiteral("move"), QVariantList{double(m.x()), double(m.y())}},
                           {QStringLiteral("look"), QVariantList{double(l.x()), double(l.y())}},
                           {QStringLiteral("held"), down},
                           {QStringLiteral("edges"), m_edges}};
    m_edges.clear();
    return step;
}

void Input::replayHands(const QVariantMap &hands)
{
    const QVariantList m = hands.value(QStringLiteral("move")).toList();
    const QVariantList l = hands.value(QStringLiteral("look")).toList();
    m_replayMove = QVector2D(m.value(0).toFloat(), m.value(1).toFloat());
    m_replayLook = QVector2D(l.value(0).toFloat(), l.value(1).toFloat());
    const QStringList held = hands.value(QStringLiteral("held")).toStringList();
    m_replayHeld = QSet<QString>(held.cbegin(), held.cend());
    if (!m_replaying) {
        m_replaying = true;
        emit replayingChanged();
    }
    emit changed();
}

void Input::endReplay()
{
    m_replayMove = {};
    m_replayLook = {};
    m_replayHeld.clear();
    m_edges.clear();
    if (m_replaying) {
        m_replaying = false;
        emit replayingChanged();
    }
    emit changed();
}

void Input::hold(const QString &name, bool down)
{
    if (down)
        m_holds.insert(name);
    else
        m_holds.remove(name);
    settle();
}

void Input::listen(const QString &name)
{
    if (name == m_listening)
        return;
    m_listening = name;
    emit listeningChanged();
}

QString Input::describe(const QString &name) const
{
    const auto it = m_actions.constFind(name);
    if (it == m_actions.cend())
        return {};
    QStringList parts = it->keys;
    for (const QString &b : it->pad)
        parts << QStringLiteral("Pad ") + b;
    return parts.join(QStringLiteral(" / "));
}

QStringList Input::actions() const
{
    QStringList names = m_actions.keys();
    names.sort();
    return names;
}

bool Input::keyHeld(const QString &binding) const
{
    if (binding.startsWith(QStringLiteral("Mouse "), Qt::CaseInsensitive))
        return m_mouse.contains(binding);
    const int key = keyOf(binding);
    return key != 0 && m_keys.contains(key);
}

void Input::bind(bool pad, const QString &binding)
{
    const QString action = m_listening;
    m_listening.clear();
    Binding &b = m_actions[action];
    (pad ? b.pad : b.keys) = QStringList{binding};
    emit listeningChanged();
    emit rebound(action);
    settle();
}

void Input::settle()
{
    QSet<QString> down;
    for (auto it = m_actions.cbegin(); it != m_actions.cend(); ++it) {
        bool on = m_holds.contains(it.key());
        for (const QString &k : it->keys)
            on = on || keyHeld(k);
        for (const QString &p : it->pad)
            on = on || m_pad.contains(p);
        if (on)
            down.insert(it.key());
    }
    for (const QString &held : std::as_const(m_holds))
        down.insert(held);  // a hold may name an action the scene has not declared: it is still held
    if (down == m_down)
        return;
    const QSet<QString> before = m_down;
    m_down = down;
    QStringList went(before.cbegin(), before.cend());
    QStringList came(down.cbegin(), down.cend());
    went.sort();
    came.sort();
    // Kept for the next tick as well as announced: a press and its release between two ticks still reach logic. Only
    // the most recent are kept, so hands left untaken while nothing plays cannot grow without end.
    auto keep = [this](const char *what, const QString &action) {
        m_edges.append(QVariant(QVariantList{QString::fromLatin1(what), action}));
        if (m_edges.size() > 256)
            m_edges.removeFirst();
    };
    for (const QString &a : std::as_const(went)) {
        if (!down.contains(a)) {
            keep("released", a);
            emit released(a);
        }
    }
    for (const QString &a : std::as_const(came)) {
        if (!before.contains(a)) {
            keep("pressed", a);
            emit pressed(a);
        }
    }
    emit changed();
}

bool Input::eventFilter(QObject *watched, QEvent *event)
{
    // Only as they reach a window: from there a press goes on to the item with focus, and would be seen twice. While a
    // replay plays, the hands are its, and no device is read.
    if (!watched->isWindowType() || m_replaying)
        return QObject::eventFilter(watched, event);
    switch (event->type()) {
    case QEvent::KeyPress:
    case QEvent::KeyRelease: {
        const auto *key = static_cast<QKeyEvent *>(event);
        if (key->isAutoRepeat())
            break;
        if (event->type() == QEvent::KeyPress) {
            if (typing(watched))
                break;
            const QString name = keyName(key->key());
            if (!m_listening.isEmpty() && !name.isEmpty()) {
                bind(false, name);  // the press that names the binding does nothing else
                break;
            }
            if (!m_keys.contains(key->key())) {
                m_keys.insert(key->key());
                settle();
                emit changed();
            }
        } else if (m_keys.remove(key->key())) {
            settle();
            emit changed();
        }
        break;
    }
    case QEvent::MouseButtonPress:
    case QEvent::MouseButtonRelease: {
        const auto *mouse = static_cast<QMouseEvent *>(event);
        const QString name = mouseName(mouse->button());
        if (mouse->position() != m_pointerAt) {
            m_pointerAt = mouse->position();
            emit pointerChanged();
        }
        if (name.isEmpty())
            break;
        if (event->type() == QEvent::MouseButtonPress) {
            if (!m_listening.isEmpty()) {
                bind(false, name);
                break;
            }
            if (!m_mouse.contains(name)) {
                m_mouse.insert(name);
                settle();
            }
        } else if (m_mouse.remove(name)) {
            settle();
        }
        break;
    }
    case QEvent::MouseMove: {
        const QPointF at = static_cast<QMouseEvent *>(event)->position();
        if (at != m_pointerAt) {
            m_pointerAt = at;
            emit pointerChanged();
        }
        break;
    }
    case QEvent::FocusOut:
        release();
        break;
    default:
        break;
    }
    return QObject::eventFilter(watched, event);
}

void Input::pollGamepad()
{
#ifdef Q_OS_WIN
    if (!m_readsHardware || m_replaying)
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
        padState({}, {}, {}, false);
        return;
    }
    const XINPUT_GAMEPAD &pad = state.Gamepad;
    auto axis = [](SHORT v) { return std::max(-1.0f, float(v) / 32767.0f); };
    static const QList<QPair<WORD, QString>> names{
        {XINPUT_GAMEPAD_A, QStringLiteral("A")},
        {XINPUT_GAMEPAD_B, QStringLiteral("B")},
        {XINPUT_GAMEPAD_X, QStringLiteral("X")},
        {XINPUT_GAMEPAD_Y, QStringLiteral("Y")},
        {XINPUT_GAMEPAD_LEFT_SHOULDER, QStringLiteral("LB")},
        {XINPUT_GAMEPAD_RIGHT_SHOULDER, QStringLiteral("RB")},
        {XINPUT_GAMEPAD_BACK, QStringLiteral("Back")},
        {XINPUT_GAMEPAD_START, QStringLiteral("Start")},
        {XINPUT_GAMEPAD_LEFT_THUMB, QStringLiteral("LS")},
        {XINPUT_GAMEPAD_RIGHT_THUMB, QStringLiteral("RS")},
        {XINPUT_GAMEPAD_DPAD_UP, QStringLiteral("Up")},
        {XINPUT_GAMEPAD_DPAD_DOWN, QStringLiteral("Down")},
        {XINPUT_GAMEPAD_DPAD_LEFT, QStringLiteral("Left")},
        {XINPUT_GAMEPAD_DPAD_RIGHT, QStringLiteral("Right")},
    };
    QSet<QString> buttons;
    for (const auto &[bit, name] : names) {
        if (pad.wButtons & bit)
            buttons.insert(name);
    }
    // A trigger counts as pressed past a little travel, so a finger resting on it does not.
    if (pad.bLeftTrigger > XINPUT_GAMEPAD_TRIGGER_THRESHOLD)
        buttons.insert(QStringLiteral("LT"));
    if (pad.bRightTrigger > XINPUT_GAMEPAD_TRIGGER_THRESHOLD)
        buttons.insert(QStringLiteral("RT"));
    padState(QVector2D(axis(pad.sThumbLX), axis(pad.sThumbLY)), QVector2D(axis(pad.sThumbRX), axis(pad.sThumbRY)), buttons,
             true);
#endif
}

}  // namespace qq
