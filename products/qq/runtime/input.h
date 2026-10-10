// Input: what a player is asking for, from whichever device they hold (DIRECTION P3.2).
//
// Keyboard, mouse and gamepad become one small set of intentions: move (left/right, back/forward), look (turn, tilt),
// jump and run. A game reads the intentions and never a device, so every game works with every device, and the agent
// (P3.3) can play one by setting the same intentions.
//
//   keyboard   W A S D or the arrows to move (Left and Right turn), Space jumps, Shift runs
//   mouse      a drag looks around: what the window's pointer handler hands to pointerMoved()
//   gamepad    left stick moves, right stick looks, A (south) jumps, the left stick pressed or a shoulder runs
//
// The keyboard is read from the whole application, except while a text field has focus, so typing a name never walks
// the player. A gamepad is read through XInput on Windows; anywhere else only keyboard and mouse.

#pragma once

#include <QObject>
#include <QPointF>
#include <QSet>
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

public:
    explicit Input(QObject *parent = nullptr);
    ~Input() override;

    static Input *create(QQmlEngine *qml, QJSEngine *js);

    QVector2D move() const;
    QVector2D look() const;
    bool jump() const { return m_keys.contains(Qt::Key_Space) || m_padJump; }
    bool run() const { return m_keys.contains(Qt::Key_Shift) || m_padRun; }
    bool gamepad() const { return m_padConnected; }
    bool readsHardware() const { return m_readsHardware; }
    void setReadsHardware(bool reads);

    /// The pointer moved by `delta` pixels while looking (a drag in the viewport).
    Q_INVOKABLE void pointerMoved(QPointF delta);
    /// How far the pointer moved since this was last asked, in pixels; asking clears it. A game asks once a frame.
    Q_INVOKABLE QPointF takePointer();

    /// A gamepad's state: sticks from -1 to 1 (y up), with the dead zone not yet applied. XInput readings arrive here,
    /// and so can a test's or the agent's.
    Q_INVOKABLE void gamepadState(QVector2D left, QVector2D right, bool jump, bool run, bool connected = true);

    /// Let go of everything: what happens when the window loses focus, so no key is held down for ever.
    Q_INVOKABLE void release();

signals:
    void changed();
    void gamepadChanged();
    void readsHardwareChanged();

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    void pollGamepad();

    QSet<int> m_keys;
    QPointF m_pointer;
    QVector2D m_padLeft;
    QVector2D m_padRight;
    bool m_padJump = false;
    bool m_padRun = false;
    bool m_padConnected = false;
    int m_padIndex = -1;
    int m_padIdlePolls = 0;
    bool m_readsHardware = true;
    QTimer *m_poll = nullptr;
};

/// A stick reading with its dead zone taken out: zero inside `dead`, rising smoothly to full at the rim.
QVector2D withDeadZone(QVector2D stick, float dead = 0.24f);

}  // namespace qq
