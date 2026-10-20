// Input: what a player is asking for, from whichever device they hold (DIRECTION P3.2, P3.9).
//
// Keyboard, mouse and gamepad become one small set of intentions: move (left/right, back/forward), look (turn, tilt),
// jump and run. A game reads the intentions and never a device, so every game works with every device, and the agent
// (P3.3) can play one by setting the same intentions.
//
//   keyboard   W A S D or the arrows to move (Left and Right turn), Space jumps, Shift runs
//   mouse      a drag looks around: what the window's pointer handler hands to pointerMoved()
//   gamepad    left stick moves, right stick looks, A (south) jumps, the left stick pressed or a shoulder runs
//
// Actions (P3.9, ADR-105): a game names what else a player can do ("fire", "interact", "pause", …), each with keys
// and mouse buttons ("E", "Space", "Mouse Left") and gamepad buttons ("X", "RT", "Start"), declared by the scene's
// InputAction entities. An action is held while any of its bindings is, or while something holds it outright: a
// ScreenButton under a finger, the agent, a check. Jump and run are actions too, built in with the bindings above, so
// they can be rebound like any other; move and look stay fixed. A player rebinds an action by asking Input to listen:
// the next key, mouse button or pad button they press becomes its binding for that device.
//
// The keyboard is read from the whole application, except while a text field has focus, so typing a name never walks
// the player. A gamepad is read through XInput on Windows; anywhere else only keyboard and mouse.
//
// Steps (P3.11): a playing scene takes the hands once a tick (takeStep): move, look, the actions held, and every press
// and release since the tick before, in order. That is what its logic and its player are given, so a run can be kept
// and played again (run.h). A replay gives the hands back (replayHands) and the devices are not read until it ends.

#pragma once

#include <QHash>
#include <QObject>
#include <QPointF>
#include <QSet>
#include <QStringList>
#include <QVariantList>
#include <QVariantMap>
#include <QVector2D>
#include <QtQml/qqmlregistration.h>

class QQmlEngine;
class QJSEngine;
class QTimer;

namespace qq {

class Input : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    /// Where to go: x to the right, y forward, each from -1 to 1, never longer than 1.
    Q_PROPERTY(QVector2D move READ move NOTIFY changed)
    /// How fast to turn (x, to the right) and tilt (y, upward), each from -1 to 1.
    Q_PROPERTY(QVector2D look READ look NOTIFY changed)
    Q_PROPERTY(bool jump READ jump NOTIFY changed)
    Q_PROPERTY(bool run READ run NOTIFY changed)
    /// Whether a gamepad is connected.
    Q_PROPERTY(bool gamepad READ gamepad NOTIFY gamepadChanged)
    /// Whether gamepads are read from the hardware (on by default). Off, only gamepadState() moves the pad: how a check,
    /// or the agent (P3.3), plays it without a real one taking over.
    Q_PROPERTY(bool readsHardware READ readsHardware WRITE setReadsHardware NOTIFY readsHardwareChanged)
    /// Where the pointer is, in the coordinates of the window it is over: what a game picks with (Logic.pointing).
    Q_PROPERTY(QPointF pointer READ pointer NOTIFY pointerChanged)
    /// The action waiting for a binding (listen()), or empty.
    Q_PROPERTY(QString listening READ listening NOTIFY listeningChanged)
    /// Whether the hands are a replay's (replayHands), and the devices are not read.
    Q_PROPERTY(bool replaying READ replaying NOTIFY replayingChanged)

public:
    explicit Input(QObject *parent = nullptr);
    ~Input() override;

    static Input *create(QQmlEngine *qml, QJSEngine *js);

    QVector2D move() const;
    QVector2D look() const;
    bool jump() const { return held(QStringLiteral("jump")); }
    bool run() const { return held(QStringLiteral("run")); }
    bool gamepad() const { return m_padConnected; }
    bool readsHardware() const { return m_readsHardware; }
    void setReadsHardware(bool reads);
    QPointF pointer() const { return m_pointerAt; }
    QString listening() const { return m_listening; }
    bool replaying() const { return m_replaying; }

    /// The pointer moved by `delta` pixels while looking (a drag in the viewport).
    Q_INVOKABLE void pointerMoved(QPointF delta);
    /// How far the pointer moved since this was last asked, in pixels; asking clears it. A game asks once a frame.
    Q_INVOKABLE QPointF takePointer();

    /// A gamepad's state: sticks from -1 to 1 (y up), with the dead zone not yet applied; jump is A held and run a
    /// shoulder held. A test's or the agent's pad arrives here; a real pad's through XInput.
    Q_INVOKABLE void gamepadState(QVector2D left, QVector2D right, bool jump, bool run, bool connected = true);
    /// The gamepad's buttons held now, by name (A, B, X, Y, LB, RB, LT, RT, Back, Start, LS, RS, Up, Down, Left,
    /// Right), its sticks left as they are: for a check, or the agent, pressing what a real pad would.
    Q_INVOKABLE void gamepadButtons(const QStringList &down);

    /// Let go of everything: what happens when the window loses focus, so no key is held down for ever.
    Q_INVOKABLE void release();

    // ── actions (P3.9) ──

    /// Name an action and bind it: keys and mouse buttons, and gamepad buttons, by name. Naming one that exists
    /// replaces its bindings (which is how a scene restates a built-in one).
    Q_INVOKABLE void define(const QString &name, const QStringList &keys, const QStringList &pad);
    /// Take an action away; a built-in one goes back to its own bindings.
    Q_INVOKABLE void forget(const QString &name);
    /// Whether an action is held now.
    Q_INVOKABLE bool held(const QString &name) const;
    /// Hold an action outright, or let it go: a ScreenButton under a finger, the agent, a check.
    Q_INVOKABLE void hold(const QString &name, bool down);
    /// The next key or mouse button pressed becomes `name`'s keyboard binding, or the next pad button its pad binding.
    /// The press does nothing else. An empty name stops listening.
    Q_INVOKABLE void listen(const QString &name);
    /// An action's bindings as a person reads them: "E / Mouse Left / Pad X".
    Q_INVOKABLE QString describe(const QString &name) const;
    /// The actions there are, by name.
    Q_INVOKABLE QStringList actions() const;

    // ── steps (P3.11) ──

    /// The hands for one tick: { move: [x, y], look: [x, y], held: [names, sorted], edges: [["pressed", name] or
    /// ["released", name], in the order they came] }. The edges are those since the last call, which it clears.
    Q_INVOKABLE QVariantMap takeStep();
    /// Play `hands` (as takeStep gave them) as the player's own: move, look and held come from them, and no device is
    /// read, until endReplay().
    Q_INVOKABLE void replayHands(const QVariantMap &hands);
    Q_INVOKABLE void endReplay();

    /// A key as a binding names it ("E", "Space", "Shift", "F1"), or empty for one that cannot be bound.
    static QString keyName(int key);
    /// A binding's key, or 0 if the name is not a key.
    static int keyOf(const QString &name);

signals:
    void changed();
    void gamepadChanged();
    void readsHardwareChanged();
    void pointerChanged();
    void listeningChanged();
    void replayingChanged();
    /// An action became held, or stopped being held.
    void pressed(const QString &action);
    void released(const QString &action);
    /// A listened-for binding arrived.
    void rebound(const QString &action);

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    struct Binding {
        QStringList keys;
        QStringList pad;
    };

    void pollGamepad();
    void padState(QVector2D left, QVector2D right, const QSet<QString> &buttons, bool connected);
    /// Work out which actions are held now; announce each change, and `changed` if anything did.
    void settle();
    bool keyHeld(const QString &binding) const;
    void bind(bool pad, const QString &binding);

    QHash<QString, Binding> m_actions;
    QSet<QString> m_holds;
    QSet<QString> m_down;
    QSet<int> m_keys;
    QSet<QString> m_mouse;
    QSet<QString> m_pad;
    /// Pad buttons pressed to name a binding, which count for nothing else until they are let go.
    QSet<QString> m_swallowedPad;
    QPointF m_pointer;
    QPointF m_pointerAt;
    QString m_listening;
    QVector2D m_padLeft;
    QVector2D m_padRight;
    bool m_padConnected = false;
    int m_padIndex = -1;
    int m_padIdlePolls = 0;
    bool m_readsHardware = true;
    QTimer *m_poll = nullptr;
    /// Presses and releases since the last takeStep(), in order.
    QVariantList m_edges;
    bool m_replaying = false;
    QVector2D m_replayMove;
    QVector2D m_replayLook;
    QSet<QString> m_replayHeld;
};

/// A stick reading with its dead zone taken out: zero inside `dead`, rising smoothly to full at the rim.
QVector2D withDeadZone(QVector2D stick, float dead = 0.24f);

}  // namespace qq
