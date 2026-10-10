// Confinement: what a game's logic may reach in the QQ Player (ADR-087 decision 2).
//
// A game's scenes and logic are QML and JavaScript, run in the Player's engine. Confined, that engine:
//
//   loads QML, scripts and files only from the game's own package (its project folder, or the executable's bundled
//     game) and from Qt's and QQ's module folders;
//   imports only the modules a game needs (allowedModules()): Qt Quick, Quick 3D and its physics, particles, helpers and
//     spatial audio, Controls, and QQ; not storage, settings, devices, dialogs, sockets or workers;
//   makes no network request at all;
//   reads scenes only from the package through SceneIO, and writes or copies no file (SceneIO::confineTo);
//   refuses to hand URLs to the operating system (Qt.openUrlExternally) for every scheme in refusedSchemes().
//
// It is a second wall, not the boundary: a stranger's game plays only in the browser, whose sandbox is the boundary
// (ADR-087 decision 1). Two limits are known, and the checks (tst_lockdown) show both:
//
//   Qt lets an application refuse URLs only scheme by scheme, so a scheme not in refusedSchemes() still reaches the
//     operating system. The list covers the common ones.
//   Qt Multimedia (its camera and microphone types among them) cannot be refused natively: Qt's spatial audio library
//     links Qt Multimedia's QML library, which registers its types when it loads, and a module whose types are
//     registered is imported without its qmldir, which is all an interceptor sees. Natively only a person's own games
//     play (ADR-087 decision 3); in a browser, the browser asks the player before any camera or microphone is used.

#pragma once

#include <QStringList>
#include <QUrl>

class QQmlEngine;

namespace qq {

class Confinement
{
public:
    /// Confine `engine` to the game whose package is `package` (a folder: file: or qrc:). Call before anything is
    /// loaded into the engine. There is no way back.
    static void apply(QQmlEngine *engine, const QUrl &package);

    /// The package a scene file belongs to: the folder above `scenes/`, or the scene's own folder.
    static QUrl packageOf(const QUrl &scene);

    /// The modules a confined game may import, as module paths ("QtQuick/Layouts").
    static const QStringList &allowedModules();
    /// The URL schemes refused to Qt.openUrlExternally.
    static const QStringList &refusedSchemes();

    /// What has been refused so far, oldest first (each "kind: what"): for the log and the checks.
    static QStringList refusals();
};

}  // namespace qq
